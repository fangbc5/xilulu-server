use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperEntity<T> {
    pub id: T,
    pub create_time: DateTime<Utc>,
    pub create_by: T,
    pub tenant_id: String,
    pub is_del: bool,
}