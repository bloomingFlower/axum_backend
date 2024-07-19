# Builder stage
FROM rust:latest AS builder

# Create APT cache directory and set permissions
RUN mkdir -p /var/cache/apt/archives/partial && \
    chmod 755 /var/cache/apt/archives/partial

# Update package lists
RUN apt-get update

# Install dependencies
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    cmake pkg-config libssl-dev

# Clean up
RUN apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .

# Build both services
RUN cargo build --release --bin web-server --bin sse-service

# Web-server image
FROM debian:bullseye-slim AS web-server

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/web-server /usr/local/bin/web-server

CMD ["web-server"]
EXPOSE 3000

# SSE-server image
FROM debian:bullseye-slim AS sse-server

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/sse-service /usr/local/bin/sse-service

CMD ["sse-service"]
EXPOSE 3001

# # Builder stage
# FROM --platform=$BUILDPLATFORM rust:1.79 AS builder
# ARG TARGETPLATFORM

# # Install common dependencies
# RUN apt-get update && \
#     DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
#     cmake pkg-config ca-certificates \
#     gcc-x86-64-linux-gnu g++-x86-64-linux-gnu libc6-dev-amd64-cross \
#     gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross \
#     libssl-dev:amd64 libssl-dev:arm64 && \
#     rm -rf /var/lib/apt/lists/*

# # Set up cross-compilation
# RUN rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu && \
#     mkdir -p ~/.cargo && \
#     echo '[target.x86_64-unknown-linux-gnu]\nlinker = "x86_64-linux-gnu-gcc"\n\n[target.aarch64-unknown-linux-gnu]\nlinker = "aarch64-linux-gnu-gcc"' >> ~/.cargo/config

# WORKDIR /usr/src/app
# COPY . .

# # Build for the target platform
# RUN case "$TARGETPLATFORM" in \
#     "linux/amd64") \
#     export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc && \
#     export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc && \
#     export CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++ && \
#     export OPENSSL_DIR=/usr/include/x86_64-linux-gnu/openssl && \
#     export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu && \
#     export OPENSSL_INCLUDE_DIR=/usr/include/x86_64-linux-gnu && \
#     cargo build --bin web-server --target x86_64-unknown-linux-gnu ;; \
#     "linux/arm64") \
#     export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc && \
#     export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc && \
#     export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ && \
#     export OPENSSL_DIR=/usr/include/aarch64-linux-gnu/openssl && \
#     export OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu && \
#     export OPENSSL_INCLUDE_DIR=/usr/include/aarch64-linux-gnu && \
#     cargo build --bin web-server --target aarch64-unknown-linux-gnu ;; \
#     *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
#     esac

# # Web-server image
# FROM debian:bullseye-slim AS web-server
# ARG TARGETPLATFORM

# RUN apt-get update && \
#     DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
#     libssl1.1 ca-certificates && \
#     rm -rf /var/lib/apt/lists/*

# # Copy the appropriate binary based on the target platform
# COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-gnu/debug/web-server /usr/local/bin/web-server.amd64
# COPY --from=builder /usr/src/app/target/aarch64-unknown-linux-gnu/debug/web-server /usr/local/bin/web-server.arm64

# # Select the appropriate binary
# RUN if [ "$TARGETPLATFORM" = "linux/arm64" ] && [ -f /usr/local/bin/web-server.arm64 ]; then \
#     mv /usr/local/bin/web-server.arm64 /usr/local/bin/web-server; \
#     elif [ "$TARGETPLATFORM" = "linux/amd64" ] && [ -f /usr/local/bin/web-server.amd64 ]; then \
#     mv /usr/local/bin/web-server.amd64 /usr/local/bin/web-server; \
#     else \
#     echo "No suitable web-server binary found for $TARGETPLATFORM" && exit 1; \
#     fi

# CMD ["web-server"]

# # Builder stage
# FROM --platform=$BUILDPLATFORM rust:1.79 AS builder
# ARG TARGETPLATFORM

# # Install common dependencies
# RUN apt-get update && \
#     DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
#     cmake libssl-dev pkg-config ca-certificates && \
#     rm -rf /var/lib/apt/lists/*

# # Set up cross-compilation for ARM64
# RUN case "$TARGETPLATFORM" in \
#     "linux/arm64") \
#     DEBIAN_FRONTEND=noninteractive apt-get update && \
#     apt-get install -y --no-install-recommends gcc-aarch64-linux-gnu && \
#     rustup target add aarch64-unknown-linux-gnu && \
#     mkdir -p ~/.cargo && \
#     echo '[target.aarch64-unknown-linux-gnu]\nlinker = "aarch64-linux-gnu-gcc"' >> ~/.cargo/config && \
#     rm -rf /var/lib/apt/lists/* ;; \
#     esac

# WORKDIR /usr/src/app
# COPY . .

# # Build for the target platform
# RUN case "$TARGETPLATFORM" in \
#     "linux/amd64") \
#     cargo build --release --bin web-server --bin sse-service ;; \
#     "linux/arm64") \
#     cargo build --release --bin web-server --bin sse-service --target aarch64-unknown-linux-gnu ;; \
#     *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
#     esac

# # Web-server image
# FROM debian:bullseye-slim AS web-server
# ARG TARGETPLATFORM

# RUN apt-get update && \
#     DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
#     libssl1.1 ca-certificates && \
#     rm -rf /var/lib/apt/lists/*

# # Copy the appropriate binary based on the target platform
# COPY --from=builder /usr/src/app/target/release/web-server /usr/local/bin/web-server.amd64
# COPY --from=builder /usr/src/app/target/aarch64-unknown-linux-gnu/release/web-server /usr/local/bin/web-server.arm64

# # Select the appropriate binary
# RUN if [ "$TARGETPLATFORM" = "linux/arm64" ] && [ -f /usr/local/bin/web-server.arm64 ]; then \
#     mv /usr/local/bin/web-server.arm64 /usr/local/bin/web-server; \
#     elif [ "$TARGETPLATFORM" = "linux/amd64" ] && [ -f /usr/local/bin/web-server.amd64 ]; then \
#     mv /usr/local/bin/web-server.amd64 /usr/local/bin/web-server; \
#     else \
#     echo "No suitable web-server binary found for $TARGETPLATFORM" && exit 1; \
#     fi

# CMD ["web-server"]

# # SSE-server image
# FROM debian:bullseye-slim AS sse-server
# ARG TARGETPLATFORM

# RUN apt-get update && \
#     DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
#     libssl1.1 ca-certificates && \
#     rm -rf /var/lib/apt/lists/*

# # Copy the appropriate binary based on the target platform
# COPY --from=builder /usr/src/app/target/release/sse-service /usr/local/bin/sse-service.amd64
# COPY --from=builder /usr/src/app/target/aarch64-unknown-linux-gnu/release/sse-service /usr/local/bin/sse-service.arm64

# # Select the appropriate binary
# RUN if [ "$TARGETPLATFORM" = "linux/arm64" ] && [ -f /usr/local/bin/sse-service.arm64 ]; then \
#     mv /usr/local/bin/sse-service.arm64 /usr/local/bin/sse-service; \
#     elif [ "$TARGETPLATFORM" = "linux/amd64" ] && [ -f /usr/local/bin/sse-service.amd64 ]; then \
#     mv /usr/local/bin/sse-service.amd64 /usr/local/bin/sse-service; \
#     else \
#     echo "No suitable sse-service binary found for $TARGETPLATFORM" && exit 1; \
#     fi

# CMD ["sse-service"]