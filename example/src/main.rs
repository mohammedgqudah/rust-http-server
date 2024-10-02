use clap::Parser;
use http::{
    request::{Method, Request},
    response::{Headers, Response, Status},
    server::Server,
};
use std::fs;

fn headers(request: &mut Request) -> Response {
    match (request.path(), &request.method) {
        ("/headers", Method::Get) => Response {
            body: fs::read("example/src/static/headers.html").expect("ON"),
            headers: Headers::new("X-Server: RustHTTP"),
            status: Status::Ok,
        },
        ("/redirect", Method::Get) => Response::new(
            Status::TemporaryRedirect,
            Headers::new("Location: /login"),
            Vec::new(),
        ),
        ("/headers", Method::Post) => {
            let body = request.body.as_mut().unwrap().all_bytes();
            let body = String::from_utf8(body).unwrap_or_else(|_| "not utf8".to_string());

            let content_type = request
                .headers
                .as_ref()
                .and_then(|headers| headers.get("Content-Type"))
                .map(|_type| _type.to_string())
                .unwrap_or_else(|| "None".to_string());

            let resp = format!(
                "<h1>body</h1>
                <pre><code>{}</code></pre>
                <hr/>
                Content-Type: <code>{}</code>",
                body, content_type
            );

            Response {
                body: resp.as_bytes().to_vec(),
                headers: Headers {
                    headers: String::new(),
                },
                status: Status::Ok,
            }
        }
        _ => Response {
            body: format!("<h1>{} Not Found</h1>", request.path())
                .as_bytes()
                .to_vec(),
            headers: Headers {
                headers: String::new(),
            },
            status: Status::NotFound,
        },
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Thread pool size (enable threading with --threading)
    #[arg(short = 'c', long, default_value_t = 50)]
    threads_count: usize,

    #[arg(short, long)]
    threaded: bool,
}

fn main() {
    let args = Args::parse();
    let server = if args.threaded {
        Server::threaded("0.0.0.0:4000", headers, args.threads_count)
    } else {
        Server::new("0.0.0.0:4000", headers)
    };
    server.listen().unwrap();
}
