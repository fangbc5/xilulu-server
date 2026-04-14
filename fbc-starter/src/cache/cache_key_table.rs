/// 缓存 Key 表
///
/// 用于统一管理和生成缓存的 key，避免多个项目使用的 key 重复
///
/// 使用 @Cacheable 时，其 value 值一定要在此处指定

/// 验证码前缀
/// 完整 key: captcha:{key} -> str
pub const CAPTCHA: &str = "captcha";

/// Token 前缀
/// 完整 key: token:{token} -> userid
pub const TOKEN: &str = "token";

//------------------
// 权限系统缓存 start

/// 总登录次数
/// total_login_pv:{TENANT} -> Long
pub const TOTAL_LOGIN_PV: &str = "total_login_pv";

/// 今日登录次数
/// today_login_pv:{TENANT}:{today} -> Long
pub const TODAY_LOGIN_PV: &str = "today_login_pv";

/// 今日登录总 IP
/// today_login_iv:{TENANT}:{today} -> int
pub const TODAY_LOGIN_IV: &str = "today_login_iv";

/// 今日登录总 IP
/// TOTAL_LOGIN_IV:{TENANT} -> int
pub const TOTAL_LOGIN_IV: &str = "total_login_iv";

/// 今日 PV
/// today_pv:{TENANT} -> int
pub const TODAY_PV: &str = "today_pv";

/// 总 PV
/// total_pv:{TENANT} -> int
pub const TOTAL_PV: &str = "total_pv";

/// 最近 10 天访问记录
/// login_log_tenday:{TENANT}:{today}:{account} -> Map
pub const LOGIN_LOG_TEN_DAY: &str = "login_log_tenday";

/// 登录总次数
/// login_log_browser:{TENANT} -> Map
pub const LOGIN_LOG_BROWSER: &str = "login_log_browser";

/// 登录总次数
/// login_log_system{TENANT} -> Map
pub const LOGIN_LOG_SYSTEM: &str = "login_log_system";

/// 参数前缀
/// 完整 key: parameter_key:{key} -> obj
pub const PARAMETER_KEY: &str = "parameter_key";

/// 在线用户前缀
/// 完整 key: online:{userid} -> token (String)
pub const ONLINE: &str = "online";

/// 用户 token 前缀
/// 完整 key: token_user_id:{token} -> userid (Long)
pub const TOKEN_USER_ID: &str = "token_user_id";

/// 用户注册前缀
/// 完整 key: register:{注册类型}:{手机号}
pub const REGISTER_USER: &str = "register";

// 权限系统缓存 end

/// 好友相关缓存 Key
pub mod friend {
    /// 好友的关联映射
    pub const RELATION: &str = "friend_relation";

    /// 好友的反向映射
    pub const REVERSE_FRIENDS: &str = "reverse_friends";

    /// 好友关系
    pub const USER_FRIENDS: &str = "user_friends";

    /// 好友关联的状态映射
    pub const RELATION_STATUS: &str = "relation_status";
}

/// 在线状态相关缓存 Key
pub mod presence {
    /// 全局用户
    pub const GLOBAL_USERS_ONLINE: &str = "global_users_online";

    /// 全局用户映射
    pub const GLOBAL_DEVICES_ONLINE: &str = "global_devices_online";

    /// 用户有多少个群的映射
    pub const USERS_GROUP: &str = "users_group";

    /// 用户有多少个群中在线的映射
    pub const USERS_GROUP_ONLINE: &str = "users_group_online";

    /// 群组成员构建器
    pub const GROUP_MEMBERS: &str = "group_members";

    /// 群组在线成员构建器
    pub const GROUP_MEMBERS_ONLINE: &str = "group_members_online";
}

/// 视频通话相关缓存 Key
pub mod video_call {
    /// 管理员的元数据
    pub const META_DATA_ADMIN: &str = "meta_data_admin";
}

/// 系统相关缓存 Key
pub mod system {
    /// 系统缓存
    pub const SYS_CONFIG: &str = "sys_config";

    /// 租户
    pub const TENANT: &str = "def_tenant";

    /// 应用
    pub const APPLICATION: &str = "def_application";

    /// 默认字典
    pub const DICT: &str = "def_dict";

    /// 默认参数
    pub const DEF_PARAMETER: &str = "def_parameter";

    /// 用户前缀
    pub const DEF_USER: &str = "def_user";

    /// 客户端
    pub const DEF_CLIENT: &str = "def_client";

