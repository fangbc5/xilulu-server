# ms-identity Docker 镜像（fbc-builder + scratch）
# 构建: docker build -f ms-identity/Dockerfile .（workspace 根目录）
# syntax=docker/dockerfile:1

FROM fbc-builder:latest AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM fbc-builder:latest AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json -p ms-identity
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p ms-identity && \
    cp target/release/ms-identity /app/ms-identity-bin && \
    strip /app/ms-identity-bin

FROM scratch
COPY --from=builder /app/ms-identity-bin /app/ms-identity
WORKDIR /app
EXPOSE 30001
ENTRYPOINT ["/app/ms-identity"]
