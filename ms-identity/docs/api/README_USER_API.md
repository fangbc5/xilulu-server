# 用户模块 API 接口文档

## 概述

本文档描述了用户模块的所有 API 接口，包括用户注册、登录、信息管理、密码管理、租户关系管理等。

## 基础信息

- **Base URL**: `http://localhost:8080`
- **API 版本**: `v1`
- **认证方式**: Bearer Token (JWT)

## 接口列表

### 1. 发送验证码

**接口地址**: `POST /api/v1/users/send-code`

**接口描述**: 发送验证码到手机号或邮箱。限制180秒内不允许重复获取，验证码有效时间5分钟。

**请求参数**:
```json
{
  "mobile": "18888888888"  // 或
  "email": "user@example.com"
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": {
    "message": "验证码已发送"
  }
}
```

**特殊说明**:
- admin用户的邮箱 `admin@admin.com` 和手机号 `18888888888` 的验证码为 `888888`（永久有效）

---

### 2. 用户注册

**接口地址**: `POST /api/v1/users/register`

**接口描述**: 用户注册接口。支持三种注册方式：
1. 用户名+密码注册
2. 手机号+验证码注册
3. 邮箱+验证码注册

注册成功后自动将用户添加到租户0。

**请求参数**:

**方式1: 用户名密码注册**
```json
{
  "username": "testuser",
  "password": "123456",
  "email": "test@example.com",
  "mobile": "13800138000",
  "nick_name": "测试用户"
}
```

**方式2: 手机号验证码注册**
```json
{
  "mobile": "13800138000",
  "code": "123456",
  "nick_name": "测试用户"
}
```

**方式3: 邮箱验证码注册**
```json
{
  "email": "test@example.com",
  "code": "123456",
  "nick_name": "测试用户"
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": {
    "user_id": 1,
    "message": "注册成功"
  }
}
```

---

### 3. 用户登录

**接口地址**: `POST /api/v1/users/login`

**接口描述**: 用户登录接口。支持三种登录方式：
1. 用户名+密码登录
2. 手机号+验证码登录
3. 邮箱+验证码登录

登录成功后返回 access_token 和 refresh_token。

**请求参数**:

**方式1: 用户名密码登录**
```json
{
  "username": "admin",
  "password": "123456"
}
```

**方式2: 手机号验证码登录**
```json
{
  "mobile": "18888888888",
  "code": "888888"
}
```

**方式3: 邮箱验证码登录**
```json
{
  "email": "admin@admin.com",
  "code": "888888"
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": {
    "user": {
      "id": 1,
      "username": "admin",
      "email": "admin@admin.com",
      "mobile": "18888888888",
      "nick_name": "管理员",
      "state": 1
    },
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "access_token_expire": 900,
    "refresh_token_expire": 604800
  }
}
```

**特殊说明**:
- access_token 有效期为 15 分钟（900秒）
- refresh_token 有效期为 7 天（604800秒）
- admin用户的邮箱和手机号验证码为 `888888`（永久有效）

---

### 4. 刷新Token

**接口地址**: `POST /api/v1/users/refresh-token`

**接口描述**: 使用 refresh_token 刷新 access_token。只生成新的 access_token，不生成新的 refresh_token。

**请求参数**:
```json
{
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": {
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "access_token_expire": 900,
    "refresh_token_expire": 604800
  }
}
```

---

### 5. 用户登出

**接口地址**: `POST /api/v1/users/logout`

**接口描述**: 用户登出接口。由于使用短期 Access Token（15分钟），不需要黑名单机制，Token 会在过期后自然失效。

**请求头**:
```
Authorization: Bearer {access_token}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": null
}
```

---

### 6. 获取用户信息

**接口地址**: `GET /api/v1/users/{id}`

**接口描述**: 获取指定用户的信息。需要认证。

**路径参数**:
- `id` (integer, required): 用户ID

**请求头**:
```
Authorization: Bearer {access_token}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": {
    "id": 1,
    "username": "admin",
    "email": "admin@admin.com",
    "mobile": "18888888888",
    "nick_name": "管理员",
    "state": 1
  }
}
```

---

### 7. 创建用户

**接口地址**: `POST /api/v1/users/`

**接口描述**: 创建新用户（管理员接口）。需要认证，创建人从JWT中获取。

**请求头**:
```
Authorization: Bearer {access_token}
```

**请求参数**:
```json
{
  "username": "newuser",
  "password": "123456",
  "email": "newuser@example.com",
  "mobile": "13900139000",
  "nick_name": "新用户"
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": {
    "user_id": 2
  }
}
```

---

### 8. 更新用户

**接口地址**: `PUT /api/v1/users/{id}`

