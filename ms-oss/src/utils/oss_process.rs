//! `x-oss-process` 参数解析与 imgproxy 翻译引擎
//!
//! 将阿里云 OSS 风格的参数 `image/resize,m_fill,w_128,h_128/format,webp`
//! 翻译为 imgproxy 处理指令 `rs:fill:128:128` + 格式后缀 `@webp`。

/// 处理类型
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessType {
    /// 图片实时处理（走 imgproxy）
    Image,
    /// 视频截帧产物（查 DB）
    Video,
    /// Style 预设（先展开，再按 Image/Video 处理）
    Style(String),
}

/// 解析后的处理参数
#[derive(Debug, Clone)]
pub struct ProcessParams {
    /// 处理类型
    pub process_type: ProcessType,
    /// imgproxy 处理指令部分（如 `rs:fill:128:128/q:85`）
    pub imgproxy_processing: String,
    /// 输出格式（如 `webp`、`jpg`），从 format 指令提取
    pub output_format: Option<String>,
    /// 视频截帧参数（毫秒）
    pub video_snapshot_time: Option<i64>,
    /// 视频截帧尺寸
    pub video_width: Option<u32>,
    pub video_height: Option<u32>,
}

/// 解析 `x-oss-process` 参数
///
/// 输入：`image/resize,m_fill,w_128,h_128/format,webp`
/// 输入：`video/snapshot,t_0`
/// 输入：`style/avatar_small`
pub fn parse(raw: &str) -> anyhow::Result<ProcessParams> {
    let raw = raw.trim();
    if raw.is_empty() {
        anyhow::bail!("x-oss-process 参数为空");
    }

    // 按 / 分割，第一段决定类型
    let parts: Vec<&str> = raw.splitn(2, '/').collect();
    if parts.is_empty() {
        anyhow::bail!("x-oss-process 格式无效: {}", raw);
    }

    match parts[0] {
        "image" => {
            let commands = if parts.len() > 1 { parts[1] } else { "" };
            parse_image_commands(commands)
        }
        "video" => {
            let commands = if parts.len() > 1 { parts[1] } else { "" };
            parse_video_commands(commands)
        }
        "style" => {
            let style_name = if parts.len() > 1 { parts[1] } else { "" };
            if style_name.is_empty() {
                anyhow::bail!("style 名称为空");
            }
            Ok(ProcessParams {
                process_type: ProcessType::Style(style_name.to_string()),
                imgproxy_processing: String::new(),
                output_format: None,
                video_snapshot_time: None,
                video_width: None,
                video_height: None,
            })
        }
        other => anyhow::bail!("不支持的处理类型: {}", other),
    }
}

/// 解析图片处理命令链
///
/// 输入：`resize,m_fill,w_128,h_128/quality,q_85/format,webp`
/// 输出：imgproxy 指令 + 格式
fn parse_image_commands(commands: &str) -> anyhow::Result<ProcessParams> {
    let mut imgproxy_parts: Vec<String> = Vec::new();
    let mut output_format: Option<String> = None;

    // 按 / 分割各处理步骤
    for cmd in commands.split('/') {
        let cmd = cmd.trim();
        if cmd.is_empty() {
            continue;
        }

        // 按逗号分割参数
        let kvs: Vec<&str> = cmd.split(',').collect();
        if kvs.is_empty() {
            continue;
        }

        match kvs[0] {
            "resize" => {
                imgproxy_parts.push(translate_resize(&kvs[1..]));
            }
            "crop" => {
                imgproxy_parts.push(translate_crop(&kvs[1..]));
            }
            "quality" => {
                if let Some(q) =
                    extract_param(&kvs[1..], "q").or_else(|| extract_param(&kvs[1..], "Q"))
                {
                    imgproxy_parts.push(format!("q:{}", q));
                }
            }
            "format" => {
                if kvs.len() > 1 {
                    output_format = Some(kvs[1].to_string());
                }
            }
            _ => {
                tracing::warn!("未知的图片处理命令: {}", kvs[0]);
            }
        }
    }

    Ok(ProcessParams {
        process_type: ProcessType::Image,
        imgproxy_processing: imgproxy_parts.join("/"),
        output_format,
        video_snapshot_time: None,
        video_width: None,
        video_height: None,
    })
}

