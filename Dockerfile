# builder stage
FROM --platform=$BUILDPLATFORM rust:1.79 as builder
ARG TARGETPLATFORM

# Cross-compilation settings for ARM64
RUN apt-get update && apt-get install -y cmake libssl-dev pkg-config \
    && case "$TARGETPLATFORM" in \
    "linux/arm64") \
    apt-get install -y gcc-aarch64-linux-gnu \
    && rustup target add aarch64-unknown-linux-gnu \
    && echo '[target.aarch64-unknown-linux-gnu]\nlinker = "aarch64-linux-gnu-gcc"' >> ~/.cargo/config ;; \
    esac \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .

# build command based on the target platform
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") \
    cargo build --release --bin web-server && \
    cargo build --release --bin sse-service ;; \
    "linux/arm64") \
    cargo build --release --bin web-server --target aarch64-unknown-linux-gnu && \
    cargo build --release --bin sse-service --target aarch64-unknown-linux-gnu ;; \
    esac

# web-server image
FROM --platform=$TARGETPLATFORM debian:bullseye-slim as web-server

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /usr/src/app/target/release/web-server /usr/local/bin/web-server
COPY --from=builder /usr/src/app/target/aarch64-unknown-linux-gnu/release/web-server /usr/local/bin/web-server

CMD ["web-server"]

# sse-server image
FROM --platform=$TARGETPLATFORM debian:bullseye-slim as sse-server

RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /usr/src/app/target/release/sse-service /usr/local/bin/sse-service
COPY --from=builder /usr/src/app/target/aarch64-unknown-linux-gnu/release/sse-service /usr/local/bin/sse-service

CMD ["sse-service"]