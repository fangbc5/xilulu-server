// 用户模块 Service 层
// 负责用户相关的业务逻辑

use crate::config::PasswordConfig;
use crate::error::IdentityError;
// 使用模块重新导出的 Repository（对于 tenant 模块，使用重新导出）
use crate::modules::plan::model::entity::TenantSubscription;
use crate::modules::plan::repository::PlanRepo;
use crate::modules::tenant::{SystemTenant, Tenant};
// 对于 user 模块内部的类型，使用模块重新导出（通过 mod.rs 的重新导出）
use crate::modules::user::{
    RoleCode, TenantUserRel, User, UserRepo, UserRole, UserRoleRepo, UserTenantRelRepo,
};
use anyhow::Result;
use chrono::{Duration, Utc};
use sqlxplus::error::SqlxPlusError;
use sqlxplus::{Crud, DbPool, UpdateBuilder};
use std::sync::Arc;

/// 用户 Service
pub struct UserService {
    db_pool: Arc<DbPool>,
    password_config: PasswordConfig,
}

impl UserService {
    /// 创建新的 UserService
    pub fn new(db_pool: Arc<DbPool>, password_config: PasswordConfig) -> Self {
        Self {
            db_pool,
            password_config,
        }
    }

    /// 验证用户名/手机号/邮箱 + 密码
    pub async fn verify_password(
        &self,
        username: Option<&str>,
        password: Option<&str>,
        mobile: Option<&str>,
        email: Option<&str>,
        region: Option<&str>,
    ) -> Result<User> {
        // 根据登录方式查找用户并验证
        let user = match (username, mobile, email) {
            (Some(username_str), None, None) => {
                // 用户名密码登录
                if password.is_none() {
                    return Err(IdentityError::PasswordError("密码不能为空".to_string()).into());
                }
                let user =
                    UserRepo::find_by_username(self.db_pool.mysql_pool(), username_str).await?;
                self.verify_password_and_handle_error(&user, password.unwrap())
                    .await?;
                user
            }
            (None, Some(mobile_str), None) => {
                // 手机号登录：密码登录或验证码登录（验证码已由上游服务校验）
                let user = UserRepo::find_by_mobile(self.db_pool.mysql_pool(), mobile_str).await?;
                if let Some(pwd) = password {
                    self.verify_password_and_handle_error(&user, pwd).await?;
                }
                user
            }
            (None, None, Some(email_str)) => {
                // 邮箱登录：密码登录或验证码登录（验证码已由上游服务校验）
                let user = UserRepo::find_by_email(self.db_pool.mysql_pool(), email_str).await?;
                if let Some(pwd) = password {
                    self.verify_password_and_handle_error(&user, pwd).await?;
                }
                user
            }
            _ => {
                return Err(
                    IdentityError::BusinessError("请提供用户名、手机号或邮箱".to_string()).into(),
                );
            }
        };

        // 检查用户状态
        if user.state != Some(1) {
            return Err(IdentityError::UserDisabled.into());
        }

        // 如果传了 region，并且用户本身设置了 region，进行校验
        if let Some(req_region) = region {
            if let Some(user_region) = &user.region {
                if !user_region.is_empty() && user_region != req_region {
                    return Err(IdentityError::BusinessError("账号或地区注册信息不符".to_string()).into());
                }
            }
        }

        // 更新登录时间和密码错误次数（重置为0）
        let updated_user = User {
            id: user.id,
            password_error_num: Some(0),
            password_error_last_time: None,
            last_login_time: Some(Utc::now()),
            last_opt_time: Some(Utc::now()),
            update_time: Some(Utc::now()),
            ..Default::default()
        };

        updated_user
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    /// 验证密码并处理错误（更新密码错误次数）
    async fn verify_password_and_handle_error(&self, user: &User, password: &str) -> Result<()> {
        let password_valid =
            Self::verify_password_static(user, password, &self.password_config).is_ok();

        if !password_valid {
            // 更新密码错误次数
            let updated_user = User {
                id: user.id,
                password_error_num: Some(user.password_error_num.unwrap_or(0) + 1),
                password_error_last_time: Some(Utc::now()),
                update_time: Some(Utc::now()),
                ..Default::default()
            };
            let _ = updated_user.update(self.db_pool.mysql_pool()).await;
            return Err(IdentityError::PasswordError("密码错误".to_string()).into());
        }

        Ok(())
    }

    /// 获取用户信息（只读操作，不需要事务）
    pub async fn get_user_info(&self, user_id: i64) -> Result<User> {
        let user = User::find_by_id(self.db_pool.mysql_pool(), user_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        user.ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))
    }

