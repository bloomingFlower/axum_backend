# Dockerfile for the axum-backend service

# Build the web-server binary
# Be careful with the rust version
FROM rust:1.79 as builder

LABEL maintainer="JYY <yourrubber@duck.com>"

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --bin web-server

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/web-server /usr/local/bin/web-server

CMD ["web-server"]