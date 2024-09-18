use std::net::{TcpStream};
use std::io::{BufRead, BufReader};

#[derive(Debug)]
enum HttpVersion {
    V0_9,
    V1_0,
    V1_1,
    V2_0,
    V3_0,
}

pub struct Request<'a> {
    pub headers: &'a str,
    pub body: &'a str,
    pub path: String,
    pub query_string: &'a str,
}

impl<'a> Request<'a> {

    pub fn from(stream: &TcpStream) -> Result<Self, String> {
        let buf = BufReader::new(stream);
        let mut lines = buf.lines();

        let request_line = lines
            .next()
            .ok_or("Expected request line")?
            .map_err(|_| "Couldn't get request line")?;

        let (verb, uri, version) = parse_request_line(&request_line)?;

        Ok(Request {
            headers: "",
            body: "",
            path: uri,
            query_string: ""
        })
    }
}

fn parse_request_line(
    request_line: &str,
) -> Result<(String, String, HttpVersion), &'static str> {
    let parts: Vec<_> = request_line.split_ascii_whitespace().collect();

    if parts.len() != 3 {
        return Err("Invalid request line");
    }

    let verb = parts[0];
    let path = parts[1];
    let version = match parts[2] {
        "HTTP/0.9" => HttpVersion::V0_9,
        "HTTP/1.0" => HttpVersion::V1_0,
        "HTTP/1.1" => HttpVersion::V1_1,
        "HTTP/2.0" => HttpVersion::V2_0,
        "HTTP/3.0" => HttpVersion::V3_0,
        _ => return Err("Unknown HTTP version"),
    };
    Ok((verb.to_string(), path.to_string(), version))
}