    /// 批量获取用户信息
    pub async fn get_users_by_ids(&self, user_ids: &[i64]) -> Result<Vec<User>> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }
        User::find_by_ids(self.db_pool.mysql_pool(), user_ids.to_vec())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()).into())
    }

    /// 根据邮箱精确查找用户
    pub async fn find_by_email(&self, email: &str) -> Result<User> {
        UserRepo::find_by_email(self.db_pool.mysql_pool(), email).await
    }

    /// 根据手机号精确查找用户
    pub async fn find_by_mobile(&self, mobile: &str) -> Result<User> {
        UserRepo::find_by_mobile(self.db_pool.mysql_pool(), mobile).await
    }

    /// 根据用户名精确查找用户
    pub async fn find_by_username(&self, username: &str) -> Result<User> {
        UserRepo::find_by_username(self.db_pool.mysql_pool(), username).await
    }

    /// 获取用户列表（分页）
    pub async fn list_users(
        &self,
        page: u32,
        page_size: u32,
        search_key: Option<&str>,
        tenant_id: Option<i64>,
    ) -> Result<(Vec<User>, i64)> {
        use sqlxplus::QueryBuilder;

        let mut builder = QueryBuilder::new("SELECT * FROM `user`");

        // 如果提供了 tenant_id，先查询该租户下的用户ID列表
        if let Some(tid) = tenant_id {
            let tenant_users = UserTenantRelRepo::find_by_tenant_id(self.db_pool.mysql_pool(), tid)
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

            let user_ids: Vec<i64> = tenant_users.into_iter().map(|rel| rel.user_id).collect();

            if user_ids.is_empty() {
                // 如果租户下没有用户，返回空结果
                return Ok((Vec::new(), 0));
            }

            // 使用 IN 查询过滤用户ID
            builder = builder.and_in("id", user_ids);
        }

        // 如果提供了搜索关键词，添加搜索条件
        if let Some(key) = search_key {
            if !key.is_empty() {
                builder = builder.and_group(|mut builder_and| {
                    builder_and = builder_and.or_like("username", key);
                    builder_and = builder_and.or_like("email", key);
                    builder_and = builder_and.or_like("mobile", key);
                    builder_and = builder_and.or_like("nick_name", key);
                    builder_and
                });
            }
        }

        let result = User::paginate(self.db_pool.mysql_pool(), builder, page, page_size)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok((result.items, result.total as i64))
    }

    /// 获取用户总数（只读操作，不需要事务）
    pub async fn get_user_count(&self) -> Result<i64> {
        let builder = sqlxplus::QueryBuilder::new("");
        let count = User::count(self.db_pool.mysql_pool(), builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        Ok(count as i64)
    }

    /// 获取活跃用户数（只读操作，不需要事务）
    /// 活跃用户定义：最近 days 天内有登录的用户（last_login_time 不为空且在指定天数内）
    /// 默认统计最近30天内的活跃用户
    pub async fn get_active_user_count(&self, days: Option<u32>) -> Result<i64> {
        let days = days.unwrap_or(30); // 默认30天

        // 计算时间阈值
        let threshold = Utc::now() - chrono::Duration::days(days as i64);

        // 使用原始 SQL 查询，因为 sqlxplus QueryBuilder 可能不支持 and_gte
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM `user` WHERE `is_del` = 0 AND `last_login_time` IS NOT NULL AND `last_login_time` >= ?"
        )
        .bind(threshold)
        .fetch_one(self.db_pool.mysql_pool())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(count.0)
    }

    /// 用户注册（公开接口）
    /// 创建用户、创建租户、绑定用户为owner、绑定Free套餐
    /// 仅支持用户名密码注册或手机号/邮箱+密码注册（验证码校验由上游服务负责）
    pub async fn register_user(
        &self,
        username: Option<&str>,
        password: Option<&str>,
        email: Option<&str>,
        mobile: Option<&str>,
        nick_name: Option<&str>,
        avatar: Option<&str>,
        region: Option<&str>,
    ) -> Result<i64> {
        // 判断注册方式：用户名密码 或 手机号/邮箱+密码
        let (final_username, final_password, final_email, final_mobile) =
            if let Some(username_str) = username {
                // 用户名密码注册
                if password.is_none() {
                    return Err(IdentityError::PasswordError("密码不能为空".to_string()).into());
                }
                let password_str = password.unwrap();
                // 验证密码长度
                if password_str.len() < self.password_config.min_length {
                    return Err(
                        IdentityError::PasswordTooShort(self.password_config.min_length).into(),
                    );
                }
                // 检查用户名是否已存在
                if UserRepo::exists_by_username(self.db_pool.mysql_pool(), username_str).await? {
                    return Err(IdentityError::UsernameExists.into());
                }
                // 检查邮箱是否已存在
                if let Some(email_str) = email {
                    if UserRepo::exists_by_email(self.db_pool.mysql_pool(), email_str).await? {
                        return Err(IdentityError::EmailExists.into());
                    }
                }
                // 检查手机号是否已存在
                if let Some(mobile_str) = mobile {
                    if UserRepo::exists_by_mobile(self.db_pool.mysql_pool(), mobile_str).await? {
                        return Err(IdentityError::MobileExists.into());
                    }
                }
                (
                    Some(username_str.to_string()),
                    Some(password_str.to_string()),
                    email.map(|s| s.to_string()),
                    mobile.map(|s| s.to_string()),
                )
            } else if let Some(mobile_str) = mobile {
                // 手机号+密码注册（验证码校验由上游服务负责）
                if password.is_none() {
                    return Err(IdentityError::PasswordError("密码不能为空".to_string()).into());
                }
                let password_str = password.unwrap();
                // 验证密码长度
                if password_str.len() < self.password_config.min_length {
                    return Err(
                        IdentityError::PasswordTooShort(self.password_config.min_length).into(),
                    );
                }
                // 检查手机号是否已存在
                if UserRepo::exists_by_mobile(self.db_pool.mysql_pool(), mobile_str).await? {
                    return Err(IdentityError::MobileExists.into());
                }
                (
                    None,
                    Some(password_str.to_string()),
                    None,
                    Some(mobile_str.to_string()),
                )
            } else if let Some(email_str) = email {
                // 邮箱+密码注册（验证码校验由上游服务负责）
                if password.is_none() {
                    return Err(IdentityError::PasswordError("密码不能为空".to_string()).into());
                }
                let password_str = password.unwrap();
                // 验证密码长度
                if password_str.len() < self.password_config.min_length {
                    return Err(
                        IdentityError::PasswordTooShort(self.password_config.min_length).into(),
                    );
                }
                // 检查邮箱是否已存在
                if UserRepo::exists_by_email(self.db_pool.mysql_pool(), email_str).await? {
                    return Err(IdentityError::EmailExists.into());
                }
                (
                    None,
                    Some(password_str.to_string()),
                    Some(email_str.to_string()),
                    None,
                )
            } else {
                return Err(
                    IdentityError::BusinessError("请提供用户名、手机号或邮箱".to_string()).into(),
                );
            };

        // 加密密码（如果有）
        // 非用户名密码注册时，不生成密码，直接返回None，提高性能
        let (hashed_password, salt) = if let Some(password_str) = &final_password {
            let (hash, salt) = self.hash_password(password_str)?;
            (Some(hash), salt)
        } else {
            // 验证码注册时，不生成密码（用户后续需要设置密码）
            // 避免不必要的密码哈希计算，提高性能
            (None, None)
        };

        // 在事务外查找Free套餐（套餐数据是静态的，不会改变）
        let free_plan = PlanRepo::find_by_name(self.db_pool.mysql_pool(), "Free").await?;

        // 验证套餐类型为personal
        if free_plan.r#type != "personal" {
            return Err(IdentityError::BusinessError(format!(
                "Free套餐类型不正确，期望personal，实际{}",
                free_plan.r#type
            ))
            .into());
        }

        let plan_id = free_plan
            .id
            .ok_or_else(|| IdentityError::BusinessError("Free套餐ID不存在".to_string()))?;

        // 使用事务确保用户创建、租户创建、租户关系创建和订阅创建的原子性
        let username_clone = final_username.clone();
        let email_clone = final_email.clone();
        let mobile_clone = final_mobile.clone();
        let nick_name = nick_name.map(|s| s.to_string());
        let avatar_clone = avatar.map(|s| s.to_string());
        let region_clone = region.map(|s| s.to_string());

        // 生成租户名称（优先使用用户名，否则使用邮箱或手机号）
        let tenant_name = if let Some(ref username) = final_username {
            username.clone()
        } else if let Some(ref email) = final_email {
            email.clone()
        } else if let Some(ref mobile) = final_mobile {
            mobile.clone()
        } else {
            return Err(IdentityError::BusinessError("无法生成租户名称".to_string()).into());
        };

        // 生成联系人名称（优先使用昵称，否则使用用户名、邮箱或手机号）
        let contact_name = nick_name
            .as_ref()
            .or(final_username.as_ref())
            .or(final_email.as_ref())
            .or(final_mobile.as_ref())
            .ok_or_else(|| IdentityError::BusinessError("无法生成联系人名称".to_string()))?
            .clone();

        let user_id = sqlxplus::with_transaction(self.db_pool.as_ref(), |tx| {
            Box::pin(async move {
                // 1. 创建用户实体
                let mut user = User::default();
                user.username = username_clone.clone();
                user.password = hashed_password.clone();
                user.salt = salt.clone();
                user.email = email_clone.clone();
                user.mobile = mobile_clone.clone();
                user.nick_name = nick_name.clone();
                user.avatar = avatar_clone;
                user.region = region_clone;
                user.state = Some(1); // 默认启用
                user.system_type = Some(1);
                user.user_type = Some(3);
                user.password_error_num = Some(0);
                // 注册时没有创建人
                user.create_by = None;
                user.create_time = Some(Utc::now());
                user.update_time = Some(Utc::now());

                // 保存用户
                let user_id = user.insert(tx.as_mysql_executor()).await?;

                // 2. 创建租户
                let now = Utc::now();
                // 免费套餐设置为10年后过期
                let expire_time = now + Duration::days(365 * 10);

                let mut tenant = Tenant::default();
                tenant.name = tenant_name.clone();
                tenant.pid = SystemTenant::PersonalDefault.id(); // 继承系统默认个人租户权限
                tenant.contact_user_id = Some(user_id);
                tenant.contact_name = contact_name.clone();
                tenant.contact_mobile = mobile_clone.clone();
                tenant.package_id = plan_id; // 使用套餐ID
                tenant.expire_time = expire_time;
                tenant.account_count = 1; // 个人版默认1个账号
                tenant.status = Some(0); // 正常状态
                tenant.create_by = Some(user_id); // 注册时没有创建人
                tenant.create_time = Some(now);

                // 保存租户
                let tenant_id = tenant.insert(tx.as_mysql_executor()).await?;

                // 3. 将用户添加到租户（is_owner=1，表示owner）
                let mut rel = TenantUserRel::default();
                rel.user_id = user_id;
                rel.tenant_id = tenant_id;
                rel.is_owner = Some(1); // 设置为owner
                rel.status = Some(1); // 默认启用
                rel.join_time = Some(now);
                rel.created_by = None; // 注册时没有创建人
                rel.created_time = Some(now);
                rel.updated_time = Some(now);

                // 插入用户租户关系
                rel.insert(tx.as_mysql_executor()).await?;

                // 3.1 查找继承的系统默认 Member 角色，并关联到 UserRole 表中
                let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `role`")
                    .and_eq("code", RoleCode::Member.to_string())
                    .and_eq("tenant_id", SystemTenant::PersonalDefault.id());
                
                let default_member_role = crate::modules::auth::Role::find_one(tx.as_mysql_executor(), builder)
                    .await?
                    .ok_or_else(|| sqlxplus::error::SqlxPlusError::Other("系统租户内置角色不存在".to_string()))?;
                
                // 插入 user_role
                let mut user_role = UserRole::default();
                user_role.user_id = user_id;
                user_role.role_id = default_member_role.id.unwrap_or_default();
                user_role.tenant_id = tenant_id; // 关联到新建的 tenant_id 下
                user_role.created_by = Some(user_id);
                user_role.created_at = Some(now);
                user_role.insert(tx.as_mysql_executor()).await?;

                // 4. 不再单独创建租户订阅，新挂载的租户直接通过继承 pid 使用父租户级别的套餐功能限制

                Ok::<i64, SqlxPlusError>(user_id)
            })
        })
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(user_id)
    }

    /// 创建用户（管理员接口）
    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
        email: Option<&str>,
        mobile: Option<&str>,
        nick_name: Option<&str>,
        create_by: Option<i64>,
    ) -> Result<i64> {
        // 验证密码长度
        if password.len() < self.password_config.min_length {
            return Err(IdentityError::PasswordTooShort(self.password_config.min_length).into());
        }

        // 加密密码
        let (hashed_password, salt) = self.hash_password(password)?;

        // 检查用户名是否已存在
        if UserRepo::exists_by_username(self.db_pool.mysql_pool(), username).await? {
            return Err(IdentityError::UsernameExists.into());
        }

        // 检查邮箱是否已存在
        if let Some(email_str) = email {
            if UserRepo::exists_by_email(self.db_pool.mysql_pool(), email_str).await? {
                return Err(IdentityError::EmailExists.into());
            }
        }

        // 创建用户实体
        let mut user = User::default();
        user.username = Some(username.to_string());
        user.password = Some(hashed_password);
        user.salt = salt;
        user.email = email.map(|s| s.to_string());
        user.mobile = mobile.map(|s| s.to_string());
        user.nick_name = nick_name.map(|s| s.to_string());
        user.state = Some(1); // 默认启用
        user.system_type = Some(1);
        user.user_type = Some(3);
        user.password_error_num = Some(0);
        user.create_by = create_by;
        user.create_time = Some(Utc::now());
        user.update_time = Some(Utc::now());

        // 保存用户
        let user_id = user
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(user_id)
    }

    /// 更新用户
    pub async fn update_user(
        &self,
        user_id: i64,
        username: Option<&str>,
        email: Option<&str>,
        mobile: Option<&str>,
        nick_name: Option<&str>,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 如果更新邮箱，检查是否已存在
        if let Some(email_str) = email {
            let existing_user = User::find_by_id(self.db_pool.mysql_pool(), user_id)
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
                .ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))?;
            if email_str != existing_user.email.as_deref().unwrap_or("") {
                if UserRepo::exists_by_email(self.db_pool.mysql_pool(), email_str).await? {
                    return Err(IdentityError::EmailExists.into());
                }
            }
        }

        // 只更新需要更新的字段
        let updated_user = User {
            id: Some(user_id),
            username: username.map(|s| s.to_string()),
            email: email.map(|s| s.to_string()),
            mobile: mobile.map(|s| s.to_string()),
            nick_name: nick_name.map(|s| s.to_string()),
            update_by,
            update_time: Some(Utc::now()),
            ..Default::default()
        };

        updated_user
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除用户（逻辑删除）
    pub async fn delete_user(&self, user_id: i64) -> Result<()> {
        User::delete_by_id(self.db_pool.mysql_pool(), user_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 修改密码
    pub async fn change_password(
        &self,
        user_id: i64,
        old_password: &str,
        new_password: &str,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 验证密码长度
        if new_password.len() < self.password_config.min_length {
            return Err(IdentityError::PasswordTooShort(self.password_config.min_length).into());
        }

        // 加密新密码
        let (hashed_password, salt) = self.hash_password(new_password)?;

        // 获取用户并验证旧密码
        let user = User::find_by_id(self.db_pool.mysql_pool(), user_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))?;
        Self::verify_password_static(&user, old_password, &self.password_config)?;

        // 只更新密码相关字段
        let updated_user = User {
            id: Some(user_id),
            password: Some(hashed_password),
            salt,
            password_error_num: Some(0),
            password_error_last_time: None,
            update_by,
            update_time: Some(Utc::now()),
            ..Default::default()
        };

        updated_user
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 重置密码
    pub async fn reset_password(
        &self,
        user_id: i64,
        new_password: &str,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 验证密码长度
        if new_password.len() < self.password_config.min_length {
            return Err(IdentityError::PasswordTooShort(self.password_config.min_length).into());
        }

        // 加密新密码
        let (hashed_password, salt) = self.hash_password(new_password)?;

        // 只更新密码相关字段
        let updated_user = User {
            id: Some(user_id),
            password: Some(hashed_password),
            salt,
            password_error_num: Some(0),
            password_error_last_time: None,
            update_by,
            update_time: Some(Utc::now()),
            ..Default::default()
        };

        updated_user
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 静态方法：验证用户密码（用于事务内部）
    fn verify_password_static(
        user: &User,
        password: &str,
        _password_config: &PasswordConfig,
    ) -> Result<()> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let stored_password = user.password.as_deref().ok_or_else(|| -> anyhow::Error {
            IdentityError::PasswordError("用户密码未设置".to_string()).into()
        })?;

        // 如果存储的密码为空字符串，直接返回错误
        if stored_password.is_empty() {
            return Err(IdentityError::PasswordError("用户密码未设置".to_string()).into());
        }

        // 解析存储的密码哈希
        let parsed_hash = PasswordHash::new(stored_password)
            .map_err(|e| IdentityError::PasswordError(format!("密码哈希解析失败: {}", e)))?;

        // 使用 argon2 验证密码
        let argon2 = Argon2::default();
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| IdentityError::PasswordError("密码错误".to_string()))?;

        Ok(())
    }

    /// 加密密码（使用 argon2）
    fn hash_password(&self, password: &str) -> Result<(String, Option<String>)> {
        use argon2::{
            password_hash::{rand_core::OsRng, SaltString},
            Argon2, PasswordHasher,
        };

        // 生成随机 salt
        let salt = SaltString::generate(&mut OsRng);

        // 使用 argon2 加密密码
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| IdentityError::PasswordEncryptError(format!("密码加密失败: {}", e)))?;

        // argon2 的 hash 已经包含了 salt，不需要单独存储
        Ok((password_hash.to_string(), None))
    }
}

