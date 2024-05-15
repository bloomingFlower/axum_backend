[![Rust](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust.yml/badge.svg)](https://github.com/bloomingFlower/axum_backend/actions/workflows/rust.yml)
# Rust Web Server

## How to Run
### Run Web Server
```sh
cargo watch -q -c -w src/ -w .cargo/ -x 'run'
```
### Run Tests
```sh
cargo watch -q -c -w tests/ -x 'test -q test -- --nocapture'
```
