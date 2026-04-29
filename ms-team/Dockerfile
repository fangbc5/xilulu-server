# ms-team Docker 镜像（fbc-builder + scratch）
# 构建: docker build -f ms-team/Dockerfile .（workspace 根目录）
# syntax=docker/dockerfile:1

FROM fbc-builder:latest AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM fbc-builder:latest AS builder
COPY --from=planner /app/recipe.json recipe.json
COPY swagger-ui-v5.17.14.zip /app/swagger-ui-v5.17.14.zip
ENV SWAGGER_UI_DOWNLOAD_URL="file:///app/swagger-ui-v5.17.14.zip"
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json -p ms-team
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p ms-team && \
    cp target/release/ms-team /app/ms-team-bin && \
    strip /app/ms-team-bin

FROM scratch
COPY --from=builder /app/ms-team-bin /app/ms-team
WORKDIR /app
EXPOSE 30101
ENTRYPOINT ["/app/ms-team"]
