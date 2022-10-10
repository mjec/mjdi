use std::num::NonZeroU16;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    format: Format,
    ntrks: NonZeroU16,
    division: Division,
}

impl Chunk {
    #[allow(dead_code)]
    pub fn new(format: Format, ntrks: NonZeroU16, division: Division) -> Self {
        Self {
            format,
            ntrks,
            division,
        }
    }
}

impl From<Chunk> for Vec<u8> {
    fn from(chunk: Chunk) -> Self {
        let mut result: Vec<u8> = Vec::new();
        result.extend(Vec::from(crate::chunk::ChunkType::Header));
        result.extend(6u32.to_be_bytes()); // length
        result.extend(Vec::from(chunk.format));
        result.extend(chunk.ntrks.get().to_be_bytes());
        result.extend(Vec::from(chunk.division));
        result
    }
}

impl IntoIterator for Chunk {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let output = vec![
            b'M',
            b'T',
            b'h',
            b'd', // Chunk type = header
            0,
            0,
            0,
            6, // Length (u32 big endian) = 6
            (self.format as u16).to_be_bytes()[0],
            (self.format as u16).to_be_bytes()[1],
            self.ntrks.get().to_be_bytes()[0],
            self.ntrks.get().to_be_bytes()[1],
            self.division.high_byte(),
            self.division.low_byte(),
        ];
        output.into_iter()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChunkError {
    SliceSize,
    ChunkType,
    ChunkLength,
    Format(FormatError),
    NumberOfTracks,
    Division(DivisionError),
}

impl From<FormatError> for ChunkError {
    fn from(e: FormatError) -> Self {
        ChunkError::Format(e)
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 14 {
            Err(ChunkError::SliceSize)
        } else if value[0..4] != [b'M', b'T', b'h', b'd'] {
            Err(ChunkError::ChunkType)
        } else if value[4..8] != [0, 0, 0, 6] {
            Err(ChunkError::ChunkLength)
        } else {
            Ok(Chunk {
                format: Format::try_from(u16::from_be_bytes([value[8], value[9]]))?,
                ntrks: NonZeroU16::new(u16::from_be_bytes([value[10], value[11]]))
                    .ok_or(ChunkError::NumberOfTracks)?,
                division: Division::try_from(u16::from_be_bytes([value[12], value[13]]))?,
            })
        }
    }
}
