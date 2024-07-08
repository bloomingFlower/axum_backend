[![Rust](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust.yml/badge.svg)](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust.yml)
[![rust-clippy analyze](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust-clippy.yml)

# Rust Web Server

```plaintext
  ┌─────────────────────────────────────────┐
  │                WEB-SERVER               │
  │    log         middleware     routes    │
  └─────────────────────────────────────────┘
  ┌────────────┐┌─────────────┐┌────────────┐
  │  CONTEXT   ││    EVENT    ││    RPC     │
  └────────────┘└─────────────┘└────────────┘
  ┌───────────────────────────┐┌────────────┐
  │            MODEL          ││    AUTH    │
  │       ┌─────────────┐     │└────────────┘
  │       │    STORE    │     │┌────────────┐
  │       └─────────────┘     ││    UTILS   │
  └───────────────────────────┘└────────────┘
```

## How to Run

### Generate the key

```sh
cargo run -p gen-key
```

### Run Web Server

```sh
#cargo watch -q -c -w src/ -w .cargo/ -x 'run'
cargo run -p web-server
```

### Run Tests

```sh
#cargo watch -q -c -x "test -- --nocapture"
#cargo watch -q -c -w examples/ -x "run --example test"
cargo run -p web-server --example test
```

### Starting the DB

```sh
# Start the DB docker container
docker run --rm --name pg -p 5432:5432 \
-e POSTGRES_USER=dev \
-e POSTGRES_PASSWORD=dev \
-e POSTGRES_DB=dev \
postgres:16.3

# (optional) psql terminal
docker exec -it pg psql -U dev -d postgres

# (optional) connect to the db
\c dev_app

# (optional) describe the tables
\d

# (optional) print all sql statements
ALTER DATABASE postgres SET log_statement = 'all';
```

## Future Work

### Database

- [x] ORM (sqlb > sea-query)
- [x] ScyllaDB

### Protocols

- [x] REST
- [x] JSON-RPC 2.0
- [ ] gRPC
- [x] SSE
- [x] WebSockets
- [ ] GraphQL

### ESP

- [ ] Apache Kafka Cluster(Strimizi)
- [x] Install the Strimizi in Cluster
- [ ] KEDA(Kubernetes Event-Driven Autoscaling)
- [ ] Add Parameters to Prometheus
- [ ] Kafka Stream Processing (Topic -> Topic)

### Prod Code

- apps
- libs
- modules

### Test Code

- examples
- tests

### Don't use

- `context(..)`
- `expect(..)`
- `unwrap()`

### PWD Multi-Scheme

- #1 HMAC (#01#)
- #2 Argon2 (#02#)

### Add a new topic in Docker

```sh
docker exec broker \
>    kafka-topics --bootstrap-server broker:9092 \
>                 --create \
>                 --topic hnstories
```

### ESP Idea

- Source: User Activity, Metrics, Logs, Financial Transactions
- Destination: Databases, Notification, Analytics, Data Warehouses, Data Lakes, Dashboards, Real-time Applications

### Kafka CLI

```sh
# Create a topic
kafka-topics --bootstrap-server localhost:9092 --create --topic test --partitions 1 --replication-factor 1

# List all topics
kafka-topics --bootstrap-server localhost:9092 --list

# Describe a topic
kafka-topics --bootstrap-server localhost:9092 --describe --topic test

# Delete a topic
kafka-topics --bootstrap-server localhost:9092 --delete --topic test

# Produce a message to a topic, without key
kafka-console-producer --broker-list localhost:9092 --topic test
> Hello, World!
> This is a message
> ^D

# Produce a message to a topic, with key
kafka-console-producer --broker-list localhost:9092 --topic test --property parse.key=true --property key.separator=:
> user1: Login Event
> user1: Click Event
> user1: Logout Event

# Consume messages from a topic, offset from beginning
kafka-console-consumer --bootstrap-server localhost:9092 --topic test --from-beginning
> (offset 0) log1
> (offset 1) log2

# Consume messages from a topic, offset 1
kafka-console-consumer --bootstrap-server localhost:9092 --topic test
> (offset 2) log3
> (offset 3) log4
```

### ScyllaDB CLI

```sh
sudo docker run -d --name Node_X -p 9042:9042 -p 7000:7000 scylladb/scylla:latest
sudo docker exec -it Node_X cqlsh
```

### Refer

- <https://apihandyman.io/do-you-really-know-why-you-prefer-rest-over-rpc/#examples>
- <https://dev.to/ghost/rust-project-structure-example-step-by-step-3ee>
- <https://numberly.com/en/learning-rust-the-hard-way-for-a-production-kafka-scylladb-pipeline>
- <https://dev.to/ciscoemerge/how-to-build-a-simple-kafka-producerconsumer-application-in-rust-3pl4>
- <https://dev.to/ciscoemerge/how-to-build-a-kafka-producer-in-rust-with-partitioning-3168> [ ]
- <https://burgers.io/custom-logging-in-rust-using-tracing>
- <https://strimzi.io>
- <https://keda.sh/docs/2.14/scalers/apache-kafka>
- <https://blog.logrocket.com/build-websocket-server-with-rust>

### Why Rust

- Secure
  - Memory and thead safety
  - No runtime or garbage collector
- Easy to deploy
  - Small sized binaries are self-sufficient
  - No runtime dependencies
- No compromises
  - Strongly and statically typed
  - Exhaustive checking
  - Built-in error management syntax and primitives
- Play well with others
  - C, C++, Python, Node.js, Java, Ruby, Go, etc.
  - PyO3 can be used to run Rust from Python (or vice versa)
