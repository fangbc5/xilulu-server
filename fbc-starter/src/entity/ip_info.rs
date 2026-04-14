use crate::entity::ip_detail::IpDetail;
/// 用户 IP 信息
///
/// 对应 Java: IpInfo
use serde::{Deserialize, Serialize};

/// 用户 IP 信息
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IpInfo {
    /// 注册时的 IP
    pub create_ip: Option<String>,

    /// 注册时的 IP 详情
    pub create_ip_detail: Option<IpDetail>,

    /// 最新登录的 IP
    pub update_ip: Option<String>,

    /// 最新登录的 IP 详情
    pub update_ip_detail: Option<IpDetail>,
}

impl IpInfo {
    /// 创建新的 IpInfo 实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 刷新 IP
    ///
    /// 如果 IP 为空，则不更新
    /// 如果 create_ip 为空，则同时设置 create_ip 和 update_ip
    /// 否则只更新 update_ip
    pub fn refresh_ip(&mut self, ip: Option<String>) {
        let ip = match ip {
            Some(ip) if !ip.trim().is_empty() => ip,
            _ => return,
        };

        self.update_ip = Some(ip.clone());

        if self.create_ip.is_none() {
            self.create_ip = Some(ip);
        }
    }

    /// 需要刷新的 IP
    ///
    /// 判断更新 IP 是否需要刷新详情
    /// 如果 update_ip_detail 存在且其 IP 与 update_ip 相同，则不需要刷新
    /// 返回需要刷新的 IP，如果不需要刷新则返回 None
    pub fn need_refresh_ip(&self) -> Option<&String> {
        let not_need_refresh = self
            .update_ip_detail
            .as_ref()
            .and_then(|detail| detail.ip.as_ref())
            .map(|ip| ip == self.update_ip.as_ref().unwrap_or(&String::new()))
            .unwrap_or(false);

        if not_need_refresh {
            None
        } else {
            self.update_ip.as_ref()
        }
    }

