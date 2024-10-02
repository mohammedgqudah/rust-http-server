# HTTP In Rust
This is a toy HTTP implementation in Rust. I'm doing this solely to practice Rust.

# Example
```rust
mod http;
use http::{
    request::{Method, Request},
    server::Server,
};
use std::fs;

fn headers(request: &Request) -> Response {
    match (request.path.as_str(), &request.method) {
        ("/", Method::GET) => Response {
            body: fs::read("src/static/headers.html").expect("ON"),
            headers: Headers::new(""),
            status: Status::Ok,
        },
        ("/", Method::POST) => {
            Response {
                body: "POST!".as_bytes().to_vec(),
                headers: Headers::new(""),
                status: Status::Ok,
            }
        }
        _ => Response {
            body: format!("<h1>{} Not Found</h1>", request.path)
                .as_bytes()
                .to_vec(),
            headers: Headers::new(""),
            status: Status::NotFound,
        },
    }
}
fn main() {
    let mut server = Server::new("0.0.0.0:4000");
    let _ = server.on_request(headers).listen();
}
```

# Roadmap
- [x] A single-thread blocking version
- [x] A thread pool version
- [ ] A non-blocking single-thread version
- [ ] A complete HTTP/1 and HTTP/1.1 parser
- [ ] TLS
- [ ] HTTP/2 Implementation
