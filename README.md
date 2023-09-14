### Intro

A mini-redis http_server, supporting `get`, `set (with ttl)`, `del`.


### quick to start

```bash
cargo run --bin server
cargo run --bin http_server
```

Then you can test with 
```bash
cargo run --bin http_client
```