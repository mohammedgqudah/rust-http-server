# HTTP In Rust
This is a toy HTTP implementation in Rust. I'm doing this solely to practice Rust.

# Example
```rust
use http::server::Server;

fn main() {
    let mut server = Server::new("0.0.0.0:4000");
    let _ = server
        .get("/", |r| format!("Path: {}", r.path))
        .get("/hello", |_| "World".to_string())
        .listen();
}
```

# Roadmap
- [ ] Complete HTTP parser
- [ ] TLS
- [ ] Path variables
