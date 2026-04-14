/// Serde 序列化/反序列化辅助函数
use serde::{de::Error, Deserialize, Deserializer};

/// 自定义反序列化器，支持多种格式解析为 Vec<String>
///
/// 支持的格式：
/// 1. JSON 数组字符串: `["item1","item2"]` 或 `["item1", "item2"]`
/// 2. 逗号分隔字符串: `item1,item2,item3`
/// 3. 单个字符串: `item1`
/// 4. 直接的数组: `vec!["item1", "item2"]`
///
/// # 使用示例
///
/// ```rust
/// use serde::{Deserialize, Serialize};
/// use fbc_starter::utils::deserialize_string_or_vec;
///
/// #[derive(Debug, Deserialize)]
/// struct Config {
///     #[serde(default, deserialize_with = "deserialize_string_or_vec")]
///     pub items: Vec<String>,
/// }
/// ```
pub fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // 首先尝试直接反序列化为 Vec<String>（支持直接的数组格式）
    // 使用 Visitor 模式来更好地处理各种输入类型
    use serde::de::{self, Visitor};

    struct VecStringVisitor;

    impl<'de> Visitor<'de> for VecStringVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string, a string array, or a JSON array string")
        }

        // 处理直接的 Vec<String>
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element::<String>()? {
                vec.push(elem);
            }
            Ok(vec)
        }

        // 处理字符串
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_string(v.to_string())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let trimmed = v.trim();

            // 1. 尝试解析为 JSON 数组字符串
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // 移除可能的外层引号
                let json_str = if trimmed.starts_with('"') && trimmed.ends_with('"') {
                    &trimmed[1..trimmed.len() - 1]
                } else {
                    trimmed
                };

                // 先尝试解析为有效的 JSON 数组
                if let Ok(vec) = serde_json::from_str::<Vec<String>>(json_str) {
                    return Ok(vec);
                }

                // 如果解析失败，可能是数组元素没有引号，例如 [ms-identity]
                // 尝试手动解析：移除 [ 和 ]，然后按逗号分割
                let content = json_str
                    .strip_prefix('[')
                    .and_then(|s| s.strip_suffix(']'))
                    .unwrap_or(json_str);

                if !content.is_empty() {
                    let items: Vec<String> = content
                        .split(',')
                        .map(|s| {
                            // 移除可能的引号、单引号和空白
                            s.trim().trim_matches('"').trim_matches('\'').to_string()
                        })
                        .filter(|s| !s.is_empty())
                        .collect();

                    if !items.is_empty() {
                        return Ok(items);
                    }
                }
            }

            // 2. 尝试作为逗号分隔的字符串
            if trimmed.contains(',') {
                return Ok(trimmed
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect());
            }

            // 3. 作为单个字符串
            if !trimmed.is_empty() {
                Ok(vec![trimmed.to_string()])
            } else {
                Ok(Vec::new())
            }
        }
    }

    deserializer.deserialize_any(VecStringVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestConfig {
        #[serde(default, deserialize_with = "deserialize_string_or_vec")]
        pub items: Vec<String>,
    }

    #[test]
    fn test_deserialize_json_array_string() {
        let json = r#"{"items": "[\"a\",\"b\",\"c\"]"}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_deserialize_json_array_with_spaces() {
        let json = r#"{"items": "[\"a\", \"b\", \"c\"]"}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_deserialize_comma_separated() {
        let json = r#"{"items": "a,b,c"}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_deserialize_comma_separated_with_spaces() {
        let json = r#"{"items": "a, b, c"}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_deserialize_single_string() {
        let json = r#"{"items": "single-item"}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, vec!["single-item"]);
    }

    #[test]
    fn test_deserialize_direct_array() {
        let json = r#"{"items": ["x", "y", "z"]}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, vec!["x", "y", "z"]);
    }

    #[test]
    fn test_deserialize_empty_string() {
        let json = r#"{"items": ""}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, Vec::<String>::new());
    }

    #[test]
    fn test_deserialize_empty_array() {
        let json = r#"{"items": []}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, Vec::<String>::new());
    }

    #[test]
    fn test_deserialize_array_without_quotes() {
        // 测试不带引号的数组格式，例如环境变量: APP__NACOS__SUBSCRIBE_SERVICES=["ms-identity"]
        let json = r#"{"items": "[ms-identity]"}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, vec!["ms-identity"]);
    }

    #[test]
    fn test_deserialize_array_without_quotes_multiple() {
        // 测试多个元素不带引号的情况
        let json = r#"{"items": "[ms-identity,ms-auth,ms-notify]"}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, vec!["ms-identity", "ms-auth", "ms-notify"]);
    }

    #[test]
    fn test_deserialize_array_without_quotes_with_spaces() {
        // 测试带空格的情况
        let json = r#"{"items": "[ms-identity, ms-auth]"}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.items, vec!["ms-identity", "ms-auth"]);
    }
}

/// 从字符串或数字反序列化 u32
///
/// 支持的格式：
/// 1. 字符串: `"10"` -> `10`
/// 2. 数字: `10` -> `10`
///
/// # 使用示例
///
/// ```rust
/// use serde::Deserialize;
/// use fbc_starter::utils::deserialize_u32_from_str_or_num;
///
/// #[derive(Debug, Deserialize)]
/// struct QueryParams {
///     #[serde(deserialize_with = "deserialize_u32_from_str_or_num")]
///     pub page_size: u32,
/// }
/// ```
pub fn deserialize_u32_from_str_or_num<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNum {
        String(String),
        Num(u32),
    }
    match StringOrNum::deserialize(deserializer)? {
        StringOrNum::String(s) => s.parse().map_err(Error::custom),
        StringOrNum::Num(n) => Ok(n),
    }
}

/// 从字符串或数字反序列化 Option<u64>
///
/// 支持的格式：
/// 1. 字符串: `"10"` -> `Some(10)`, `""` -> `None`
/// 2. 数字: `10` -> `Some(10)`
/// 3. null: `null` -> `None`
///
/// # 使用示例
///
/// ```rust
/// use serde::Deserialize;
/// use fbc_starter::utils::deserialize_option_u64_from_str_or_num;
///
/// #[derive(Debug, Deserialize)]
/// struct QueryParams {
///     #[serde(deserialize_with = "deserialize_option_u64_from_str_or_num")]
///     pub cursor: Option<u64>,
/// }
/// ```
pub fn deserialize_option_u32_from_str_or_num<'de, D>(
    deserializer: D,
) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNum {
        String(String),
        Num(u32),
        Null,
    }
    match StringOrNum::deserialize(deserializer)? {
        StringOrNum::String(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse().map(Some).map_err(Error::custom)
            }
        }
        StringOrNum::Num(n) => {
            // 将 0 视为 None（第一页）
            if n == 0 {
                Ok(None)
            } else {
                Ok(Some(n))
            }
        }
        StringOrNum::Null => Ok(None),
    }
}
