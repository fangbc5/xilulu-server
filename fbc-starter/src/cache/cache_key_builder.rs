/// Cache Key Builder Trait
///
/// Redis key 命名规范：
/// 【推荐】Redis key 命名需具有可读性以及可管理性，不该使用含义不清的 key 以及特别长的 key 名；
/// 【强制】以英文字母开头，命名中只能出现小写字母、数字、英文点号(.)和英文半角冒号(:)；
/// 【强制】不要包含特殊字符，如下划线、空格、换行、单双引号以及其他转义字符；
///
/// 命名规范：
/// 【强制】命名规范：[前缀:][租户编码:][服务模块名:]业务类型[:业务字段][:value类型][:业务值]
///
/// 0）前缀：   可选。用来区分不同项目，不同环境。
/// 1）租户ID： 可选。用来区分不同租户数据缓存。
/// 2）服务模块名：可选。用来区分不同服务或功能模块的缓存。
/// 3）业务类型： 必填。用来区分不同业务类型的数据缓存。通常设置为表名。
/// 4）业务字段： 可选。用来区分业务值是哪个字段。通常设置为字段名。
/// 5）value类型：可选。用来区分 value 类型。
/// 6）业务值：   可选。用来区分同一业务类型的不同行的数据缓存。
///
/// 示例：
/// ```text
/// 0000:authority:user.activity:id.id:1:3:number => [1,2,3,4]
/// 表示：租户(0000)，权限服务(authority)中用户表(user)的用户id为1，活动表(activity)的活动id为3
///
/// 0000:authority:user:id:1:obj => {"id":1, "name":"张三"}
/// 表示：租户(0000)，权限服务(authority)中用户表(user)的用户id为1的数据
/// ```
use super::cache_key::{CacheHashKey, CacheKey, ValueType};
use std::time::Duration;

pub trait CacheKeyBuilder {
    /// 缓存前缀，用于区分项目、环境等
    fn get_prefix(&self) -> Option<&str> {
        None
    }

    /// 租户 ID，用于区分租户
    /// 非租户模式返回 None
    fn get_tenant(&self) -> Option<&str> {
        None
    }

    /// 设置租户 ID（默认不做处理，子类可重写）
    fn set_tenant_id(&mut self, _tenant_id: u64) {}

    /// 服务模块名，用于区分后端服务、前端模块等
    fn get_modular(&self) -> Option<&str> {
        None
    }

    /// key 的业务类型，用于区分表（必填）
    fn get_table(&self) -> &str;

    /// key 的字段名，用于区分字段
    fn get_field(&self) -> Option<&str> {
        None
    }

    /// 缓存的 value 存储的类型
    fn get_value_type(&self) -> ValueType {
        ValueType::Obj
    }

    /// 缓存自动过期时间
    fn get_expire(&self) -> Option<Duration> {
        None
    }

    /// 获取通配符模式
    fn get_pattern(&self) -> String {
        format!("*:{}:*", self.get_table())
    }

    /// 构建通用 KV 模式的 cache key
    /// 兼容 redis 和 caffeine
    ///
    /// # 参数
    /// - `uniques`: 动态参数（业务值）
    fn key(&self, uniques: &[&dyn ToString]) -> CacheKey {
        let key = self.build_key(uniques);
        assert!(!key.is_empty(), "key 不能为空");
        CacheKey::new(key, self.get_expire())
    }

    /// 构建 Redis 类型的 hash cache key（带 field）
    ///
    /// # 参数
    /// - `field`: hash field
    /// - `uniques`: 动态参数
    fn hash_field_key(&self, field: &dyn ToString, uniques: &[&dyn ToString]) -> CacheHashKey {
        let key = self.build_key(uniques);
        assert!(!key.is_empty(), "key 不能为空");
        CacheHashKey::new(key, Some(field.to_string()), self.get_expire())
    }

    /// 构建 Redis 类型的 hash cache key（无 field）
    ///
    /// # 参数
    /// - `uniques`: 动态参数
    fn hash_key(&self, uniques: &[&dyn ToString]) -> CacheHashKey {
        let key = self.build_key(uniques);
        assert!(!key.is_empty(), "key 不能为空");
        CacheHashKey::new(key, None, self.get_expire())
    }

