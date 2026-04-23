# ms-media-processor Docker 镜像（fbc-builder + scratch）
# 构建: docker build -f ms-media-processor/Dockerfile .（workspace 根目录）
# 注意: FFmpeg 需以静态链接方式打包，或使用 alpine 运行镜像
# syntax=docker/dockerfile:1

FROM fbc-builder:latest AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM fbc-builder:latest AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json -p ms-media-processor
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p ms-media-processor && \
    cp target/release/ms-media-processor /app/ms-media-processor-bin && \
    strip /app/ms-media-processor-bin

# 使用 alpine 作为运行镜像（需要 FFmpeg 系统依赖）
FROM alpine:3.21
RUN apk add --no-cache ffmpeg ca-certificates
COPY --from=builder /app/ms-media-processor-bin /app/ms-media-processor
WORKDIR /app
# 创建临时工作目录
RUN mkdir -p /tmp/media_work
EXPOSE 30105
ENTRYPOINT ["/app/ms-media-processor"]
