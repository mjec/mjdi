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
    fn from(mtrk_event: MTrkEvent) -> Self {
        let mut result = Vec::<u8>::from(&mtrk_event.delta_time);
        result.extend(Vec::<u8>::from(&mtrk_event.event));
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Midi,
    Sysex,
    Meta,
}

impl From<&Event> for Vec<u8> {
    fn from(event: &Event) -> Self {
        #[allow(clippy::match_single_binding)]
        // we will need to come back to this and fill in the match arms
        match event {
            e => vec![*e as u8],
        }
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 10 {
            Err(ChunkError::SliceSize)
        } else if value[0..4] != Vec::<u8>::from(crate::chunk::ChunkType::Track) {
            Err(ChunkError::ChunkType)
        } else if u32::from_be_bytes([value[4], value[5], value[7], value[7]])
            != (value.len() - 8)
                .try_into()
                .map_err(|_| ChunkError::SliceSize)?
        {
            Err(ChunkError::ChunkLength)
        } else {
            Ok(Chunk {
                events: EventsList(todo!()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vlq::MAX_REPRESENTABLE as VLQ_MAX;
    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;

    use super::*;

    impl Arbitrary for Vlq {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut u32_value: u32 = u32::arbitrary(g);
            while u32_value > VLQ_MAX {
                u32_value = u32::arbitrary(g);
            }

            Vlq::try_from(u32_value).unwrap()
        }
    }

    impl Arbitrary for Event {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            g.choose(&[Event::Meta, Event::Midi, Event::Sysex]).expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose").clone()
        }
    }

    impl Arbitrary for MTrkEvent {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            MTrkEvent {
                delta_time: Vlq::arbitrary(g),
                event: Event::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn make_bytes(events: Vec<MTrkEvent>) {
        let mut payload_bytes: Vec<u8> = Vec::with_capacity((events.len() * 5) + 8);

        for event in &events {
            let dt_bytes = Vec::<u8>::from(&event.delta_time);
            debug_assert!(
                dt_bytes.len() <= 4,
                "We expect all VLQs to be at most four bytes, per the spec."
            );
            let event_bytes = vec![event.event as u8];
            debug_assert!(event_bytes.len() == 1, "We're forcing this to be true but... at some point we might not? Who am I kidding, this assert is entirely here for symmetry purposes.");

            payload_bytes.extend(dt_bytes);
            payload_bytes.extend(event_bytes);
        }

        let mut expected: Vec<u8> = vec![77, 84, 114, 107];
        expected.extend(u32::try_from(payload_bytes.len()).expect(r#"Payload size must never exceed a u32, per the spec: "Each chunk has ... a 32-bit length" (p. 3)."#).to_be_bytes());
        expected.extend(payload_bytes);
        assert_eq!(Vec::<u8>::from(Chunk::new(events)), expected);
    }
}
