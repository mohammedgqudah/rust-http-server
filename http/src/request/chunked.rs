#![allow(clippy::all)]

use super::body::{BodyDecoder, Chunk};
use std::io::BufRead;

/// A Chunked Transfer Decoder
/// RFC: <https://datatracker.ietf.org/doc/html/rfc9112#section-7.1>
#[expect(clippy::module_name_repetitions)]
pub struct ChunkedDecoder<A: BufRead> {
    buf: A,
    stopped: bool,
}

#[allow(dead_code)]
impl<A: BufRead> ChunkedDecoder<A> {
    pub fn new(buf: A) -> Self {
        ChunkedDecoder {
            buf,
            stopped: false,
        }
    }
}

impl<A: BufRead> Iterator for ChunkedDecoder<A> {
    type Item = Result<Chunk, &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        // The decoder is stopped when an invalid chunk is received.
        if self.stopped {
            return None;
        }

        let mut line = String::new();

        if let Err(_) = self.buf.read_line(&mut line) {
            self.stopped = true;
            return Some(Err("Expected chunk size"));
        };

        let line = line.trim();

        // The end of chunks is denoted by 0, but this is just in case the client
        // messed up.
        if line.is_empty() {
            return None;
        }

        // Optionally read the chunk extension
        // https://datatracker.ietf.org/doc/html/rfc9112#section-7.1.1
        let (length, extension) = match line.split_once(';') {
            None => (line, ""),
            // trim the the first part because a BWS is allowed
            Some((length, extension)) => (length.trim(), extension.trim()),
        };

        #[allow(clippy::cast_possible_truncation)]
        let chunk_size = if let Ok(size) = u64::from_str_radix(length, 16) {
            size as usize
        } else {
            self.stopped = true;
            return Some(Err("Invalid chunk size"));
        };

        // If the chunk size is zero, mark the iterator as `stopped` but still return an empty chunk.
        // The last chunk signals the end of the stream, but may include an extension.
        if chunk_size == 0 {
            self.stopped = true;
        }

        let mut chunk = vec![0; chunk_size];

        if let Err(_) = self.buf.read_exact(&mut chunk) {
            self.stopped = true;
            return Some(Err("Expected a chunk"));
        };

        // Read CR LF
        let mut skip = [0; 2];
        self.buf.read_exact(&mut skip).ok()?;

        Some(Ok(Chunk {
            buf: chunk,
            extension: extension.to_string(),
        }))
    }
}
impl<A: BufRead> BodyDecoder for ChunkedDecoder<A> {}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn it_parses_each_chunk() {
        let expected = vec![
            Chunk {
                buf: "Hello".as_bytes().to_vec(),
                extension: "name_only;key1=value1".to_string(),
            },
            Chunk {
                buf: "This is exactly 18".as_bytes().to_vec(),
                extension: "one_key".to_string(),
            },
            Chunk {
                buf: vec![],
                extension: String::new(),
            },
        ];
        let body = String::from(
            r"5; name_only;key1=value1 
Hello
12; one_key
This is exactly 18
0

",
        )
        .replace('\n', "\r\n");
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
        .replace('\n', "\r\n");
        let cursor = Cursor::new(body.into_bytes());
        let mut buf = BufReader::new(cursor);
        let mut decoder = ChunkedDecoder::new(&mut buf);
        assert_eq!(
            Chunk {
                buf: "Hello".as_bytes().to_vec(),
                extension: String::new(),
            },
            #[expect(clippy::expect_used)]
            decoder
                .next()
                .expect("The first chunk was not parsed")
                .expect("The first chunk is valid")
        );
        assert_eq!(Err("Invalid chunk size"), decoder.next().unwrap());
        assert_eq!(None, decoder.next());
    }

    #[test]
    fn it_accepts_an_extension_for_the_last_chunk() {
        let expected = vec![
            Chunk {
                buf: "Hello".as_bytes().to_vec(),
                extension: "name_only;key1=value1".to_string(),
            },
            Chunk {
                buf: "This is exactly 18".as_bytes().to_vec(),
                extension: "one_key".to_string(),
            },
            Chunk {
                buf: vec![],
                extension: "progress=100".to_string(),
            },
        ];
        let body = String::from(
            r"5; name_only;key1=value1 
Hello
12; one_key
This is exactly 18
0;progress=100

",
        )
        .replace('\n', "\r\n");
        let cursor = Cursor::new(body.into_bytes());
        let mut buf = BufReader::new(cursor);
        let decoder = ChunkedDecoder::new(&mut buf);
        let chunks: Vec<Chunk> = decoder.map(|c| c.unwrap()).collect();
        assert_eq!(expected, chunks);
    }
}
