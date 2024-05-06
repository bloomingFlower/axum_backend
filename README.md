# Rust Web Server

## How to Run
### Run Web Server
```sh
cargo watch -q -c -w src/ -x 'run'
```
### Run Tests
```sh
cargo watch -q -c -w tests/ -x 'test -q test -- --nocapture'
```