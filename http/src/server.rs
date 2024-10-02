use super::request::Request;
use super::response::Response;
use crate::threadpool::ThreadPool;
use std::io::Write;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

pub type Handler = fn(&mut Request) -> Response;

pub struct Server {
    listener: TcpListener,
    handler: Handler,
    threadpool: Option<ThreadPool>,
}

impl Server {
    /// Build a single-thread HTTP server.
    ///
    /// # Panics
    ///
    /// Will panic if the socket can't bind to the address
    pub fn new<A: ToSocketAddrs>(addr: A, handler: Handler) -> Self {
        Server {
            #[expect(clippy::unwrap_used)]
            listener: TcpListener::bind(addr).unwrap(),
            handler,
            threadpool: None,
        }
    }

    /// Build a multi-threaded HTTP server using a thread-pool.
    ///
    /// # Panics
    ///
    /// Will panic if the socket can't bind to the address
    pub fn threaded<A: ToSocketAddrs>(
        addr: A,
        handler: Handler,
        pool_count: usize,
    ) -> Self {
        Server {
            #[expect(clippy::unwrap_used)]
            listener: TcpListener::bind(addr).unwrap(),
            handler,
            threadpool: Some(ThreadPool::new(pool_count)),
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
        if let Some(pool) = &self.threadpool {
            for stream in self.listener.incoming() {
                let handler = self.handler;
                let _ = pool.execute(move || {
                    if let Err(e) = handle_connection(handler, stream) {
                        eprintln!("Error handling connection: {e:?}");
                    }
                });
            }
        } else {
            for stream in self.listener.incoming() {
                if let Err(e) = handle_connection(self.handler, stream) {
                    eprintln!("Error handling connection: {e:?}");
                }
            }
        }

        Ok(())
    }
}

/// Handles an incoming connection by parsing the HTTP request from the provided
/// `TcpStream`, invoking the `handler` to generate a response, and writing
/// the formatted HTTP response back to the stream.
///
/// # TODOs
/// Allow connections to be re-used.
#[inline]
fn handle_connection(
    handler: Handler,
    stream: std::io::Result<TcpStream>,
) -> std::io::Result<()> {
    let stream = stream?;

    #[expect(clippy::unwrap_used)]
    let mut request = Request::from(&stream).unwrap();

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

    let _ = (&stream).write_all(resp.as_bytes()); // status line + headers
    let _ = (&stream).write_all(&response.body); // body
    Ok(())
}
