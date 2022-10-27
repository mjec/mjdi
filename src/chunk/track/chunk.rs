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
        concat_vecs!(
            Vec::<u8>::from(crate::chunk::ChunkType::Track),
            (payload_bytes.len() as u32).to_be_bytes(),
            payload_bytes
        )
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
    Midi(ChannelMessage),
    Sysex(SysexMessage),
    Meta(MetaMessage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelMessage {
    message: ChannelMessageWithoutChannel,
    channel: u8,
}

impl ChannelMessage {
    pub fn new(channel: u8, message: ChannelMessageWithoutChannel) -> Option<ChannelMessage> {
        if channel >= 16 {
            None
        } else {
            Some(Self { message, channel })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMessageWithoutChannel {
    Voice(VoiceMessage),
    Mode(ModeMessage),
}

backed_enum! {
  pub enum VoiceMessage(u8, VoiceMessageError) {
    NoteOff = 0x80,
    NoteOn = 0x90,
    PolyKeyPressure = 0xA0,
    ControlChange = 0xB0,
    ProgramChange = 0xC0,
    ChannelPressure = 0xD0,
    PitchBend = 0xE0,
  }
}

backed_enum! {
  pub enum ModeMessage(u8, ModeMessageError) {
    AllSoundOff = 120,
    ResetAllControllers = 121,
    LocalControl = 122,
    AllNotesOff =123,
    OmniOff = 124,
    OmniOn = 125,
    Mono = 126,
    Poly = 127,
  }
}

impl ModeMessage {
    fn to_be_bytes(&self, channel: u8) -> [u8; 2] {
        assert!(channel < 16);
        [VoiceMessage::ControlChange as u8 | channel, *self as u8]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysexMessage {
    length: Vlq,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tempo(u32);
impl Tempo {
    pub fn new(microseconds_per_quarter_note: u32) -> Option<Self> {
        if microseconds_per_quarter_note > (2 ^ 24) - 1 {
            None
        } else {
            Some(Self(microseconds_per_quarter_note))
        }
    }
}

backed_enum! {
  pub enum KeyType(u8, KeyTypeError) {
    Major = 0,
    Minor = 1,
  }
}

backed_enum! {
  pub enum SharpsOrFats(i8, SharpsOrFlatsError) {
    SevenFlats = -7,
    SixFlats = -6,
    FiveFlats = -5,
    FourFlats = -4,
    ThreeFlats = -3,
    TwoFlats = -2,
    OneFlat = -1,
    None = 0,
    OneSharp = 1,
    TwoSharps = 2,
    ThreeSharps = 3,
    FourSharps = 4,
    FiveSharps = 5,
    SixSharps = 6,
    SevenSharps = 7,
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaMessage {
    SequenceNumber(u16),
    TextEvent(String),
    CopyrightNotice(String),
    SequenceName(String),
    InstrumentName(String),
    Lyric(String),
    Marker(String),
    CuePoint(String),
    ChannelPrefix(u8),
    EndOfTrack,
    SetTempo(Tempo),
    SMPTEOffset {
        hour: u8,
        minute: u8,
        second: u8,
        frame: u8,
        hundredths_of_a_frame: u8,
    },
    TimeSignature {
        numerator: u8,
        /// Expressed as a negative power of two, i.e. denominator = 3 would represent 8th notes
        denominator: u8,
        /// the number of MIDI clocks in a metronome click
        cc: u8,
        ///  the number of notated 32 nd-notes in what MIDI thinks of as a quarter-note (24 MIDI clocks)
        bb: u8,
    },
    KeySignature {
        sharps_or_flats: SharpsOrFats,
        key_type: KeyType,
    },
    SequencerSpecificEvent(Vec<u8>),
}

impl From<&MetaMessage> for Vec<u8> {
    fn from(message: &MetaMessage) -> Self {
        match message {
            MetaMessage::SequenceNumber(n) => concat_vecs!(vec![0x00, 0x02], n.to_be_bytes()),
            MetaMessage::TextEvent(text) => {
                debug_assert!(u32::try_from(text.len()).unwrap() <= crate::vlq::MAX_REPRESENTABLE);
                concat_vecs!(
                    vec![0x01],
                    Vec::<u8>::from(&Vlq::try_from(text.len() as u32).unwrap()),
                    text.as_bytes()
                )
            }
            MetaMessage::CopyrightNotice(text) => {
                debug_assert!(u32::try_from(text.len()).unwrap() <= crate::vlq::MAX_REPRESENTABLE);
                concat_vecs!(
                    vec![0x02],
                    Vec::<u8>::from(&Vlq::try_from(text.len() as u32).unwrap()),
                    text.as_bytes()
                )
            }
            MetaMessage::SequenceName(text) => {
                debug_assert!(u32::try_from(text.len()).unwrap() <= crate::vlq::MAX_REPRESENTABLE);
                concat_vecs!(
                    vec![0x03],
                    Vec::<u8>::from(&Vlq::try_from(text.len() as u32).unwrap()),
                    text.as_bytes()
                )
            }
            MetaMessage::InstrumentName(text) => {
                debug_assert!(u32::try_from(text.len()).unwrap() <= crate::vlq::MAX_REPRESENTABLE);
                concat_vecs!(
                    vec![0x04],
                    Vec::<u8>::from(&Vlq::try_from(text.len() as u32).unwrap()),
                    text.as_bytes()
                )
            }
            MetaMessage::Lyric(text) => {
                debug_assert!(u32::try_from(text.len()).unwrap() <= crate::vlq::MAX_REPRESENTABLE);
                concat_vecs!(
                    vec![0x05],
                    Vec::<u8>::from(&Vlq::try_from(text.len() as u32).unwrap()),
                    text.as_bytes()
                )
            }
            MetaMessage::Marker(text) => {
                debug_assert!(u32::try_from(text.len()).unwrap() <= crate::vlq::MAX_REPRESENTABLE);
                concat_vecs!(
                    vec![0x06],
                    Vec::<u8>::from(&Vlq::try_from(text.len() as u32).unwrap()),
                    text.as_bytes()
                )
            }
            MetaMessage::CuePoint(text) => {
                debug_assert!(u32::try_from(text.len()).unwrap() <= crate::vlq::MAX_REPRESENTABLE);
                concat_vecs!(
                    vec![0x07],
                    Vec::<u8>::from(&Vlq::try_from(text.len() as u32).unwrap()),
                    text.as_bytes()
                )
            }
            MetaMessage::ChannelPrefix(ch) => {
                debug_assert!(*ch < 16);
                vec![0x20, 0x01, *ch]
            }
            MetaMessage::EndOfTrack => vec![0x2F, 0x00],
            MetaMessage::SetTempo(tempo) => {
                debug_assert!(tempo.0 < 2 ^ 24);
                concat_vecs!(vec![0x51, 0x03], &tempo.0.to_be_bytes()[1..])
            }
            MetaMessage::SMPTEOffset {
                hour,
                minute,
                second,
                frame,
                hundredths_of_a_frame,
            } => vec![
                0x54,
                0x05,
                *hour,
                *minute,
                *second,
                *frame,
                *hundredths_of_a_frame,
            ],
            MetaMessage::TimeSignature {
                numerator,
                denominator,
                cc,
                bb,
            } => vec![0x58, 0x04, *numerator, *denominator, *cc, *bb],
            MetaMessage::KeySignature {
                sharps_or_flats,
                key_type,
            } => vec![
                0x59,
                0x02,
                sharps_or_flats.clone() as u8,
                key_type.clone() as u8,
            ],
            MetaMessage::SequencerSpecificEvent(bytes) => {
                debug_assert!(u32::try_from(bytes.len()).unwrap() <= crate::vlq::MAX_REPRESENTABLE);
                concat_vecs!(
                    vec![0x7F],
                    Vec::<u8>::from(&Vlq::try_from(bytes.len() as u32).unwrap()),
                    bytes
                )
            }
        }
    }
}

impl From<&Event> for Vec<u8> {
    fn from(event: &Event) -> Self {
        match event {
            Event::Midi(_) => todo!(),
            Event::Sysex(_) => todo!(),
            Event::Meta(message) => {
                concat_vecs!(vec![0xFF], Vec::<u8>::from(message))
            }
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

// Putting this in cfg(false)for now, while I work on getting Event working.
// #[cfg(FALSE)]
#[cfg(test)]
mod tests {
    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;

    use super::*;

    impl Arbitrary for Event {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let channel_message = ChannelMessage::arbitrary(g);
            let sysex_message = SysexMessage::arbitrary(g);
            let meta_message = MetaMessage::arbitrary(g);
            g.choose(&[
                Event::Midi(channel_message),
                Event::Sysex(sysex_message),
                Event::Meta(meta_message),
            ])
            .expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
            .clone()
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                Event::Midi(x) => Box::new(x.shrink().map(Event::Midi)),
                Event::Sysex(x) => Box::new(x.shrink().map(Event::Sysex)),
                Event::Meta(x) => Box::new(x.shrink().map(Event::Meta)),
            }
        }
    }

    impl Arbitrary for MetaMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            todo!()
        }
    }

    impl Arbitrary for ChannelMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            todo!()
        }
    }

    impl Arbitrary for SysexMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let length = Vlq::arbitrary(g);

            SysexMessage {
                length,
                bytes: Vec::<u8>::arbitrary(&mut quickcheck::Gen::new(usize::try_from(u32::from(length)).expect("usize should be large enough for a length, hopefully. If not, who knows what could happen."))),
            }
        }
    }

    impl Arbitrary for MTrkEvent {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            MTrkEvent {
                delta_time: Vlq::arbitrary(g),
                event: Event::arbitrary(g),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            // TODO: add an actual shrinker here. The trouble is, we have to shrink in both dimensions at once (delta_time and event)
            quickcheck::empty_shrinker()
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
            let event_bytes = Vec::<u8>::from(&event.event);
            debug_assert!(event_bytes.len() == 1, "We're forcing this to be true but... at some point we might not? Who am I kidding, this assert is entirely here for symmetry purposes.");

            payload_bytes.extend(dt_bytes);
            payload_bytes.extend(event_bytes);
        }

        let mut expected: Vec<u8> = concat_vecs!(
          vec![b'M', b'T', b'r', b'k'],
          u32::try_from(payload_bytes.len()).expect(r#"Payload size must never exceed a u32, per the spec: "Each chunk has ... a 32-bit length" (p. 3)."#).to_be_bytes(),
          payload_bytes
        );
        assert_eq!(Vec::<u8>::from(Chunk::new(events)), expected);
    }
}
