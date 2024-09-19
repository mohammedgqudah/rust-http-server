mod http;
use http::{request::Request, server::Server};

fn headers(request: &Request) -> String {
    format!(
        "<h1>Headers:</h1>
        <pre><code>{:#?}</code></pre>
        <h2>body</h2>
        <code>{}</code>
        <form method='POST' action='/headers'>
            <input name=\"name\"/>
            <button>submit</button>
        </form>",
        request.headers,
        String::from_utf8(request.body.clone().unwrap_or("Nothing".into()))
            .unwrap_or("not utf8".to_string())
    )
}
fn main() {
    let mut server = Server::new("0.0.0.0:4000");
    let _ = server
        .get("/", |r| format!("YOOO {}", r.path))
        .get("/robots.txt", |_r| "/xyz\n/admin".to_string())
        .get("/headers", headers)
        .listen();
}