/// 翻译 resize 命令为 imgproxy 指令
///
/// 输入参数（已去掉 "resize"）：`["m_fill", "w_128", "h_128"]`
/// 输出：`rs:fill:128:128`
fn translate_resize(params: &[&str]) -> String {
    let mut mode = "fit";
    let mut width = "0";
    let mut height = "0";
    let mut percentage: Option<&str> = None;

    for p in params {
        if let Some(v) = p.strip_prefix("m_") {
            mode = match v {
                "lfit" => "fit",
                "fill" => "fill",
                "fixed" => "force",
                "mfit" => "fill-down",
                _ => "fit",
            };
        } else if let Some(v) = p.strip_prefix("w_") {
            width = v;
        } else if let Some(v) = p.strip_prefix("h_") {
            height = v;
        } else if let Some(v) = p.strip_prefix("p_") {
            percentage = Some(v);
        }
    }

    if let Some(pct) = percentage {
        format!("rs:fit:{}p:{}p", pct, pct)
    } else {
        format!("rs:{}:{}:{}", mode, width, height)
    }
}

/// 翻译 crop 命令为 imgproxy 指令
fn translate_crop(params: &[&str]) -> String {
    let mut width = "0";
    let mut height = "0";
    let mut gravity = "ce";
    let mut offset_x: Option<&str> = None;
    let mut offset_y: Option<&str> = None;

    for p in params {
        if let Some(v) = p.strip_prefix("w_") {
            width = v;
        } else if let Some(v) = p.strip_prefix("h_") {
            height = v;
        } else if let Some(v) = p.strip_prefix("g_") {
            gravity = match v {
                "center" | "centre" => "ce",
                "nw" => "nowe",
                "ne" => "noea",
                "sw" => "sowe",
                "se" => "soea",
                "north" => "no",
                "south" => "so",
                "west" => "we",
                "east" => "ea",
                _ => "ce",
            };
        } else if let Some(v) = p.strip_prefix("x_") {
            offset_x = Some(v);
        } else if let Some(v) = p.strip_prefix("y_") {
            offset_y = Some(v);
        }
    }

    match (offset_x, offset_y) {
        (Some(x), Some(y)) => format!("c:{}:{}:{}:{}:{}", width, height, gravity, x, y),
        _ => format!("c:{}:{}:{}", width, height, gravity),
    }
}

/// 解析视频处理命令
fn parse_video_commands(commands: &str) -> anyhow::Result<ProcessParams> {
    let kvs: Vec<&str> = commands.split(',').collect();
    let mut snapshot_time: Option<i64> = None;
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;

    // 跳过 "snapshot" 前缀
    for kv in &kvs {
        if let Some(v) = kv.strip_prefix("t_") {
            snapshot_time = v.parse().ok();
        } else if let Some(v) = kv.strip_prefix("w_") {
            width = v.parse().ok();
        } else if let Some(v) = kv.strip_prefix("h_") {
            height = v.parse().ok();
        }
    }

    Ok(ProcessParams {
        process_type: ProcessType::Video,
        imgproxy_processing: String::new(),
        output_format: None,
        video_snapshot_time: snapshot_time,
        video_width: width,
        video_height: height,
    })
}

/// 从参数数组中提取指定前缀的值
fn extract_param<'a>(params: &[&'a str], prefix: &str) -> Option<&'a str> {
    let prefix_with_underscore = format!("{}_", prefix);
    params
        .iter()
        .find_map(|p| p.strip_prefix(&prefix_with_underscore))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_resize() {
        let result = parse("image/resize,m_fill,w_128,h_128/format,webp").unwrap();
        assert_eq!(result.process_type, ProcessType::Image);
        assert_eq!(result.imgproxy_processing, "rs:fill:128:128");
        assert_eq!(result.output_format, Some("webp".to_string()));
    }

    #[test]
    fn test_parse_image_pipeline() {
        let result = parse("image/resize,m_fill,w_300,h_300/quality,q_85/format,webp").unwrap();
        assert_eq!(result.imgproxy_processing, "rs:fill:300:300/q:85");
        assert_eq!(result.output_format, Some("webp".to_string()));
    }

    #[test]
    fn test_parse_image_percentage() {
        let result = parse("image/resize,p_50").unwrap();
        assert_eq!(result.imgproxy_processing, "rs:fit:50p:50p");
    }

    #[test]
    fn test_parse_video() {
        let result = parse("video/snapshot,t_1000,w_800,h_600").unwrap();
        assert_eq!(result.process_type, ProcessType::Video);
        assert_eq!(result.video_snapshot_time, Some(1000));
        assert_eq!(result.video_width, Some(800));
        assert_eq!(result.video_height, Some(600));
    }

    #[test]
    fn test_parse_style() {
        let result = parse("style/avatar_small").unwrap();
        assert_eq!(
            result.process_type,
            ProcessType::Style("avatar_small".to_string())
        );
    }

    #[test]
    fn test_parse_crop() {
        let result = parse("image/crop,w_300,h_200,g_center").unwrap();
        assert_eq!(result.imgproxy_processing, "c:300:200:ce");
    }
}
