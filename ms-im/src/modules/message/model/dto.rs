use serde::Deserialize;

/// 发送消息请求
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    /// 房间 ID
    pub room_id: i64,
    /// 消息内容（JSON 字符串）
    pub content: String,
    /// 消息类型: 1文本 2图片 3文件 4语音 5视频
    pub r#type: i16,
    /// 回复的消息 ID（可选）
    pub reply_msg_id: Option<i64>,
    /// 扩展信息
    pub extra: Option<serde_json::Value>,
}

/// 消息游标分页查询参数
#[derive(Debug, Deserialize)]
pub struct MessageCursorQuery {
    /// 房间 ID
    pub room_id: i64,
    /// 游标（上一页最后一条消息的 ID，首次不传）
    pub cursor: Option<i64>,
    /// 每页条数（默认 20，最大 50）
    #[serde(default = "default_page_size")]
    pub size: i64,
    /// 抓取方向（0: 历史向下查, 1: 未读新消息向上查）
    #[serde(default)]
    pub fetch_mode: i16,
}

fn default_page_size() -> i64 {
    20
}

/// 消息标记请求
#[derive(Debug, Deserialize)]
pub struct MarkRequest {
    /// 标记类型: 1点赞 2举报
    pub r#type: i16,
}

/// 游标分页响应
#[derive(Debug, serde::Serialize)]
pub struct CursorPageResponse<T: serde::Serialize> {
    /// 数据列表
    pub list: Vec<T>,
    /// 下一页游标（None 表示没有更多数据）
    pub cursor: Option<i64>,
    /// 是否还有更多
    pub has_more: bool,
}