    /// 刷新 IP 详情
    ///
    /// 根据 IP 详情中的 IP 地址，更新对应的 create_ip_detail 或 update_ip_detail
    pub fn refresh_ip_detail(&mut self, ip_detail: IpDetail) {
        if let Some(ip) = &ip_detail.ip {
            // 如果 IP 详情中的 IP 与 create_ip 相同，更新 create_ip_detail
            if self.create_ip.as_ref().map(|s| s == ip).unwrap_or(false) {
                self.create_ip_detail = Some(ip_detail.clone());
            }

            // 如果 IP 详情中的 IP 与 update_ip 相同，更新 update_ip_detail
            if self.update_ip.as_ref().map(|s| s == ip).unwrap_or(false) {
                self.update_ip_detail = Some(ip_detail);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refresh_ip() {
        let mut ip_info = IpInfo::new();

        // 测试首次设置 IP
        ip_info.refresh_ip(Some("127.0.0.1".to_string()));
        assert_eq!(ip_info.create_ip, Some("127.0.0.1".to_string()));
        assert_eq!(ip_info.update_ip, Some("127.0.0.1".to_string()));

        // 测试更新 IP
        ip_info.refresh_ip(Some("192.168.1.1".to_string()));
        assert_eq!(ip_info.create_ip, Some("127.0.0.1".to_string())); // create_ip 不变
        assert_eq!(ip_info.update_ip, Some("192.168.1.1".to_string())); // update_ip 更新

        // 测试空 IP（不更新）
        ip_info.refresh_ip(Some("".to_string()));
        assert_eq!(ip_info.update_ip, Some("192.168.1.1".to_string())); // 不变

        // 测试 None（不更新）
        ip_info.refresh_ip(None);
        assert_eq!(ip_info.update_ip, Some("192.168.1.1".to_string())); // 不变
    }

    #[test]
    fn test_need_refresh_ip() {
        let mut ip_info = IpInfo::new();

        // 没有 update_ip，返回 None
        assert_eq!(ip_info.need_refresh_ip(), None);

        // 有 update_ip 但没有 update_ip_detail，需要刷新
        ip_info.update_ip = Some("127.0.0.1".to_string());
        assert_eq!(ip_info.need_refresh_ip(), Some(&"127.0.0.1".to_string()));

        // 有 update_ip_detail 但 IP 不匹配，需要刷新
        ip_info.update_ip_detail = Some(IpDetail::builder().ip("192.168.1.1".to_string()).build());
        assert_eq!(ip_info.need_refresh_ip(), Some(&"127.0.0.1".to_string()));

        // 有 update_ip_detail 且 IP 匹配，不需要刷新
        ip_info.update_ip_detail = Some(IpDetail::builder().ip("127.0.0.1".to_string()).build());
        assert_eq!(ip_info.need_refresh_ip(), None);
    }

    #[test]
    fn test_refresh_ip_detail() {
        let mut ip_info = IpInfo::new();
        ip_info.create_ip = Some("127.0.0.1".to_string());
        ip_info.update_ip = Some("192.168.1.1".to_string());

        // 刷新 create_ip 的详情
        let create_detail = IpDetail::builder()
            .ip("127.0.0.1".to_string())
            .city("北京".to_string())
            .build();
        ip_info.refresh_ip_detail(create_detail.clone());
        assert_eq!(ip_info.create_ip_detail, Some(create_detail));
        assert_eq!(ip_info.update_ip_detail, None);

        // 刷新 update_ip 的详情
        let update_detail = IpDetail::builder()
            .ip("192.168.1.1".to_string())
            .city("上海".to_string())
            .build();
        ip_info.refresh_ip_detail(update_detail.clone());
        assert_eq!(ip_info.update_ip_detail, Some(update_detail));

        // 刷新不匹配的 IP（不应该更新）
        let other_detail = IpDetail::builder()
            .ip("10.0.0.1".to_string())
            .city("广州".to_string())
            .build();
        let create_detail_before = ip_info.create_ip_detail.clone();
        let update_detail_before = ip_info.update_ip_detail.clone();
        ip_info.refresh_ip_detail(other_detail);
        assert_eq!(ip_info.create_ip_detail, create_detail_before);
        assert_eq!(ip_info.update_ip_detail, update_detail_before);
    }

    #[test]
    fn test_deserialize_from_json() {
        let json_str = r#"{
            "createIp": "206.237.119.215",
            "updateIp": "120.231.232.41",
            "createIpDetail": {
                "ip": "206.237.119.215",
                "isp": "",
                "area": "",
                "city": "",
                "ispId": "",
                "region": "",
                "cityId": "",
                "country": "美国",
                "regionId": "",
                "countryId": "US"
            },
            "updateIpDetail": {
                "ip": "120.231.232.41",
                "isp": "移动",
                "area": "",
                "city": "",
                "ispId": "100025",
                "region": "广东",
                "cityId": "",
                "country": "中国",
                "regionId": "440000",
                "countryId": "CN"
            }
        }"#;

        let ip_info: IpInfo = serde_json::from_str(json_str).expect("反序列化 JSON 失败");

        // 验证 create_ip
        assert_eq!(ip_info.create_ip, Some("206.237.119.215".to_string()));

        // 验证 update_ip
        assert_eq!(ip_info.update_ip, Some("120.231.232.41".to_string()));

        // 验证 create_ip_detail
        assert!(ip_info.create_ip_detail.is_some());
        let create_detail = ip_info.create_ip_detail.as_ref().unwrap();
        assert_eq!(create_detail.ip, Some("206.237.119.215".to_string()));
        assert_eq!(create_detail.country, Some("美国".to_string()));
        assert_eq!(create_detail.country_id, Some("US".to_string()));
        assert_eq!(create_detail.isp, Some("".to_string()));
        assert_eq!(create_detail.area, Some("".to_string()));

        // 验证 update_ip_detail
        assert!(ip_info.update_ip_detail.is_some());
        let update_detail = ip_info.update_ip_detail.as_ref().unwrap();
        assert_eq!(update_detail.ip, Some("120.231.232.41".to_string()));
        assert_eq!(update_detail.isp, Some("移动".to_string()));
        assert_eq!(update_detail.isp_id, Some("100025".to_string()));
        assert_eq!(update_detail.region, Some("广东".to_string()));
        assert_eq!(update_detail.region_id, Some("440000".to_string()));
        assert_eq!(update_detail.country, Some("中国".to_string()));
        assert_eq!(update_detail.country_id, Some("CN".to_string()));
    }

    #[test]
    fn test_serialize_to_json() {
        let mut ip_info = IpInfo::new();
        ip_info.create_ip = Some("206.237.119.215".to_string());
        ip_info.update_ip = Some("120.231.232.41".to_string());

        let create_detail = IpDetail::builder()
            .ip("206.237.119.215".to_string())
            .country("美国".to_string())
            .country_id("US".to_string())
            .isp("".to_string())
            .area("".to_string())
            .build();
        ip_info.create_ip_detail = Some(create_detail);

        let update_detail = IpDetail::builder()
            .ip("120.231.232.41".to_string())
            .isp("移动".to_string())
            .isp_id("100025".to_string())
            .region("广东".to_string())
            .region_id("440000".to_string())
            .country("中国".to_string())
            .country_id("CN".to_string())
            .build();
        ip_info.update_ip_detail = Some(update_detail);

        let json_str = serde_json::to_string(&ip_info).expect("序列化 JSON 失败");

        // 验证可以反序列化回来
        let deserialized: IpInfo = serde_json::from_str(&json_str).expect("反序列化 JSON 失败");

        assert_eq!(deserialized.create_ip, ip_info.create_ip);
        assert_eq!(deserialized.update_ip, ip_info.update_ip);
        assert_eq!(deserialized.create_ip_detail, ip_info.create_ip_detail);
        assert_eq!(deserialized.update_ip_detail, ip_info.update_ip_detail);
    }
}