**接口描述**: 更新用户信息。需要认证，更新人从JWT中获取。

**路径参数**:
- `id` (integer, required): 用户ID

**请求头**:
```
Authorization: Bearer {access_token}
```

**请求参数**:
```json
{
  "email": "updated@example.com",
  "mobile": "13900139001",
  "nick_name": "更新后的昵称"
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": null
}
```

---

### 9. 删除用户

**接口地址**: `DELETE /api/v1/users/{id}`

**接口描述**: 删除用户（逻辑删除）。需要认证。

**路径参数**:
- `id` (integer, required): 用户ID

**请求头**:
```
Authorization: Bearer {access_token}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": null
}
```

---

### 10. 修改密码

**接口地址**: `PUT /api/v1/users/{id}/password`

**接口描述**: 修改用户密码。需要认证，需要提供旧密码和新密码。更新人从JWT中获取。

**路径参数**:
- `id` (integer, required): 用户ID

**请求头**:
```
Authorization: Bearer {access_token}
```

**请求参数**:
```json
{
  "old_password": "123456",
  "new_password": "newpass123"
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": null
}
```

---

### 11. 重置密码

**接口地址**: `PUT /api/v1/users/{id}/password/reset`

**接口描述**: 重置用户密码（管理员接口）。需要认证，不需要提供旧密码。更新人从JWT中获取。

**路径参数**:
- `id` (integer, required): 用户ID

**请求头**:
```
Authorization: Bearer {access_token}
```

**请求参数**:
```json
{
  "new_password": "newpass123"
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": null
}
```

---

### 12. 获取用户的租户列表

**接口地址**: `GET /api/v1/users/{id}/tenants`

**接口描述**: 获取指定用户所属的所有租户列表。需要认证。

**路径参数**:
- `id` (integer, required): 用户ID

**请求头**:
```
Authorization: Bearer {access_token}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "id": 1,
      "user_id": 1,
      "tenant_id": 0,
      "role_code": "member",
      "is_owner": 1,
      "status": 1,
      "join_time": "2024-01-01T00:00:00Z"
    }
  ]
}
```

---

### 13. 添加用户到租户

**接口地址**: `POST /api/v1/users/{id}/tenants`

**接口描述**: 将用户添加到指定租户。需要认证，创建人从JWT中获取。

**路径参数**:
- `id` (integer, required): 用户ID

**请求头**:
```
Authorization: Bearer {access_token}
```

**请求参数**:
```json
{
  "tenant_id": 1
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": null
}
```

---

### 14. 从租户移除用户

**接口地址**: `DELETE /api/v1/users/{id}/tenants`

**接口描述**: 从指定租户中移除用户。需要认证。

**路径参数**:
- `id` (integer, required): 用户ID

**请求头**:
```
Authorization: Bearer {access_token}
```

**请求参数**:
```json
{
  "tenant_id": 1
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": null
}
```

---

### 15. 设置默认租户

**接口地址**: `PUT /api/v1/users/{id}/tenants/default`

**接口描述**: 设置用户的默认租户。需要认证，更新人从JWT中获取。

**路径参数**:
- `id` (integer, required): 用户ID

**请求头**:
```
Authorization: Bearer {access_token}
```

**请求参数**:
```json
{
  "tenant_id": 1
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 200,
  "msg": "操作成功",
  "data": null
}
```

---

## 错误码说明

| 错误码 | 说明 |
|--------|------|
| 200 | 操作成功 |
| 4001 | 用户不存在 |
| 4002 | 密码错误 |
| 4003 | 用户已禁用 |
| 4004 | Token 无效 |
| 4005 | Token 已过期 |
| 4011 | 用户名已存在 |
| 4012 | 邮箱已存在 |
| 4015 | 手机号已存在 |
| 4016 | 验证码错误 |
| 4017 | 验证码已过期 |
| 5000 | 系统错误 |
| 5003 | 数据库错误 |
| 5004 | 业务错误 |

## 导入说明

### Apipost 格式

1. 打开 Apipost
2. 点击"导入" -> "OpenAPI"
3. 选择 `user_api_apipost_format.json` 文件
4. 导入后即可使用所有接口

### Postman 格式

1. 打开 Postman
2. 点击"Import"
3. 选择 `user_api.json` 文件
4. 导入后即可使用所有接口

## 注意事项

1. 所有需要认证的接口都需要在请求头中添加 `Authorization: Bearer {access_token}`
2. 验证码发送限制：180秒内不允许重复获取
3. 验证码有效期：5分钟
4. admin用户特殊验证码：`888888`（永久有效）
5. 注册时自动将用户添加到租户0
6. 所有 insert 和 update 操作都会自动记录创建人和更新人（从JWT中获取）

