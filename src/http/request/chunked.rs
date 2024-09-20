use std::io::BufRead;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Chunk {
    pub buf: Vec<u8>,
    pub extension: String,
}

pub struct ChunkedDecoder<'a> {
    buf: &'a mut dyn BufRead,
    stopped: bool,
}

#[allow(dead_code)]
impl<'a> ChunkedDecoder<'a> {
    pub fn new(buf: &'a mut dyn BufRead) -> Self {
        ChunkedDecoder {
            buf: buf,
            stopped: false,
        }
    }
}

impl<'a> Iterator for ChunkedDecoder<'a> {
    type Item = Result<Chunk, &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        // The decoder is stopped when an invalid chunk is received.
        if self.stopped {
            return None;
        }

        let mut line = String::new();

        match self.buf.read_line(&mut line) {
            Ok(_) => {}
            Err(_) => return Some(Err("Expected chunk size")),
        };

        let line = line.trim();

        // The end of chunks is denoted by 0, but this is just in case the client
        // messed up.
        if line.is_empty() {
            return None;
        }

        // TODO: Start parsing optional chunk extensions
        let chunk_size = match u64::from_str_radix(line, 16) {
            Ok(size) => size as usize,
            Err(_) => {
                self.stopped = true;
                return Some(Err("Invalid chunk size"));
            }
        };

        if chunk_size == 0 {
            return None;
        }

        let mut chunk = vec![0; chunk_size];

        match self.buf.read_exact(&mut chunk) {
            Ok(_) => {}
            Err(_) => return Some(Err("Expected a chunk")),
        };

        let mut _skip = [0; 2];
        self.buf.read_exact(&mut _skip).ok()?;

        Some(Ok(Chunk {
            buf: chunk,
            extension: String::new(),
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn it_parses_each_chunk() {
        let expected = vec![
            Chunk {
                buf: "Hello".as_bytes().to_vec(),
                extension: String::new(),
            },
            Chunk {
                buf: "This is exactly 18".as_bytes().to_vec(),
                extension: String::new(),
            },
        ];
        let body = String::from(
            r"5
Hello
12
This is exactly 18
0

",
        )
        .replace("\n", "\r\n");
        let cursor = Cursor::new(body.into_bytes());
        let mut buf = BufReader::new(cursor);
        let decoder = ChunkedDecoder::new(&mut buf);
        let chunks: Vec<Chunk> = decoder.map(|c| c.unwrap()).collect();
        assert_eq!(expected, chunks);
    }

    #[test]
    fn it_does_not_accept_an_invalid_chunk_size() {
        let body = String::from(
            r"5
Hello
i_should_be_in_hex
invalid chunk
7
ignored
0

",
        )
        .replace("\n", "\r\n");
        let cursor = Cursor::new(body.into_bytes());
        let mut buf = BufReader::new(cursor);
        let mut decoder = ChunkedDecoder::new(&mut buf);
        assert_eq!(
            Chunk {
                buf: "Hello".as_bytes().to_vec(),
                extension: String::new(),
            },
            decoder
                .next()
                .expect("The first chunk was not parsed")
                .expect("The first chunk is valid")
        );
        assert_eq!(Err("Invalid chunk size"), decoder.next().unwrap());
        assert_eq!(None, decoder.next());
    }
}
