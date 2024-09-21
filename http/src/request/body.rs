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
