# fbc-builder: 微服务统一构建基础镜像
#
# 基于 Alpine（原生 musl），编译产物可直接运行在 scratch 镜像上。
#
# 构建命令（在 xilulu-server 根目录）:
#   docker build -f docker/Dockerfile.builder -t fbc-builder:latest .

FROM rust:1.88-alpine

# ===== 系统构建依赖 =====
# - musl-dev, gcc, g++, make: C/C++ 编译（musl 原生）
# - pkgconf: pkg-config 替代
# - perl: openssl 编译需要
# - cmake: rdkafka cmake-build 需要
# - openssl-dev, openssl-libs-static: TLS 静态链接
# - protobuf-dev, protoc: gRPC / tonic-build
RUN apk add --no-cache \
    pkgconf \
    perl \
    make \
    musl-dev \
    gcc \
    g++ \
    cmake \
    openssl-dev openssl-libs-static \
    protobuf-dev protoc \
    ca-certificates

# ===== Cargo 国内镜像源 =====
RUN mkdir -p /usr/local/cargo && \
    echo '[source.crates-io]' > /usr/local/cargo/config.toml && \
    echo 'replace-with = "ustc"' >> /usr/local/cargo/config.toml && \
    echo '[source.ustc]' >> /usr/local/cargo/config.toml && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> /usr/local/cargo/config.toml

# ===== cargo-chef（workspace 依赖缓存） =====
RUN cargo install cargo-chef --locked

WORKDIR /app
