pub mod header;
pub mod track;

pub enum ChunkType {
    Header,
    Track,
}

impl From<ChunkType> for Vec<u8> {
    fn from(t: ChunkType) -> Self {
        match t {
            ChunkType::Header => vec![b'M', b'T', b'h', b'd'],
            ChunkType::Track => vec![b'M', b'T', b'r', b'k'],
        }
    }
}
