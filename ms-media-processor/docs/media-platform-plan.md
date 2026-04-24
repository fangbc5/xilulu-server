# ms-media-processor — 企业级媒体处理平台规划

> **版本**: v1.0
> **更新日期**: 2026-04-23
> **定位**: 企业级异步媒体处理中台，对标阿里云 MPS（媒体处理服务）

---

## 一、平台定位与目标

### 1.1 定位

`ms-media-processor` 是一个面向企业级的 **异步媒体处理中台**，提供以下核心能力：

| 能力域 | 说明 |
|--------|------|
| **视频处理** | 抽帧截图、转码（H.264/H.265）、HLS/DASH 自适应码率切片 |
| **图片处理** | 裁剪、缩放、水印、格式转换（WebP/AVIF） |
| **流媒体** | VOD 点播切片 + 自适应码率阶梯 + Master Playlist 生成 |
| **任务调度** | 优先级队列、幂等任务、乐观锁抢占、重试/DLQ |

### 1.2 核心设计原则

```
 ┌──────────────────────────────────────────────────────────┐
 │                     设计原则                              │
 ├──────────────────────────────────────────────────────────┤
 │  1. 解耦：上游只发 Kafka 消息，不关心处理细节              │
 │  2. 幂等：同一 task_id + version 不会被重复处理            │
 │  3. 可扩展：新增处理类型只需实现 MediaProcessor trait      │
 │  4. 可观测：全链路 tracing，任务状态实时可查               │
 │  5. 容错：3 次重试 + DLQ，不会丢任务                      │
 └──────────────────────────────────────────────────────────┘
```

---

## 二、系统架构

### 2.1 整体架构图

```
                          ┌──────────────┐
                          │  ms-oss /    │
                          │  业务服务     │
                          └──────┬───────┘
                                 │ Kafka: sys.media.task.submit
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                     ms-media-processor                          │
│                                                                 │
│  ┌──────────┐   ┌──────────────┐   ┌─────────────────────────┐ │
│  │  Kafka    │──▶│  Service     │──▶│  Processor (策略模式)    │ │
│  │  Handler  │   │  (编排层)    │   │                         │ │
│  └──────────┘   └──────┬───────┘   │  ┌───────────────────┐  │ │
│                        │           │  │ VideoSnapshot      │  │ │
│                        │           │  │ VideoTranscode     │  │ │
│                        │           │  │ VideoHlsSegment    │  │ │
│                        │           │  │ ImageResize        │  │ │
│                        │           │  │ ImageWatermark     │  │ │
│                        ▼           │  │ AudioExtract       │  │ │
│                 ┌──────────────┐   │  └───────────────────┘  │ │
│                 │  Repository  │   └─────────────────────────┘ │
│                 │  (DB 持久化) │                                │
│                 └──────────────┘                                │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  基础设施层                                               │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐               │   │
│  │  │ S3Client │  │ FFmpeg   │  │ imgproxy │               │   │
│  │  │ (MinIO)  │  │ (系统)   │  │ (图片)   │               │   │
│  │  └──────────┘  └──────────┘  └──────────┘               │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
         │                                      │
         │  Kafka: sys.media.task.completed      │  S3 存储
         ▼                                      ▼
┌──────────────┐                    ┌────────────────────┐
│  业务服务     │                    │  MinIO / S3        │
│  (回调处理)   │                    │  ├── source/       │
└──────────────┘                    │  ├── derivative/   │
                                    │  └── hls/          │
                                    └───────┬────────────┘
                                            │
                                    ┌───────▼────────────┐
                                    │  Nginx             │
                                    │  ├── CDN 缓存      │
                                    │  └── HLS 分发      │
                                    └────────────────────┘
```

### 2.2 代码分层架构

