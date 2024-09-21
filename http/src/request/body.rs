#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Chunk {
    pub buf: Vec<u8>,
    pub extension: String,
}

#[allow(dead_code)]
pub trait BodyDecoder: Iterator<Item = Result<Chunk, &'static str>> {}
