use serde::{Deserialize, Serialize};

use crate::entity::entity::Entity;

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeEntity<E,T> {
    #[serde(flatten)]
    pub entity: Entity<T>,
    pub parent_id: T,
    pub sort_value: i32,
    pub children: Vec<E>,
}