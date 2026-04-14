use serde::{Deserialize, Serialize};

use crate::entity::entity::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TanentEntity<T> {
    #[serde(flatten)]
    pub entity: Entity<T>,
    pub tenant_id: i64,
}
