use super::request::Request;
use super::response::Response;
use std::io::{BufWriter, Write};
use std::net::{TcpListener, ToSocketAddrs};

pub type Handler = fn(&mut Request) -> Response;

pub struct Server {
    listener: TcpListener,
    handler: Handler,
}

impl Server {
    /// # Panics
    ///
    /// Will panic if the socket can't bind to the address
    pub fn new<A: ToSocketAddrs>(addr: A, handler: Handler) -> Self {
        Server {
            #[expect(clippy::unwrap_used)]
            listener: TcpListener::bind(addr).unwrap(),
            handler,
        }
    }

    /// Start listening for incoming connections.
    ///
    /// # Panics
    ///
    /// Will panic if a request can't be parsed, this will change once 400 handlers are
    /// introduced.
    ///
    /// # Errors
    ///
    /// Will return an error if a `TCPStream` can't be opened.
    pub fn listen(&self) -> std::io::Result<()> {
        // TODO: Allow connections to be re-used.
        for stream in self.listener.incoming() {
            let stream = stream?;
            let mut writer = BufWriter::new(&stream);

            #[expect(clippy::unwrap_used)]
            let mut request = Request::from(&stream).unwrap();

            let response = (self.handler)(&mut request);

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
        Ok(())
    }
}