```
ms-media-processor/
├── Cargo.toml
├── docs/
│   └── media-platform-plan.md       # 本文档
└── src/
    ├── main.rs                      # 入口：Server::run
    ├── config.rs                    # MediaConfig（S3 + 处理参数）
    ├── error.rs                     # MediaError
    ├── kafka/
    │   ├── mod.rs
    │   └── handler.rs               # 薄 Handler：反序列化 → 调 Service
    └── modules/
        └── media/
            ├── mod.rs
            ├── model/
            │   ├── dto.rs           # Kafka 任务事件 DTO
            │   ├── entity.rs        # MediaTask 数据库实体
            │   └── enums.rs         # TaskType / TaskStatus 枚举
            ├── repository.rs        # 数据库操作
            ├── service.rs           # ⭐ 业务编排层
            ├── s3_client.rs         # S3/MinIO 文件操作
            └── processor/           # ⭐ 策略模式处理器
                ├── mod.rs           # MediaProcessor trait + 工厂
                ├── video_snapshot.rs     # 视频截图
                ├── video_transcode.rs    # 视频转码
                ├── video_hls.rs          # HLS 自适应码率切片
                ├── image_resize.rs       # 图片裁剪/缩放
                ├── image_watermark.rs    # 图片加水印
                └── audio_extract.rs      # 音频提取
```

---

## 三、任务类型定义

### 3.1 任务类型枚举

| TaskType | 说明 | 输入 | 输出 | 参数（parameters JSON） |
|----------|------|------|------|------------------------|
| `VIDEO_SNAPSHOT` | 视频截图 | 视频文件 | JPG/PNG | `{"time_offset_ms": 5000, "format": "jpg", "quality": 85}` |
| `VIDEO_TRANSCODE` | 视频转码 | 视频文件 | MP4 | `{"codec": "h264", "resolution": "1280x720", "bitrate": "2000k"}` |
| `VIDEO_HLS` | HLS 切片 | 视频文件 | m3u8 + ts/fmp4 | `{"segment_duration": 4, "renditions": [...], "format": "fmp4"}` |
| `IMAGE_RESIZE` | 图片裁剪 | 图片文件 | 图片 | `{"width": 800, "height": 600, "mode": "cover"}` |
| `IMAGE_WATERMARK` | 图片水印 | 图片文件 | 图片 | `{"text": "©2026", "position": "bottom-right", "opacity": 0.5}` |
| `AUDIO_EXTRACT` | 音频提取 | 视频文件 | AAC/MP3 | `{"codec": "aac", "bitrate": "128k"}` |

### 3.2 任务状态机

```
           提交
            │
            ▼
         ┌──────┐    乐观锁抢占     ┌────────────┐
         │ INIT │───────────────▶│ PROCESSING │
         └──────┘                └──────┬─────┘
            ▲                          │
            │ retry_count < 3          │
            │ (重置为 INIT)             ├──── 成功 ──▶ ┌──────┐
            │                          │              │ DONE │
            └──────────────────────────┤              └──────┘
                                       │
                                       └──── 失败
                                              │
                                    retry < 3 ?
                                     ├── Yes ──▶ INIT (retry_count++)
                                     └── No  ──▶ ┌────────┐
                                                  │ FAILED │ → DLQ
                                                  └────────┘
```

---

## 四、HLS 流媒体切片方案（重点）

### 4.1 自适应码率阶梯（ABR Encoding Ladder）

为每个视频生成多个码率版本，播放器根据网络质量动态切换。

> **智能阶梯选择**：系统会自动根据源视频分辨率选择合适的阶梯。例如源视频为 1080p 时，不会生成 4K/2K 档位（避免无意义放大）。

#### 完整码率阶梯

| 档位 | 别名 | 分辨率 | 编码器 | 视频码率 | 音频码率 | Profile | 转码优先级 | 适用场景 |
|------|------|--------|--------|---------|---------|---------|-----------|----------|
| **4K HDR** | 蓝光 | 3840×2160 | H.265 (HEVC) | 15000k | 256k | Main 10 | P2 低 | 大屏 / 高端设备 |
| **4K** | 超清 | 3840×2160 | H.265 (HEVC) | 12000k | 192k | Main | P2 低 | 大屏 / 高端设备 |
| **2K** | 超清 | 2560×1440 | H.264 / H.265 | 8000k | 192k | High | P1 中 | 高端显示器 / Pad |
| **1080p** | 高清 | 1920×1080 | H.264 | 5000k | 192k | High | **P0 高** | WiFi / 高速网络 |
| **720p** | 标清 | 1280×720 | H.264 | 2800k | 128k | Main | **P0 高** | 4G / 一般 WiFi |
| **480p** | 流畅 | 854×480 | H.264 | 1400k | 128k | Main | P1 中 | 弱网兜底 |

#### 转码波次调度策略

多码率转码按**优先级分波次**异步执行，保证用户最快能看到可播放的视频：

