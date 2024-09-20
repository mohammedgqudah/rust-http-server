use super::request::Request;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::net::{TcpListener, ToSocketAddrs};

pub struct Server {
    listener: TcpListener,
    paths: std::collections::HashMap<String, Box<dyn Fn(&Request) -> String>>,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Self {
        Server {
            listener: TcpListener::bind(addr).unwrap(),
            paths: HashMap::new(),
        }
    }

    pub fn listen(&self) -> std::io::Result<()> {
        // TODO: Allow connections to be re-used.
        for stream in self.listener.incoming() {
            let stream = stream?;
            let mut writer = BufWriter::new(&stream);
            let request = Request::from(&stream).unwrap();
            if let Some(handler) = self.paths.get(&request.path) {
                let content = handler(&request);
                let resp = format!(
                    "{} 200 OK\r\nContent-Length:{}\r\n\r\n{content}",
                    request.http_version,
                    content.len()
                );
                let _ = writer.write_all(resp.as_bytes());
            } else {
                let content = format!("<h1>404, Page {} not found.</h1>", request.path);
                let resp = format!(
                    "{} 404 NOT FOUND\r\nContent-Length:{}\r\n\r\n{content}",
                    request.http_version,
                    content.len()
                );
                let _ = writer.write_all(resp.as_bytes());
            }
            println!("Connection closed");
        }

        Ok(())
    }

    pub fn get<F>(&mut self, path: &'static str, handler: F) -> &mut Self
    where
        F: Fn(&Request) -> String + 'static,
    {
        self.paths.insert(path.to_string(), Box::new(handler));
        self
    }
}