/// 用户租户关系 Service
pub struct UserTenantService {
    db_pool: Arc<DbPool>,
}

impl UserTenantService {
    /// 创建新的 UserTenantService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 添加用户到租户
    pub async fn add_user_to_tenant(
        &self,
        user_id: i64,
        tenant_id: i64,
        is_default: bool,
        create_by: Option<i64>,
    ) -> Result<()> {
        // 验证用户存在
        User::find_by_id(self.db_pool.mysql_pool(), user_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))?;

        // 验证租户存在并获取 pid
        let tenant = Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::TenantNotFound))?;

        // 检查关系是否已存在
        if let Some(_) = UserTenantRelRepo::find_by_user_and_tenant(
            self.db_pool.mysql_pool(),
            user_id,
            tenant_id,
        )
        .await?
        {
            return Err(IdentityError::UserTenantRelExists.into());
        }

        // 如果设置为默认租户（通过 is_owner 字段表示），需要先取消其他默认租户
        if is_default {
            self.unset_default_tenants_internal(user_id, create_by)
                .await?;
        }

        // 开启事务插入用户与租户关系及内置角色关联
        sqlxplus::with_transaction(self.db_pool.as_ref(), |tx| {
            Box::pin(async move {
                // 1. 创建关系
                let mut rel = TenantUserRel::default();
                rel.user_id = user_id;
                rel.tenant_id = tenant_id;
                rel.is_owner = if is_default { Some(1) } else { Some(0) };
                rel.status = Some(1); // 默认启用
                rel.join_time = Some(Utc::now());
                rel.created_by = create_by;
                rel.created_time = Some(Utc::now());
                rel.updated_time = Some(Utc::now());

                rel.insert(tx.as_mysql_executor()).await?;

                // 2. 插入 built-in 继承的角色记录
                let code_str = if is_default { RoleCode::Owner.to_string() } else { RoleCode::Member.to_string() };
                let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `role`")
                    .and_eq("code", code_str)
                    .and_eq("tenant_id", tenant.pid);

                let default_role = crate::modules::auth::Role::find_one(tx.as_mysql_executor(), builder)
                    .await?
                    .ok_or_else(|| sqlxplus::error::SqlxPlusError::Other("目标角色不存在".to_string()))?;

                let mut user_role = UserRole::default();
                user_role.user_id = user_id;
                user_role.role_id = default_role.id.unwrap_or_default();
                user_role.tenant_id = tenant_id;
                user_role.created_by = create_by;
                user_role.created_at = Some(Utc::now());
                user_role.insert(tx.as_mysql_executor()).await?;

                Ok::<(), SqlxPlusError>(())
            })
        })
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 从租户移除用户
    pub async fn remove_user_from_tenant(&self, user_id: i64, tenant_id: i64) -> Result<()> {
        // 先查找关系记录获取 ID
        let rel = UserTenantRelRepo::find_by_user_and_tenant(
            self.db_pool.mysql_pool(),
            user_id,
            tenant_id,
        )
        .await?;

        if let Some(rel) = rel {
            if let Some(id) = rel.id {
                // 使用 sqlxplus 的 delete_by_id 方法删除
                TenantUserRel::delete_by_id(self.db_pool.mysql_pool(), id)
                    .await
                    .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// 设置默认租户
    pub async fn set_default_tenant(
        &self,
        user_id: i64,
        tenant_id: i64,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 验证用户存在
        User::find_by_id(self.db_pool.mysql_pool(), user_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))?;

        // 验证租户存在
        Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::TenantNotFound))?;

        // 检查关系是否存在
        let mut rel = match UserTenantRelRepo::find_by_user_and_tenant(
            self.db_pool.mysql_pool(),
            user_id,
            tenant_id,
        )
        .await?
        {
            Some(rel) => rel,
            None => return Err(IdentityError::UserTenantRelNotFound.into()),
        };

        // 取消其他默认租户
        self.unset_default_tenants_internal(user_id, update_by)
            .await?;

        // 设置当前租户为默认（通过 is_owner 字段表示）
        rel.is_owner = Some(1);
        // TenantUserRel 没有 update_by 字段，只有 created_by
        rel.updated_time = Some(Utc::now());

        rel.update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 获取用户的租户列表（只读操作，不需要事务）
    pub async fn get_user_tenants(&self, user_id: i64) -> Result<Vec<TenantUserRel>> {
        UserTenantRelRepo::find_by_user_id(self.db_pool.mysql_pool(), user_id).await
    }

    /// 取消用户的所有默认租户（内部方法）
    async fn unset_default_tenants_internal(
        &self,
        user_id: i64,
        _update_by: Option<i64>,
    ) -> Result<()> {
        let mut update_model = TenantUserRel::default();
        update_model.user_id = user_id;
        update_model.is_owner = Some(0);
        update_model.updated_time = Some(Utc::now());
        UpdateBuilder::new(update_model)
            .fields(&["is_owner", "updated_time"])
            .condition(|c| c.and_eq("user_id", user_id).and_eq("is_owner", 1))
            .execute(self.db_pool.mysql_pool())
            .await?;
        Ok(())
    }
}

