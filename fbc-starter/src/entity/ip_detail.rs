/// 用户 IP 信息
///
/// 对应 Java: IpDetail
use serde::{Deserialize, Serialize};

/// 用户 IP 详细信息
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IpDetail {
    /// 区域
    pub area: Option<String>,

    /// 注册时的 IP
    pub ip: Option<String>,

    /// 最新登录的 IP
    pub isp: Option<String>,

    /// ISP ID
    pub isp_id: Option<String>,

    /// 城市
    pub city: Option<String>,

    /// 城市 ID
    pub city_id: Option<String>,

    /// 国家
    pub country: Option<String>,

    /// 国家 ID
    pub country_id: Option<String>,

    /// 地区/省份
    pub region: Option<String>,

    /// 地区/省份 ID
    pub region_id: Option<String>,
}

impl IpDetail {
    /// 创建新的 IpDetail 实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用 Builder 模式创建实例
    pub fn builder() -> IpDetailBuilder {
        IpDetailBuilder::new()
    }
}

/// IpDetail 构建器
///
/// 对应 Java: @Builder
pub struct IpDetailBuilder {
    area: Option<String>,
    ip: Option<String>,
    isp: Option<String>,
    isp_id: Option<String>,
    city: Option<String>,
    city_id: Option<String>,
    country: Option<String>,
    country_id: Option<String>,
    region: Option<String>,
    region_id: Option<String>,
}

impl IpDetailBuilder {
    pub fn new() -> Self {
        Self {
            area: None,
            ip: None,
            isp: None,
            isp_id: None,
            city: None,
            city_id: None,
            country: None,
            country_id: None,
            region: None,
            region_id: None,
        }
    }

    pub fn area(mut self, area: impl Into<String>) -> Self {
        self.area = Some(area.into());
        self
    }

    pub fn ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self
    }

    pub fn isp(mut self, isp: impl Into<String>) -> Self {
        self.isp = Some(isp.into());
        self
    }

    pub fn isp_id(mut self, isp_id: impl Into<String>) -> Self {
        self.isp_id = Some(isp_id.into());
        self
    }

    pub fn city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    pub fn city_id(mut self, city_id: impl Into<String>) -> Self {
        self.city_id = Some(city_id.into());
        self
    }

    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    pub fn country_id(mut self, country_id: impl Into<String>) -> Self {
        self.country_id = Some(country_id.into());
        self
    }

    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn region_id(mut self, region_id: impl Into<String>) -> Self {
        self.region_id = Some(region_id.into());
        self
    }

    pub fn build(self) -> IpDetail {
        IpDetail {
            area: self.area,
            ip: self.ip,
            isp: self.isp,
            isp_id: self.isp_id,
            city: self.city,
            city_id: self.city_id,
            country: self.country,
            country_id: self.country_id,
            region: self.region,
            region_id: self.region_id,
        }
    }
}

impl Default for IpDetailBuilder {
    fn default() -> Self {
        Self::new()
    }
}