```
Wave 0 (P0 高优)  ──▶  1080p + 720p    ← 用户最常用，优先产出
                          │ 完成后立即发布 completed 事件
                          │ 前端即可开始播放
                          ▼
Wave 1 (P1 中)    ──▶  480p + 2K        ← 补充高低两端
                          │ 完成后更新 master.m3u8
                          ▼
Wave 2 (P2 低)    ──▶  4K + 4K HDR      ← 耗时最久，后台慢慢跑
                          │ 完成后最终更新 master.m3u8
```

> **关键设计**：
> - Wave 0 完成后**立即生成 master.m3u8 并发布完成事件**，前端可开始播放 1080p/720p
> - 后续波次完成后**增量追加**到 master.m3u8，无需等全部转完
> - 每个波次内的多个档位**并行转码**，充分利用 CPU

#### 预设阶梯模板

业务方提交任务时可通过 `preset` 字段快速选择阶梯组合，无需手动指定每个档位：

| 预设名称 | 包含档位（按转码优先级排序） | 适用场景 |
|----------|---------------------------|----------|
| `mobile` | P0: 720p → P1: 480p | 移动端短视频 |
| `standard` | P0: 1080p+720p → P1: 480p | 常规 VOD 点播 |
| `premium` | P0: 1080p+720p → P1: 2K+480p | 高品质视频 |
| `ultra` | P0: 1080p+720p → P1: 2K+480p → P2: 4K | 电影 / 纪录片级 |
| `cinema` | P0: 1080p+720p → P1: 2K+480p → P2: 4K+4K HDR | 蓝光级影院品质 |
| `custom` | 由 `renditions` 数组指定 | 完全自定义 |

#### 编码器选择策略

| 分辨率 | 默认编码器 | 原因 |
|--------|-----------|------|
| ≤ 1080p | H.264 (libx264) | 兼容性最好，所有设备支持 |
| 2K | H.264（默认）/ H.265（可选） | H.265 码率更低但解码要求更高 |
| 4K / 4K HDR | H.265 (libx265) | 4K 下 H.264 码率过大，H.265 可节省 ~40% 带宽 |

> **HDR 支持**：4K HDR 档位使用 H.265 Main 10 Profile + HDR10 色彩空间（BT.2020 + SMPTE ST 2084 PQ）。要求源视频本身为 HDR 内容。

### 4.2 FFmpeg HLS 切片核心参数

```bash
# ── H.264 档位（≤ 1080p）──
ffmpeg -i input.mp4 \
  -c:v libx264 -profile:v high -preset medium \
  -b:v 2800k -maxrate 2800k -bufsize 5600k \
  -g 120 -keyint_min 120 -sc_threshold 0 \
  -flags +cgop \
  -c:a aac -b:a 128k \
  -f hls \
  -hls_time 4 \
  -hls_segment_type fmp4 \
  -hls_flags independent_segments \
  -hls_playlist_type vod \
  -hls_segment_filename "720p_%04d.m4s" \
  720p.m3u8

# ── H.265 (HEVC) 档位（4K / 2K）──
ffmpeg -i input_4k.mp4 \
  -c:v libx265 -preset medium -tag:v hvc1 \
  -b:v 12000k -maxrate 12000k -bufsize 24000k \
  -g 120 -keyint_min 120 -sc_threshold 0 \
  -c:a aac -b:a 192k \
  -f hls \
  -hls_time 4 \
  -hls_segment_type fmp4 \
  -hls_flags independent_segments \
  -hls_playlist_type vod \
  -hls_segment_filename "4k_%04d.m4s" \
  4k.m3u8

# ── 4K HDR (HEVC Main 10 + HDR10) ──
ffmpeg -i input_4k_hdr.mp4 \
  -c:v libx265 -preset medium -tag:v hvc1 \
  -profile:v main10 \
  -x265-params "hdr-opt=1:repeat-headers=1:colorprim=bt2020:transfer=smpte2084:colormatrix=bt2020nc" \
  -b:v 15000k -maxrate 15000k -bufsize 30000k \
  -g 120 -keyint_min 120 -sc_threshold 0 \
  -c:a aac -b:a 256k \
  -f hls \
  -hls_time 4 \
  -hls_segment_type fmp4 \
  -hls_flags independent_segments \
  -hls_playlist_type vod \
  -hls_segment_filename "4k_hdr_%04d.m4s" \
  4k_hdr.m3u8
```