/// 用户角色 Service
pub struct UserRoleService {
    db_pool: Arc<DbPool>,
}

impl UserRoleService {
    /// 创建新的 UserRoleService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 获取用户在租户下的角色列表
    pub async fn get_user_roles(&self, user_id: i64, tenant_id: i64) -> Result<Vec<UserRole>> {
        UserRoleRepo::find_by_user_and_tenant(self.db_pool.mysql_pool(), user_id, tenant_id).await
    }

    /// 为用户分配角色
    pub async fn assign_role_to_user(
        &self,
        user_id: i64,
        role_id: i64,
        tenant_id: i64,
        created_by: Option<i64>,
    ) -> Result<()> {
        // 验证用户存在
        User::find_by_id(self.db_pool.mysql_pool(), user_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))?;

        // 验证租户存在
        Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::TenantNotFound))?;

        // 验证角色存在（从 auth 模块获取）
        use crate::modules::auth::Role;
        Role::find_by_id(self.db_pool.mysql_pool(), role_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                anyhow::Error::from(IdentityError::DatabaseError("角色不存在".to_string()))
            })?;

        // 检查用户是否属于该租户
        UserTenantRelRepo::find_by_user_and_tenant(self.db_pool.mysql_pool(), user_id, tenant_id)
            .await?
            .ok_or_else(|| {
                anyhow::Error::from(IdentityError::DatabaseError("用户不属于该租户".to_string()))
            })?;

        // 检查角色是否已分配
        if let Some(_) = UserRoleRepo::find_by_user_role_and_tenant(
            self.db_pool.mysql_pool(),
            user_id,
            role_id,
            tenant_id,
        )
        .await?
        {
            return Err(IdentityError::DatabaseError("角色已分配给该用户".to_string()).into());
        }

        // 获取角色信息以获取 role_code
        let role = Role::find_by_id(self.db_pool.mysql_pool(), role_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                anyhow::Error::from(IdentityError::DatabaseError("角色不存在".to_string()))
            })?;

        // 创建用户角色关系
        let mut user_role = UserRole::default();
        user_role.user_id = user_id;
        user_role.role_id = role_id;
        user_role.role_code = role.code.clone();
        user_role.tenant_id = tenant_id;
        user_role.created_by = created_by;
        user_role.created_at = Some(Utc::now());

        user_role
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 批量为用户分配角色
    pub async fn batch_assign_roles_to_user(
        &self,
        user_id: i64,
        role_ids: Vec<i64>,
        tenant_id: i64,
        created_by: Option<i64>,
    ) -> Result<()> {
        // 验证用户存在
        User::find_by_id(self.db_pool.mysql_pool(), user_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))?;

        // 验证租户存在
        Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::TenantNotFound))?;

        // 检查用户是否属于该租户
        UserTenantRelRepo::find_by_user_and_tenant(self.db_pool.mysql_pool(), user_id, tenant_id)
            .await?
            .ok_or_else(|| {
                anyhow::Error::from(IdentityError::DatabaseError("用户不属于该租户".to_string()))
            })?;

        // 检查哪些角色已分配
        let existing_roles =
            UserRoleRepo::find_by_user_and_tenant(self.db_pool.mysql_pool(), user_id, tenant_id)
                .await?;
        let existing_role_ids: std::collections::HashSet<i64> =
            existing_roles.into_iter().map(|ur| ur.role_id).collect();

        // 过滤掉已分配的角色，只分配新的
        let new_role_ids: Vec<i64> = role_ids
            .into_iter()
            .filter(|role_id| !existing_role_ids.contains(role_id))
            .collect();

        if new_role_ids.is_empty() {
            return Err(
                IdentityError::DatabaseError("所有角色都已分配给该用户".to_string()).into(),
            );
        }

        // 验证所有角色存在并获取角色信息
        use crate::modules::auth::Role;
        let mut role_map = std::collections::HashMap::new();
        for role_id in &new_role_ids {
            let role = Role::find_by_id(self.db_pool.mysql_pool(), *role_id)
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
                .ok_or_else(|| {
                    anyhow::Error::from(IdentityError::DatabaseError(format!(
                        "角色 {} 不存在",
                        role_id
                    )))
                })?;
            if let Some(id) = role.id {
                role_map.insert(id, role);
            }
        }

        // 批量创建用户角色关系
        let now = Utc::now();
        for role_id in &new_role_ids {
            let role = role_map.get(role_id).ok_or_else(|| {
                anyhow::Error::from(IdentityError::DatabaseError(format!(
                    "角色 {} 信息不存在",
                    role_id
                )))
            })?;

            let mut user_role = UserRole::default();
            user_role.user_id = user_id;
            user_role.role_id = *role_id;
            user_role.role_code = role.code.clone();
            user_role.tenant_id = tenant_id;
            user_role.created_by = created_by;
            user_role.created_at = Some(now);

            user_role
                .insert(self.db_pool.mysql_pool())
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    /// 移除用户角色
    pub async fn remove_role_from_user(
        &self,
        user_id: i64,
        role_id: i64,
        tenant_id: i64,
    ) -> Result<()> {
        // 查找用户角色关系
        let user_role = UserRoleRepo::find_by_user_role_and_tenant(
            self.db_pool.mysql_pool(),
            user_id,
            role_id,
            tenant_id,
        )
        .await?
        .ok_or_else(|| {
            anyhow::Error::from(IdentityError::DatabaseError(
                "用户角色关系不存在".to_string(),
            ))
        })?;

        if let Some(id) = user_role.id {
            UserRole::delete_by_id(self.db_pool.mysql_pool(), id)
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }
}