    /// 租户拥有的资源
    pub const TENANT_APPLICATION_RESOURCE: &str = "t_a_r";

    /// 租户拥有的应用
    pub const TENANT_APPLICATION: &str = "t_a";

    /// 资源
    pub const RESOURCE: &str = "dr";

    /// 资源接口
    pub const RESOURCE_API: &str = "dra";

    /// 应用的资源
    pub const APPLICATION_RESOURCE: &str = "app_res";

    pub const ALL_RESOURCE_API: &str = "all_dra";
}

/// 聊天相关缓存 Key
pub mod chat {
    /// 房间元数据
    pub const ROOM_META: &str = "room_meta";

    /// 关闭房间
    pub const CLOSE_ROOM: &str = "close_room";

    /// 密码
    pub const CHAT_PASSWORD: &str = "chat_password";

    /// chat-gpt
    pub const CHAT_GPT: &str = "chat_GPT";

    /// 认证
    pub const AUTH: &str = "auth";

    /// 异步
    pub const AYSNC: &str = "aysnc";

    /// 聊天 token
    pub const CHAT_TOKEN: &str = "chat_token";

    /// 用户
    pub const USER_CACHE: &str = "user_cache";

    /// 微信消息
    pub const WX_MSG: &str = "wxMsg";

    /// 用户在线状态
    pub const USER_STATE: &str = "user_state";

    /// 用户在线状态
    pub const ANNOUNCEMENTS: &str = "announcements";

    /// 朋友圈
    pub const FEED: &str = "feed";

    /// 朋友圈素材
    pub const FEED_MEDIA: &str = "feedMedia";

    /// 朋友圈权限
    pub const FEED_TARGET: &str = "feedTarget";

    /// 朋友圈评论
    pub const FEED_COMMENT: &str = "feedComment";

    /// 朋友圈点赞
    pub const FEED_LIKE: &str = "feedLike";

    /// 会话信息
    pub const USER_CONTACT: &str = "user_contact";

    /// 在途消息
    pub const PASSAGE_MSG: &str = "passage_msg";

    /// 群组信息（基于 group_id）
    pub const GROUP_INFO_FORMAT: &str = "group_info";

    /// 群组信息（基于 room_id）
    pub const ROOM_GROUP_INFO_FORMAT: &str = "room_group_info";

    /// 房间信息
    pub const ROOM_INFO_FORMAT: &str = "room_info";

    /// 单聊房间信息
    pub const ROOM_FRIEND_FORMAT: &str = "room_friend";

    /// 房间消息
    pub const ROOM_MSG_FORMAT: &str = "room_msg";

    /// 群公告已读数量
    pub const GROUP_ANNOUNCEMENTS_FORMAT: &str = "group_announcements";

    /// 热门房间 ZSet
    pub const HOT_ROOM_ZET: &str = "hot_room";
}

/// OAuth 相关缓存 Key
pub mod oauth {
    /// 租户自定义字典
    pub const QR: &str = "qr_status";
}

/// 基础服务相关缓存 Key
pub mod base {
    /// 租户自定义字典
    pub const BASE_DICT: &str = "base_dict";

    /// 组织前缀
    pub const BASE_ORG: &str = "base_org";

    /// 岗位前缀
    pub const BASE_POSITION: &str = "base_position";

    /// 员工前缀
    pub const BASE_EMPLOYEE: &str = "base_employee";

    /// 全局员工前缀
    pub const DEF_USER_TENANT: &str = "def_user_tenant";

    /// 角色前缀
    /// 完整 key: role:{roleId}
    pub const ROLE: &str = "role";

    /// 角色拥有那些资源前缀
    /// 完整 key: role_resource:{ROLE_ID} -> [RESOURCE_ID, ...]
    pub const ROLE_RESOURCE: &str = "role_resource";

    /// 员工拥有那些角色前缀
    /// 完整 key: employee_role:{EMPLOYEE_ID} -> [ROLE_ID, ...]
    pub const EMPLOYEE_ROLE: &str = "employee_role";

    /// 角色拥有那些组织前缀
    /// 完整 key: employee_org:{EMPLOYEE_ID} -> [ORG_ID, ...]
    pub const EMPLOYEE_ORG: &str = "employee_org";

    /// 角色拥有那些组织前缀
    /// 完整 key: org_role:{ORG_ID} -> [ROLE_ID, ...]
    pub const ORG_ROLE: &str = "org_role";
}