> **注意**：H.265/HEVC 的 fMP4 段必须设置 `-tag:v hvc1` 才能被 Safari / iOS 正确识别。

### 4.3 输出目录结构（S3）

```
bucket/
└── hls/
    └── {video_id}/
        ├── master.m3u8            # Master Playlist（指向各码率版本）
        ├── 4k_hdr/                # ── HEVC Main 10 HDR ──
        │   ├── playlist.m3u8
        │   ├── init.mp4
        │   └── seg_*.m4s
        ├── 4k/                    # ── HEVC ──
        │   ├── playlist.m3u8
        │   ├── init.mp4
        │   └── seg_*.m4s
        ├── 2k/                    # ── H.264 或 HEVC ──
        │   └── ...
        ├── 1080p/                 # ── H.264 ──
        │   ├── playlist.m3u8
        │   ├── init.mp4
        │   ├── seg_0000.m4s
        │   ├── seg_0001.m4s
        │   └── ...
        ├── 720p/
        │   └── ...
        └── 480p/
            └── ...
```

### 4.4 Master Playlist 示例

```m3u8
#EXTM3U
#EXT-X-VERSION:7

## ── HEVC (H.265) 高清档位 ──

#EXT-X-STREAM-INF:BANDWIDTH=15256000,RESOLUTION=3840x2160,CODECS="hvc1.2.4.L153.B0,mp4a.40.2",VIDEO-RANGE=PQ
4k_hdr/playlist.m3u8

#EXT-X-STREAM-INF:BANDWIDTH=12192000,RESOLUTION=3840x2160,CODECS="hvc1.1.6.L150.B0,mp4a.40.2"
4k/playlist.m3u8

#EXT-X-STREAM-INF:BANDWIDTH=8192000,RESOLUTION=2560x1440,CODECS="avc1.640032,mp4a.40.2"
2k/playlist.m3u8

## ── H.264 标准档位 ──

#EXT-X-STREAM-INF:BANDWIDTH=5192000,RESOLUTION=1920x1080,CODECS="avc1.640028,mp4a.40.2"
1080p/playlist.m3u8

#EXT-X-STREAM-INF:BANDWIDTH=2928000,RESOLUTION=1280x720,CODECS="avc1.64001f,mp4a.40.2"
720p/playlist.m3u8

#EXT-X-STREAM-INF:BANDWIDTH=1528000,RESOLUTION=854x480,CODECS="avc1.64001e,mp4a.40.2"
480p/playlist.m3u8
```

> **HEVC 兼容性说明**：`hvc1` 编码标识被 Safari (macOS/iOS) 和大多数智能电视原生支持。Chrome / Firefox 在桌面端对 HLS HEVC 支持有限，但通过 fMP4 + MSE 可以用 hls.js 软解播放。平台会自动生成 H.264 版本作为 fallback。

---

## 五、处理器策略模式设计

### 5.1 核心 Trait

```rust
/// 媒体处理器接口 — 所有处理类型的统一抽象
#[async_trait]
pub trait MediaProcessor: Send + Sync {
    /// 处理类型标识
    fn task_type(&self) -> &str;

    /// 执行处理
    /// 
    /// - input_dir:  本地临时输入目录（文件已下载好）
    /// - output_dir: 本地临时输出目录（处理完的文件放这里）
    /// - params:     任务参数 JSON
    ///
    /// 返回：衍生文件的 S3 key 列表
    async fn process(
        &self,
        input_path: &Path,
        output_dir: &Path,
        params: &serde_json::Value,
    ) -> Result<Vec<ProcessOutput>, MediaError>;
}

/// 处理输出描述
pub struct ProcessOutput {
    /// 本地文件路径
    pub local_path: PathBuf,
    /// 目标 S3 key
    pub s3_key: String,
    /// 文件 MIME 类型
    pub content_type: String,
}
```

### 5.2 处理器工厂

