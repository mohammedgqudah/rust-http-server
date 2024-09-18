mod http;
use http::server::Server;

fn main() {
    let mut server = Server::new("0.0.0.0:4000");
    let _ = server
        .get("/", |r| format!("YOOO {}", r.path))
        .get("/robots.txt", |_r| "/xyz\n/admin".to_string())
        .listen();
}
