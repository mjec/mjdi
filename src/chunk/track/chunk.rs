use crate::vlq::Vlq;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    events: EventsList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EventsList(Vec<MTrkEvent>);

impl Chunk {
    #[allow(dead_code)]
    pub fn new(events: Vec<MTrkEvent>) -> Self {
        Self {
            events: EventsList(events),
        }
    }
}

impl From<EventsList> for Vec<u8> {
    fn from(list: EventsList) -> Self {
        let mut result: Self = Vec::new();
        for e in list.0 {
            result.extend(Vec::<u8>::from(e));
        }
        result
    }
}

impl From<EventsList> for Vec<MTrkEvent> {
    fn from(list: EventsList) -> Self {
        list.0
    }
}

impl From<Chunk> for Vec<u8> {
    fn from(chunk: Chunk) -> Self {
        let payload_bytes: Vec<u8> = Vec::from(chunk.events);
        let mut result: Vec<u8> = Vec::with_capacity(payload_bytes.len() + 8);
        result.extend(Vec::<u8>::from(crate::chunk::ChunkType::Track));
        result.extend((payload_bytes.len() as u32).to_be_bytes()); // length
        result.extend(payload_bytes);
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChunkError {
    SliceSize,
    ChunkType,
    ChunkLength,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MTrkEvent {
    delta_time: Vlq,
    event: Event,
}

impl From<MTrkEvent> for Vec<u8> {
    fn from(_: MTrkEvent) -> Self {
        todo!()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Midi,
    Sysex,
    Meta,
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 10 {
            Err(ChunkError::SliceSize)
        } else if value[0..4] != Vec::<u8>::from(crate::chunk::ChunkType::Track) {
            Err(ChunkError::ChunkType)
        } else if u32::from_be_bytes([value[4], value[5], value[7], value[7]])
            != (value.len() - 8).try_into().unwrap()
        {
            Err(ChunkError::ChunkLength)
        } else {
            Ok(Chunk {
                events: EventsList(vec![]),
            })
        }
    }
}