```rust
/// 根据 task_type 获取对应的处理器
pub fn get_processor(task_type: &str) -> Result<Box<dyn MediaProcessor>, MediaError> {
    match task_type {
        "VIDEO_SNAPSHOT"   => Ok(Box::new(VideoSnapshotProcessor)),
        "VIDEO_TRANSCODE"  => Ok(Box::new(VideoTranscodeProcessor)),
        "VIDEO_HLS"        => Ok(Box::new(VideoHlsProcessor)),
        "IMAGE_RESIZE"     => Ok(Box::new(ImageResizeProcessor)),
        "IMAGE_WATERMARK"  => Ok(Box::new(ImageWatermarkProcessor)),
        "AUDIO_EXTRACT"    => Ok(Box::new(AudioExtractProcessor)),
        _ => Err(MediaError::UnsupportedTaskType(task_type.to_string())),
    }
}
```

### 5.3 Service 编排层

```rust
impl MediaTaskService {
    /// 处理入口 — 由 Kafka handler 调用
    pub async fn process(&self, event: SubmitTaskEvent) -> Result<(), MediaError> {
        let task_id = &event.task_id;

        // 1. 查询任务 + 幂等校验
        let task = self.repo.get_task(task_id).await?
            .ok_or_else(|| MediaError::TaskNotFound(task_id.clone()))?;

        // 2. 乐观锁抢占
        if !self.repo.claim_task(task_id, task.version.unwrap_or(1)).await? {
            return Ok(()); // 已被其他 worker 抢占
        }

        // 3. 获取处理器（策略模式）
        let processor = get_processor(&event.task_type)?;

        // 4. 准备临时工作目录
        let work_dir = self.prepare_work_dir(task_id).await?;

        // 5. 下载源文件
        let input_path = self.s3.download_to_dir(
            &event.source.bucket, &event.source.key, &work_dir
        ).await?;

        // 6. 执行处理
        let output_dir = work_dir.join("output");
        let outputs = processor.process(&input_path, &output_dir, 
            &event.parameters.unwrap_or_default()).await?;

        // 7. 上传全部衍生文件到 S3
        for output in &outputs {
            self.s3.upload_from_file(
                &event.source.bucket,
                &output.s3_key,
                &output.local_path,
                &output.content_type,
            ).await?;
        }

        // 8. 更新状态 + 发布完成事件
        let primary_key = outputs.first()
            .map(|o| o.s3_key.clone())
            .unwrap_or_default();
        self.repo.mark_done(task_id, &primary_key).await?;
        self.publish_completed(task_id, &event, &outputs).await;

        // 9. 清理
        self.cleanup_work_dir(&work_dir).await;
        Ok(())
    }
}
```

---

## 六、Nginx 流媒体分发方案

### 6.1 架构

```
客户端 (hls.js / Video.js)
    │
    │  GET /hls/{video_id}/master.m3u8
    ▼
┌──────────────┐
│  Nginx       │
│  ├── 缓存层   │ ── 命中 → 直接返回
│  └── 代理层   │ ── 未命中 → 回源到 MinIO
└──────┬───────┘
       │ proxy_pass
       ▼
┌──────────────┐
│  MinIO / S3  │
│  hls/{id}/   │
└──────────────┘
```

### 6.2 Nginx HLS 分发配置

```nginx
# /etc/nginx/conf.d/hls_vod.conf

# HLS 片段缓存区
proxy_cache_path /var/cache/nginx/hls
    levels=1:2
    keys_zone=hls_cache:50m
    max_size=50g
    inactive=30d
    use_temp_path=off;

server {
    listen 80;
    server_name media.example.com;

    # HLS 播放列表 — 短缓存（便于更新）
    location ~* \.m3u8$ {
        proxy_pass http://minio:9000;
        proxy_cache hls_cache;
        proxy_cache_valid 200 1m;       # 播放列表缓存 1 分钟
        
        add_header Access-Control-Allow-Origin *;
        add_header X-Cache-Status $upstream_cache_status;
        
        # CORS 支持 HLS
        add_header Access-Control-Allow-Methods "GET, OPTIONS";
        add_header Access-Control-Allow-Headers "Range";
        add_header Access-Control-Expose-Headers "Content-Length,Content-Range";
    }

    # HLS 切片文件 — 长缓存（不变资源）
    location ~* \.(ts|m4s|mp4)$ {
        proxy_pass http://minio:9000;
        proxy_cache hls_cache;
        proxy_cache_valid 200 30d;      # 切片文件缓存 30 天
        
        add_header Cache-Control "public, max-age=2592000, immutable";
        add_header Access-Control-Allow-Origin *;
        add_header X-Cache-Status $upstream_cache_status;
    }

    # 缩略图和衍生文件
    location /derivative/ {
        proxy_pass http://minio:9000;
        proxy_cache hls_cache;
        proxy_cache_valid 200 7d;
        
        add_header Cache-Control "public, max-age=604800";
        add_header X-Cache-Status $upstream_cache_status;
    }
}
```

