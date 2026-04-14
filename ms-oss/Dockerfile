# ms-oss Docker 镜像（fbc-builder + scratch）
# 构建: docker build -f ms-oss/Dockerfile .（workspace 根目录）
# 注意: 如果 OSS endpoint 使用 HTTPS，需要加回 CA 证书
# syntax=docker/dockerfile:1

FROM fbc-builder:latest AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM fbc-builder:latest AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json -p ms-oss
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p ms-oss && \
    cp target/release/ms-oss /app/ms-oss-bin && \
    strip /app/ms-oss-bin

FROM scratch
COPY --from=builder /app/ms-oss-bin /app/ms-oss
WORKDIR /app
EXPOSE 30003
ENTRYPOINT ["/app/ms-oss"]
