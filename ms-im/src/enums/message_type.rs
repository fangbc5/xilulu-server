/// 消息类型枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageTypeEnum {
    /// 正常消息
    Text = 1,
    /// 撤回消息
    Recall = 2,
    /// 图片
    Img = 3,
    /// 文件
    File = 4,
    /// 语音
    Sound = 5,
    /// 视频
    Video = 6,
    /// 表情
    Emoji = 7,
    /// 系统消息
    System = 8,
    /// 合并消息
    Merge = 9,
    /// 公告消息
    Notice = 10,
    /// 机器人
    Bot = 11,
    /// 视频电话消息
    VideoCall = 12,
    /// 音频电话消息
    AudioCall = 13,
    /// 混合消息
    Mixed = 14,
    /// 艾特
    Ait = 15,
    /// 回复
    Reply = 16,
    /// AI
    Ai = 17,
    /// 地图
    Location = 18,
}

impl MessageTypeEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            MessageTypeEnum::Text => "正常消息",
            MessageTypeEnum::Recall => "撤回消息",
            MessageTypeEnum::Img => "图片",
            MessageTypeEnum::File => "文件",
            MessageTypeEnum::Sound => "语音",
            MessageTypeEnum::Video => "视频",
            MessageTypeEnum::Emoji => "表情",
            MessageTypeEnum::System => "系统消息",
            MessageTypeEnum::Merge => "合并消息",
            MessageTypeEnum::Notice => "公告消息",
            MessageTypeEnum::Bot => "机器人",
            MessageTypeEnum::VideoCall => "视频电话消息",
            MessageTypeEnum::AudioCall => "音频电话消息",
            MessageTypeEnum::Mixed => "混合消息",
            MessageTypeEnum::Ait => "艾特",
            MessageTypeEnum::Reply => "回复",
            MessageTypeEnum::Ai => "AI",
            MessageTypeEnum::Location => "地图",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, MessageTypeEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, MessageTypeEnum::Text);
    map.insert(2, MessageTypeEnum::Recall);
    map.insert(3, MessageTypeEnum::Img);
    map.insert(4, MessageTypeEnum::File);
    map.insert(5, MessageTypeEnum::Sound);
    map.insert(6, MessageTypeEnum::Video);
    map.insert(7, MessageTypeEnum::Emoji);
    map.insert(8, MessageTypeEnum::System);
    map.insert(9, MessageTypeEnum::Merge);
    map.insert(10, MessageTypeEnum::Notice);
    map.insert(11, MessageTypeEnum::Bot);
    map.insert(12, MessageTypeEnum::VideoCall);
    map.insert(13, MessageTypeEnum::AudioCall);
    map.insert(14, MessageTypeEnum::Mixed);
    map.insert(15, MessageTypeEnum::Ait);
    map.insert(16, MessageTypeEnum::Reply);
    map.insert(17, MessageTypeEnum::Ai);
    map.insert(18, MessageTypeEnum::Location);
    map
});

impl MessageTypeEnum {
    /// 根据类型值获取枚举
    pub fn of(type_val: i32) -> Option<Self> {
        CACHE.get(&type_val).copied()
    }
}

