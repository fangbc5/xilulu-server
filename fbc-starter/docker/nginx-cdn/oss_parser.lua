-- oss_parser.lua
-- OpenResty access 阶段脚本
-- 解析 x-oss-process 参数，设置 ngx.var.oss_target 为代理目标 URL
--
-- 运行于 access_by_lua_file，职责：
--   1. 判断请求是否为 image 图片处理
--   2. 是 → 翻译为 imgproxy 签名 URL，设 $oss_target = http://imgproxy:8080/...
--   3. 否 → 设 $oss_target = http://host.docker.internal:30003/oss/...（回源 ms-oss）

-- ============================================
-- FFI：OpenSSL HMAC-SHA256
-- ============================================
local ffi = require("ffi")

-- 防止 worker 内重复定义
if not pcall(function() ffi.typeof("struct evp_md_st") end) then
    ffi.cdef[[
        typedef struct evp_md_st EVP_MD;
        const EVP_MD *EVP_sha256(void);
        unsigned char *HMAC(const EVP_MD *evp_md, const void *key, int key_len,
                            const unsigned char *d, size_t n, unsigned char *md,
                            unsigned int *md_len);
    ]]
end
local C = ffi.C

-- ============================================
-- 签名密钥（与 docker-compose-imgproxy.yml 一致）
-- ============================================
local KEY_HEX  = os.getenv("IMGPROXY_KEY")  or ""
local SALT_HEX = os.getenv("IMGPROXY_SALT") or ""

-- hex → binary
local function from_hex(s)
    return (s:gsub('..', function(cc) return string.char(tonumber(cc, 16)) end))
end

local KEY_BIN  = from_hex(KEY_HEX)
local SALT_BIN = from_hex(SALT_HEX)

-- ============================================
-- 工具函数
-- ============================================

-- URL-safe Base64（无 padding）
local function base64url(s)
    local b = ngx.encode_base64(s)
    return b:gsub('+', '-'):gsub('/', '_'):gsub('=', '')
end

-- HMAC-SHA256
local function hmac_sha256(key, data)
    local buf = ffi.new("unsigned char[32]")
    local len = ffi.new("unsigned int[1]")
    C.HMAC(C.EVP_sha256(), key, #key, data, #data, buf, len)
    return ffi.string(buf, len[0])
end

-- imgproxy 签名：HMAC(key, salt + path)
local function sign_path(path)
    return base64url(hmac_sha256(KEY_BIN, SALT_BIN .. path))
end

-- 字符串按分隔符切割
local function split(str, sep)
    local t = {}
    for m in (str .. sep):gmatch("(.-)" .. sep) do
        t[#t + 1] = m
    end
    return t
end

-- 从参数列表中提取 "prefix_value" 格式的值
local function extract(params, prefix)
    local pfx = prefix .. "_"
    for _, p in ipairs(params) do
        if p:sub(1, #pfx) == pfx then
            return p:sub(#pfx + 1)
        end
    end
    return nil
end

-- ============================================
-- 阿里云 OSS → imgproxy 指令翻译
-- ============================================

local function tr_resize(params)
    local mode_map = { lfit = "fit", fill = "fill", fixed = "force", mfit = "fill-down" }
    local m = extract(params, "m")
    local mode = mode_map[m] or "fit"
    local w = extract(params, "w") or "0"
    local h = extract(params, "h") or "0"
    local p = extract(params, "p")
    if p then
        return ("rs:fit:%sp:%sp"):format(p, p)
    end
    return ("rs:%s:%s:%s"):format(mode, w, h)
end

local function tr_crop(params)
    local grav_map = {
        center = "ce", centre = "ce", nw = "nowe", ne = "noea",
        sw = "sowe", se = "soea", north = "no", south = "so",
        west = "we", east = "ea",
    }
    local w = extract(params, "w") or "0"
    local h = extract(params, "h") or "0"
    local g = grav_map[extract(params, "g") or "center"] or "ce"
    local x, y = extract(params, "x"), extract(params, "y")
    if x and y then
        return ("c:%s:%s:%s:%s:%s"):format(w, h, g, x, y)
    end
    return ("c:%s:%s:%s"):format(w, h, g)
end

local function parse_image(commands)
    local parts = split(commands, "/")
    local imgproxy = {}
    local fmt = nil

    for _, raw in ipairs(parts) do
        if raw ~= "" then
            local kvs = split(raw, ",")
            local cmd = kvs[1]
            table.remove(kvs, 1)

            if cmd == "resize" then
                imgproxy[#imgproxy + 1] = tr_resize(kvs)
            elseif cmd == "crop" then
                imgproxy[#imgproxy + 1] = tr_crop(kvs)
            elseif cmd == "quality" then
                local q = extract(kvs, "q") or extract(kvs, "Q")
                if q then imgproxy[#imgproxy + 1] = "q:" .. q end
            elseif cmd == "format" then
                if #kvs > 0 then fmt = kvs[1] end
            end
        end
    end

    return table.concat(imgproxy, "/"), fmt
end

-- ============================================
-- 主逻辑
-- ============================================
local bucket = ngx.var.oss_bucket
local key    = ngx.var.oss_key

-- 默认：回源 ms-oss（携带完整原始路径和查询参数）
local fallback = "/oss/" .. (bucket or "") .. "/" .. (key or "")
local qs = ngx.var.args
if qs and qs ~= "" then
    fallback = fallback .. "?" .. qs
end
ngx.var.oss_target = "http://host.docker.internal:30003" .. fallback

-- 解析 x-oss-process（注意参数名带连字符，不能用 ngx.var.arg_xxx）
local uri_args   = ngx.req.get_uri_args()
local process    = uri_args["x-oss-process"]

if not process or process == "" then
    ngx.log(ngx.DEBUG, "[oss] 无 x-oss-process，回源 ms-oss")
    return
end

if not bucket or not key then
    ngx.log(ngx.WARN, "[oss] bucket/key 为空，回源 ms-oss")
    return
end

-- 仅拦截 image 类型（image 或 image/...）
if process:sub(1, 5) ~= "image" then
    ngx.log(ngx.INFO, "[oss] 非图片处理(", process:sub(1, 5), ")，回源 ms-oss")
    return
end

-- 提取 image 后面的指令部分（兼容 "image" 和 "image/..." 两种写法）
local commands = ""
if #process > 5 then
    -- 跳过 "image" 后面的 "/" (如果有)
    local start = 6
    if process:sub(6, 6) == "/" then start = 7 end
    commands = process:sub(start)
end
local processing, fmt = parse_image(commands)
if processing == "" then processing = "raw:1" end

-- 拼接 imgproxy path
local source = "s3://" .. bucket .. "/" .. key
local path
if fmt then
    path = ("/%s/plain/%s@%s"):format(processing, source, fmt)
else
    path = ("/%s/plain/%s"):format(processing, source)
end

-- 签名 + 设置目标
local sig = sign_path(path)
local target = "http://imgproxy:8080/" .. sig .. path

ngx.var.oss_target = target
ngx.log(ngx.INFO, "[oss] ✓ 拦截图片 → imgproxy: ", target)
