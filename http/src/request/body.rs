use std::io::BufRead;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Chunk {
    pub buf: Vec<u8>,
    pub extension: String,
}

#[allow(dead_code)]
pub trait BodyDecoder: Iterator<Item = Result<Chunk, &'static str>> {
    fn all_bytes(&mut self) -> Vec<u8> {
        let mut res = Vec::new();

        while let Some(Ok(mut chunk)) = self.next() {
            res.append(&mut chunk.buf);
        }
        res
    }
}

/// A type used for requests with a known body size, explicitly indicated by the Content-Length
/// header.
pub struct Body<B: BufRead> {
    buf: B,
    length: usize,
    done: bool,
}

impl<B: BufRead> Body<B> {
    pub fn new(length: usize, buf: B) -> Self {
        Body {
            buf,
            length,
            done: false,
        }
    }
}

/// An iterator that is supposed to be used once, as the body is a single chunk with a known
/// size.
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
