# ms-im Docker 镜像（fbc-builder + scratch）
# 构建: docker build -f ms-im/Dockerfile .（workspace 根目录）
# syntax=docker/dockerfile:1

FROM fbc-builder:latest AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM fbc-builder:latest AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json -p ms-im
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p ms-im && \
    cp target/release/ms-im /app/ms-im-bin && \
    strip /app/ms-im-bin

FROM scratch
COPY --from=builder /app/ms-im-bin /app/ms-im
WORKDIR /app
EXPOSE 30102
ENTRYPOINT ["/app/ms-im"]
