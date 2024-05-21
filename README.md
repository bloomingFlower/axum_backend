[![Rust](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust.yml/badge.svg)](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust.yml)
# Rust Web Server

## How to Run
### Run Web Server
```sh
cargo watch -q -c -w src/ -w .cargo/ -x 'run'
```
### Run Tests
```sh
cargo watch -q -c -x "test model::task -- --nocapture"
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
