`[![Rust](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust.yml/badge.svg)](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust.yml)
[![rust-clippy analyze](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust-clippy.yml)
# Rust Web Server
```plaintext
  ┌─────────────────────────────────────────┐
  │                WEB-SERVER               │
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
- [ ] ScyllaDB

### Protocols
- [x] REST
- [x] JSON-RPC 2.0
- [ ] GraphQL
- [ ] gRPC
- [ ] WebSockets

### Broker
- [ ] Apache Kafka Cluster(Strimizi)
  - [x] Install the Strimizi in Cluster
- [ ] KEDA(Kubernetes Event-Driven Autoscaling)
- [ ] Add Parameters to Prometheus

### Prod Code
- apps
- libs
- modules

### Test Code
- examples
- tests

### Don't use..
- context(..)
- expect(..)
- unwrap()

### Refer
- https://apihandyman.io/do-you-really-know-why-you-prefer-rest-over-rpc/#examples
- https://dev.to/ghost/rust-project-structure-example-step-by-step-3ee
- https://numberly.com/en/learning-rust-the-hard-way-for-a-production-kafka-scylladb-pipeline
- https://dev.to/ciscoemerge/how-to-build-a-simple-kafka-producerconsumer-application-in-rust-3pl4
- https://dev.to/ciscoemerge/how-to-build-a-kafka-producer-in-rust-with-partitioning-3168
- https://burgers.io/custom-logging-in-rust-using-tracing
- https://strimzi.io
- https://keda.sh/docs/2.14/scalers/apache-kafka

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