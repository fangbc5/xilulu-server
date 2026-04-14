use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entity::super_entity::SuperEntity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity<T> {
    #[serde(flatten)]
    pub super_entity: SuperEntity<T>,
    pub update_time: DateTime<Utc>,
    pub update_by: T,
}