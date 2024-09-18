use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
enum HttpVersion {
    V0_9,
    V1_0,
    V1_1,
    V2_0,
    V3_0,
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

fn handle_client(
    stream: &TcpStream,
) -> Result<(String, String, HttpVersion), String> {
    let buf = BufReader::new(stream);
    let mut lines = buf.lines();

    let request_line = lines
        .next()
        .ok_or("Expected request line")?
        .map_err(|_| "Couldn't get request line")?;

    Ok(parse_request_line(&request_line)?)
}

fn write_http<T: ?Sized + Write>(
    writer: &mut BufWriter<T>,
    headers: &str,
    content: &str,
) {
    let _headers = headers;
    if headers.is_empty() {
        let _headers = &format!("\r\n{headers}").to_string();
    }
    let length = content.len();
    let _ = writer.write_all(
        format!("HTTP/1.1 200 OK\r\nContent-Length: {length}{_headers}\r\n\r\n{content}")
            .as_bytes(),
    );
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000").unwrap();

    for stream in listener.incoming() {
        let stream = stream?;
        let mut writer = BufWriter::new(&stream);
        match handle_client(&stream) {
            Ok((verb, path, _)) => {
                write_http(
                    &mut writer,
                    "",
                    &format!("<html><h1>{verb} on {path}</h1></html>"),
                );
            }
            Err(err) => {
                write_http(&mut writer, "", &err);
            }
        }
    }

    Ok(())
}
