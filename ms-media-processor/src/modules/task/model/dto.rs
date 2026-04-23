use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceObject {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitTaskEvent {
    pub task_id: String,
    pub task_type: String,
    pub source: SourceObject,
    pub parameters: Option<serde_json::Value>,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletedTaskEvent {
    pub task_id: String,
    pub status: String,
    pub original_source: String,
    pub result: Option<ResultObject>,
    pub error_msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultObject {
    pub bucket: String,
    pub key: String,
    pub size: Option<i64>,
}