---

## 七、数据库设计

### 7.1 media_task 表

```sql
CREATE TABLE `media_task` (
    `id`             VARCHAR(64) NOT NULL COMMENT '任务ID（UUID）',
    `source_bucket`  VARCHAR(64) COMMENT '源 Bucket',
    `source_key`     VARCHAR(512) COMMENT '源文件路径',
    `task_type`      VARCHAR(32) NOT NULL COMMENT '任务类型',
    `parameters`     TEXT COMMENT '任务参数 JSON',
    `status`         VARCHAR(20) NOT NULL DEFAULT 'INIT' COMMENT 'INIT/PROCESSING/DONE/FAILED',
    `priority`       TINYINT NOT NULL DEFAULT 0 COMMENT '优先级：0普通 1高 2紧急',
    `retry_count`    INT NOT NULL DEFAULT 0 COMMENT '重试次数',
    `max_retry`      INT NOT NULL DEFAULT 3 COMMENT '最大重试次数',
    `version`        INT NOT NULL DEFAULT 1 COMMENT '乐观锁版本',
    `result_key`     VARCHAR(512) COMMENT '主衍生文件路径（如 master.m3u8）',
    `result_meta`    TEXT COMMENT '衍生文件元信息 JSON',
    `error_message`  TEXT COMMENT '错误信息',
    `callback_topic` VARCHAR(128) COMMENT '完成通知的 Kafka topic',
    `created_by`     VARCHAR(64) COMMENT '提交方服务名',
    `created_at`     BIGINT NOT NULL COMMENT '创建时间',
    `updated_at`     BIGINT NOT NULL COMMENT '更新时间',
    PRIMARY KEY (`id`),
    INDEX `idx_status` (`status`),
    INDEX `idx_task_type` (`task_type`),
    INDEX `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='媒体处理任务表';