    /// 根据动态参数拼接 key
    ///
    /// key 命名规范：[前缀:][租户ID:][服务模块名:]业务类型[:业务字段][:value类型][:业务值]
    fn build_key(&self, uniques: &[&dyn ToString]) -> String {
        let mut parts = Vec::new();

        // 前缀
        if let Some(prefix) = self.get_prefix() {
            if !prefix.is_empty() {
                parts.push(prefix.to_string());
            }
        }

        // 租户编码
        if let Some(tenant) = self.get_tenant() {
            if !tenant.is_empty() {
                parts.push(tenant.to_string());
            }
        }

        // 服务模块名
        if let Some(modular) = self.get_modular() {
            if !modular.is_empty() {
                parts.push(modular.to_string());
            }
        }

        // 业务类型（必填）
        let table = self.get_table();
        assert!(!table.is_empty(), "缓存业务类型不能为空");
        parts.push(table.to_string());

        // 业务字段
        if let Some(field) = self.get_field() {
            if !field.is_empty() {
                parts.push(field.to_string());
            }
        }

        // value 类型
        parts.push(self.get_value_type().as_str().to_string());

        // 业务值
        for unique in uniques {
            let value = unique.to_string();
            if !value.is_empty() {
                parts.push(value);
            }
        }

        parts.join(":")
    }
}

/// 简单的缓存键构建器实现
///
/// 用于快速构建缓存键，无需实现完整的 trait
#[derive(Debug, Clone)]
pub struct SimpleCacheKeyBuilder {
    pub prefix: Option<String>,
    pub tenant: Option<String>,
    pub modular: Option<String>,
    pub table: String,
    pub field: Option<String>,
    pub value_type: ValueType,
    pub expire: Option<Duration>,
}

impl SimpleCacheKeyBuilder {
    /// 创建新的缓存键构建器
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            prefix: None,
            tenant: None,
            modular: None,
            table: table.into(),
            field: None,
            value_type: ValueType::Obj,
            expire: None,
        }
    }

    /// 设置前缀
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// 设置租户
    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = Some(tenant.into());
        self
    }

    /// 设置模块名
    pub fn with_modular(mut self, modular: impl Into<String>) -> Self {
        self.modular = Some(modular.into());
        self
    }

    /// 设置字段名
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// 设置 value 类型
    pub fn with_value_type(mut self, value_type: ValueType) -> Self {
        self.value_type = value_type;
        self
    }

    /// 设置过期时间
    pub fn with_expire(mut self, expire: Duration) -> Self {
        self.expire = Some(expire);
        self
    }
}

impl CacheKeyBuilder for SimpleCacheKeyBuilder {
    fn get_prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    fn get_tenant(&self) -> Option<&str> {
        self.tenant.as_deref()
    }

    fn get_modular(&self) -> Option<&str> {
        self.modular.as_deref()
    }

    fn get_table(&self) -> &str {
        &self.table
    }

    fn get_field(&self) -> Option<&str> {
        self.field.as_deref()
    }

    fn get_value_type(&self) -> ValueType {
        self.value_type
    }

    fn get_expire(&self) -> Option<Duration> {
        self.expire
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_cache_key_builder() {
        let builder = SimpleCacheKeyBuilder::new("user")
            .with_prefix("dev")
            .with_tenant("0000")
            .with_modular("authority")
            .with_field("id")
            .with_value_type(ValueType::Obj);

        let key = builder.key(&[&1u64]);
        assert_eq!(key.key, "dev:0000:authority:user:id:obj:1");
    }

    #[test]
    fn test_cache_key_without_optional_fields() {
        let builder = SimpleCacheKeyBuilder::new("tenant");

        let key = builder.key(&[&1u64]);
        assert_eq!(key.key, "tenant:obj:1");
    }

    #[test]
    fn test_hash_key() {
        let builder = SimpleCacheKeyBuilder::new("user")
            .with_tenant("0000")
            .with_modular("authority");

        let hash_key = builder.hash_field_key(&"name", &[&1u64]);
        assert_eq!(hash_key.key, "0000:authority:user:obj:1");
        assert_eq!(hash_key.field, Some("name".to_string()));
    }

    #[test]
    fn test_multiple_uniques() {
        let builder = SimpleCacheKeyBuilder::new("user.activity")
            .with_tenant("0000")
            .with_modular("authority")
            .with_field("id.id")
            .with_value_type(ValueType::Number);

        let key = builder.key(&[&1u64, &3u64]);
        assert_eq!(key.key, "0000:authority:user.activity:id.id:number:1:3");
    }
}
