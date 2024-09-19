use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

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
    pub body: &'a str,
    pub path: String,
    pub query_string: &'a str,
    pub http_version: HttpVersion,
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

impl<'a> Request<'a> {
    pub fn from(stream: &TcpStream) -> Result<Self, String> {
        let buf = BufReader::new(stream);
        let mut lines = buf.lines();

        let request_line = lines
            .next()
            .ok_or("Expected request line")?
            .map_err(|_| "Couldn't get request line")?;

        let mut lines = lines
            .take_while(|line| !line.as_ref().unwrap().is_empty())
            .peekable();

        let headers = match lines.peek() {
            None => None,
            Some(_) => {
                let mut map: HashMap<String, String> = HashMap::new();
                lines.try_for_each(|line| -> Result<(), String> {
                    let binding = line.map_err(|_| "Expected a header".to_string())?;
                    let mut pair = binding.split(':');
                    if let (Some(key), Some(value)) = (pair.next(), pair.next()) {
                        map.insert(key.to_string(), value.trim().to_string());
                        Ok(())
                    } else {
                        Err("Malformed HTTP header".to_string())
                    }
                })?;
                Some(map)
            }
        };

        let (verb, uri, version) = parse_request_line(&request_line)?;

        Ok(Request {
            headers: headers,
            body: "",
            path: uri,
            query_string: "",
            http_version: version,
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
