use super::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NTracks(u16);

impl NTracks {
    pub fn to_be_bytes(self: &Self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NTracksError {
    NumberOfTracksMustBeGreaterThanZero,
}

impl TryFrom<u16> for NTracks {
    type Error = NTracksError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Err(NTracksError::NumberOfTracksMustBeGreaterThanZero),
            n => Ok(Self(n)),
        }
    }
}

impl From<NTracksError> for ChunkError {
    fn from(e: NTracksError) -> Self {
        ChunkError::InvalidNumberOfTracks(e)
    }
}
