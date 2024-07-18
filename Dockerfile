# Build the web-server and sse-service binaries
FROM --platform=$BUILDPLATFORM rust:1.79 as builder

LABEL maintainer="JYY <yourrubber@duck.com>"

# Install cmake and other necessary dependencies
RUN apt-get update && apt-get install -y cmake libssl-dev pkg-config

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --bin web-server
RUN cargo build --release --bin sse-service

FROM --platform=$TARGETPLATFORM debian:bullseye-slim as web-server

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/web-server /usr/local/bin/web-server

CMD ["web-server"]

FROM --platform=$TARGETPLATFORM debian:bullseye-slim as sse-server

RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/sse-service /usr/local/bin/sse-service

CMD ["sse-service"]