```

### 7.2 media_task_output 表（一对多：一个任务可产出多个文件）

```sql
CREATE TABLE `media_task_output` (
    `id`            BIGINT AUTO_INCREMENT COMMENT '主键',
    `task_id`       VARCHAR(64) NOT NULL COMMENT '关联任务ID',
    `output_key`    VARCHAR(512) NOT NULL COMMENT 'S3 路径',
    `output_type`   VARCHAR(32) COMMENT '输出类型：thumbnail/playlist/segment/audio',
    `content_type`  VARCHAR(64) COMMENT 'MIME 类型',
    `file_size`     BIGINT COMMENT '文件大小（字节）',
    `metadata`      TEXT COMMENT '额外元信息 JSON',
    `created_at`    BIGINT NOT NULL COMMENT '创建时间',
    PRIMARY KEY (`id`),
    INDEX `idx_task_id` (`task_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='媒体任务输出文件表';
```

---

## 八、Kafka 消息协议

### 8.1 Topic 定义

| Topic | 方向 | 说明 |
|-------|------|------|
| `sys.media.task.submit` | **入** | 业务服务提交处理任务 |
| `sys.media.task.completed` | **出** | 任务完成通知（含产物信息） |
| `sys.media.task.progress` | **出** | 处理进度上报（可选，用于长时间转码） |
| `sys.media.task.dlq` | **出** | 死信队列（超过最大重试） |

### 8.2 提交事件 SubmitTaskEvent

```json
{
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "task_type": "VIDEO_HLS",
    "source": {
        "bucket": "user-uploads",
        "key": "videos/original/abc123.mp4"
    },
    "parameters": {
        "preset": "premium",
        "segment_duration": 4,
        "format": "fmp4"
    },
    "priority": "high",
    "callback_topic": "biz.video.ready"
}
```

### 8.3 完成事件 CompletedTaskEvent

```json
{
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "DONE",
    "task_type": "VIDEO_HLS",
    "original_source": "videos/original/abc123.mp4",
    "result": {
        "master_playlist": "hls/abc123/master.m3u8",
        "outputs": [
            {
                "key": "hls/abc123/1080p/playlist.m3u8",
                "type": "playlist",
                "content_type": "application/vnd.apple.mpegurl"
            },
            {
                "key": "hls/abc123/1080p/init.mp4",
                "type": "segment",
                "content_type": "video/mp4"
            }
        ],
        "duration_ms": 126400,
        "thumbnail": "derivative/abc123_thumb.jpg"
    },
    "processing_time_ms": 45230
}
```

---

## 九、分期实施计划

### Phase 1 — 架构重构 + 基础能力（当前冲刺）

> **目标**：将现有代码重构为规范的分层架构，稳定视频截图能力

- [ ] 重构目录结构：合并 `task` + `worker` → `media` 模块
- [ ] 抽出 `MediaTaskService` 编排层
- [ ] 定义 `MediaProcessor` trait + 处理器工厂
- [ ] 将现有 `VideoSnapshot` 迁移到策略模式
- [ ] 删除死代码（`ProcessState`, `state.rs`）
- [ ] 修复 `config.rs` 配置加载方式
- [ ] 创建 `media_task` 表 DDL

### Phase 2 — HLS 流媒体切片

> **目标**：实现 HLS 自适应码率切片，支持视频点播

- [ ] 实现 `VideoHlsProcessor`
  - 多码率并行转码
  - fMP4 分段
  - Master Playlist 自动生成
- [ ] HLS 产物上传到 S3 分层目录
- [ ] 新增 Nginx HLS 分发配置（`docker-compose-media.yml`）
- [ ] 端到端测试：上传 → 切片 → hls.js 播放

### Phase 3 — 图片处理 + 视频转码

> **目标**：补齐常规媒体处理能力

- [ ] `ImageResizeProcessor`（对接 imgproxy 或本地处理）
- [ ] `ImageWatermarkProcessor`
- [ ] `VideoTranscodeProcessor`（H.264/H.265）
- [ ] `AudioExtractProcessor`
- [ ] S3 大文件流式传输（避免 OOM）
- [ ] 临时文件目录配置化

### Phase 4 — 生产加固

> **目标**：达到生产级稳定性和可观测性

- [ ] 任务优先级队列
- [ ] 进度上报（长时间转码场景）
- [ ] 硬件加速探测（NVENC / VA-API / VideoToolbox）
- [ ] 任务超时取消机制
- [ ] Prometheus 指标暴露
- [ ] 任务管理 HTTP API（查询/重试/取消）
- [ ] 负载控制（并发任务数限制）

---

## 十、基础设施依赖

| 组件 | 用途 | Docker 镜像 | 状态 |
|------|------|------------|------|
| MinIO | S3 对象存储 | `minio/minio` | ✅ 已部署 |
| Kafka | 消息队列 | `bitnami/kafka` | ✅ 已部署 |
| MySQL | 任务持久化 | `mysql:8.0` | ✅ 已部署 |
| imgproxy | 动态图片处理 | `darthsim/imgproxy` | ✅ 已部署 |
| Nginx CDN | HLS 分发 + 缓存 | `nginx:alpine` | ⚠️ 需扩展 HLS 配置 |
| FFmpeg | 视频处理引擎 | 系统安装 / Docker 内置 | ⚠️ 需确认版本 |

---

## 十一、关键技术决策

| 决策点 | 选择 | 原因 |
|--------|------|------|
| HLS 切片格式 | fMP4 (CMAF) | 比 MPEG-TS 更高效，HLS/DASH 兼容 |
| GOP 模式 | 闭合 GOP (cgop) | ABR 切换必须的前提条件 |
| 切片时长 | 4 秒 | 兼顾编码效率与播放启动速度 |
| 处理器架构 | 策略模式 (trait) | 新增任务类型零侵入 |
| 文件传输 | 先下载到本地处理 | FFmpeg 不支持流式处理 S3 |
| 图片处理 | imgproxy 优先 | 动态处理+CDN 缓存，避免重复计算 |
| 视频编码器 (≤1080p) | H.264 (libx264) | 兼容性最好，所有设备支持 |
| 视频编码器 (4K/2K) | H.265 (libx265) | 相比 H.264 节省 ~40% 带宽，4K 必选 |
| HDR 支持 | HEVC Main 10 + HDR10 | BT.2020 色域 + PQ 传递函数 |
| HEVC fMP4 标签 | `-tag:v hvc1` | Safari / iOS 原生 HLS HEVC 必需 |
