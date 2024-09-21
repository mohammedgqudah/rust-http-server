pub mod body;
pub mod chunked;

use body::{BodyDecoder, Chunk};
use chunked::ChunkedDecoder;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::str::FromStr;

#[derive(Debug)]
pub enum HttpVersion {
    V0_9,
    V1_0,
    V1_1,
    V2_0,
    V3_0,
}

pub struct Request<'a> {
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Box<dyn BodyDecoder + 'a>>,
    pub path: String,

    #[allow(dead_code)]
    pub query_string: &'a str,
    pub http_version: HttpVersion,
    pub method: Method,
}

impl std::fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HttpVersion::V0_9 => "HTTP/0.9",
                HttpVersion::V1_0 => "HTTP/1.0",
                HttpVersion::V1_1 => "HTTP/1.1",
                HttpVersion::V2_0 => "HTTP/2.0",
                HttpVersion::V3_0 => "HTTP/3.0",
            }
        )
    }
}

impl FromStr for HttpVersion {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<HttpVersion, Self::Err> {
        match input {
            "HTTP/0.9" => Ok(HttpVersion::V0_9),
            "HTTP/1.0" => Ok(HttpVersion::V1_0),
            "HTTP/1.1" => Ok(HttpVersion::V1_1),
            "HTTP/2.0" => Ok(HttpVersion::V2_0),
            "HTTP/3.0" => Ok(HttpVersion::V3_0),
            _ => Err("Unknown HTTP version"),
        }
    }
}

#[derive(PartialEq)]
pub enum Method {
    Head,
    Get,
    Options,
    Post,
    Put,
    Patch,
    Delete,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Get => "GET",
                Method::Head => "HEAD",
                Method::Options => "OPTIONS",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Patch => "PATCH",
                Method::Delete => "DELETE",
            }
        )
    }
}

impl FromStr for Method {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Method, Self::Err> {
        match input {
            "GET" => Ok(Method::Get),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "PATCH" => Ok(Method::Patch),
            "DELETE" => Ok(Method::Delete),
            _ => Err("Invalid HTTP Method"),
        }
    }
}

struct Body<B: BufRead> {
    buf: B,
    length: usize,
    done: bool,
}

impl<B: BufRead> Body<B> {
    fn new(length: usize, buf: B) -> Self {
        Body {
            buf,
            length,
            done: false,
        }
    }
}

impl<B: BufRead> Iterator for Body<B> {
    type Item = Result<Chunk, &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let mut chunk = Chunk {
            buf: vec![0; self.length],
            extension: String::new(),
        };

        match self.buf.read_exact(&mut chunk.buf) {
            Ok(_) => {}
            Err(_) => {
                self.done = true;
                return Some(Err("Expected a body"));
            }
        };

        self.done = true;
        Some(Ok(chunk))
    }
}

impl<B: BufRead> BodyDecoder for Body<B> {}

impl<'a> Request<'a> {
    pub fn from(stream: &'a TcpStream) -> Result<Self, String> {
        // TODO: Support url-encoding
        let mut buf = BufReader::new(stream);
        let mut lines = buf.by_ref().lines();

        let request_line = lines
            .next()
            .ok_or("Expected request line")?
            .map_err(|_| "Couldn't get request line")?;

        let mut lines = lines
            // TODO: don't use unwrap
            .take_while(|line| !line.as_ref().unwrap().is_empty())
            .peekable();

        // TODO: Parse headers only when asked to.
        // This will pose a challenge to internally used headers such as Content-Length,
        // but this can be solved by saving the headers we're interested in as a variable or struct.
        let headers = match lines.peek() {
            None => None,
            Some(_) => {
                let mut map: HashMap<String, String> = HashMap::new();
                lines.try_for_each(|line| -> Result<(), String> {
                    let binding = line.map_err(|_| "Expected a header".to_string())?;
                    let mut pair = binding.split(':');
                    if let (Some(key), Some(value)) = (pair.next(), pair.next()) {
                        // TODO: Store headers in lower-case.
                        // TODO: Store both `Referer` and `Referrer`
                        map.insert(key.to_string(), value.trim().to_string());
                        Ok(())
                    } else {
                        Err("Malformed HTTP header".to_string())
                    }
                })?;
                Some(map)
            }
        };

        let body: Option<Box<dyn BodyDecoder>> = match headers {
            None => None,
            Some(ref headers) => {
                if let Some(length) = headers.get("Content-Length") {
                    // TODO: Handle isize::MAX and a max body size.
                    let length: usize = length
                        .parse()
                        .map_err(|_| "Content-Length is not a number")?;
                    Some(Box::new(Body::new(length, buf)))
                } else if let Some(encoding) =
                    headers.get("Transfer-Encoding").map(|h| h.to_lowercase())
                {
                    if encoding.as_str() == "chunked" {
                        Some(Box::new(ChunkedDecoder::new(buf)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };

        let (method, uri, version) = parse_request_line(&request_line)?;

        Ok(Request {
            headers,
            body,
            path: uri,
            query_string: "",
            http_version: version,
            method,
        })
    }
}

fn parse_request_line(
    request_line: &str,
) -> Result<(Method, String, HttpVersion), &'static str> {
    let parts: Vec<_> = request_line.split_ascii_whitespace().collect();

    if parts.len() != 3 {
        return Err("Invalid request line");
    }

    let method = Method::from_str(parts[0])?;
    let path = parts[1];
    let version = HttpVersion::from_str(parts[2])?;

    Ok((method, path.to_string(), version))
}
