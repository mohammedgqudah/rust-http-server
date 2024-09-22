use super::request::Request;
use super::response::Response;
use std::io::{BufWriter, Write};
use std::net::{TcpListener, ToSocketAddrs};

pub type Handler = Option<Box<dyn Fn(&mut Request) -> Response>>;

pub struct Server {
    listener: TcpListener,
    handler: Handler,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Self {
        Server {
            listener: TcpListener::bind(addr).unwrap(),
            handler: None,
        }
    }

    pub fn listen(&self) -> std::io::Result<()> {
        // TODO: Allow connections to be re-used.
        for stream in self.listener.incoming() {
            let stream = stream?;
            let mut writer = BufWriter::new(&stream);
            let mut request = Request::from(&stream).unwrap();

            if let Some(handler) = &self.handler {
                let response = handler(&mut request);

                let mut headers = response.headers.headers;
                if !headers.is_empty() {
                    headers.insert_str(0, "\r\n");
                }

                let resp = format!(
                    "{http_version} {status_number} {status_description}\r\nContent-Length: {len}{headers}\r\n\r\n",
                    http_version = request.http_version,
                    status_number = response.status as u16,
                    status_description = response.status,
                    headers = headers,
                    len = response.body.len(),
                );

                let _ = writer.write_all(resp.as_bytes()); // status line + headers
                let _ = writer.write_all(&response.body); // body
            }

            println!("Connection closed");
        }

        Ok(())
    }

    pub fn on_request<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(&mut Request) -> Response + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }
}
