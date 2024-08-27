# Web-server image
FROM debian:bookworm-slim AS web-server

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set PROJECT_ROOT environment variable
ENV PROJECT_ROOT=/usr/src/app

# Copy the pre-built binary and SQL files
ARG BINARY_PATH
COPY ${BINARY_PATH} /usr/local/bin/web-server
COPY sql /usr/src/app/sql

CMD ["web-server"]
EXPOSE 3000

# SSE-server image
FROM debian:bookworm-slim AS sse-service

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set PROJECT_ROOT environment variable
ENV PROJECT_ROOT=/usr/src/app

# Copy the pre-built binary and SQL files
ARG BINARY_PATH
COPY ${BINARY_PATH} /usr/local/bin/sse-service
COPY sql /usr/src/app/sql

CMD ["sse-service"]
EXPOSE 3001