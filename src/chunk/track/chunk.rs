use std::{error::Error, fmt::Display, string::FromUtf8Error};

use crate::vlq::{Vlq, VlqError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub events: EventsList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventsList(pub Vec<MTrkEvent>);

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

impl From<&EventsList> for Vec<u8> {
    fn from(list: &EventsList) -> Self {
        let mut result: Self = Vec::new();
        for e in &list.0 {
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

impl From<&Chunk> for Vec<u8> {
    fn from(chunk: &Chunk) -> Self {
        let payload_bytes: Vec<u8> = Vec::from(&chunk.events);
        concat_vecs!(
            Vec::<u8>::from(crate::chunk::ChunkType::Track),
            (payload_bytes.len() as u32).to_be_bytes(),
            payload_bytes
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChunkError {
    NotEnoughBytes,
    ChunkType,
    MTrkEventError(MTrkEventError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MTrkEvent {
    delta_time: Vlq,
    event: Event,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MTrkEventError {
    NotEnoughBytes,
    DeltaTime(VlqError),
    Event(EventError),
}

impl From<VlqError> for MTrkEventError {
    fn from(e: VlqError) -> Self {
        Self::DeltaTime(e)
    }
}

impl From<EventError> for MTrkEventError {
    fn from(e: EventError) -> Self {
        Self::Event(e)
    }
}

impl From<MTrkEvent> for Vec<u8> {
    fn from(mtrk_event: MTrkEvent) -> Self {
        let mut result = Vec::<u8>::from(&mtrk_event.delta_time);
        result.extend(Vec::<u8>::from(&mtrk_event.event));
        result
    }
}

impl From<&MTrkEvent> for Vec<u8> {
    fn from(mtrk_event: &MTrkEvent) -> Self {
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

#[derive(Debug, PartialEq, Eq)]
pub enum EventError {
    NotEnoughBytes,
    MetaMessage(MetaMessageError),
    ModeMessage(ModeMessageError),
    VoiceMessageData(VoiceMessageDataError),
}

impl From<MetaMessageError> for EventError {
    fn from(e: MetaMessageError) -> Self {
        Self::MetaMessage(e)
    }
}

impl From<VoiceMessageDataError> for EventError {
    fn from(e: VoiceMessageDataError) -> Self {
        Self::VoiceMessageData(e)
    }
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct ChannelMessage {
//     message: ChannelMessageWithoutChannel,
//     channel: Channel,
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel(u8);

#[cfg(test)]
impl quickcheck::Arbitrary for Channel {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self::try_from(u8::arbitrary(g) & 0b0000_1111).expect("We just did a bitwise operation that guarantees we're passing in a u8 < 16, so that should be valid. If it's not, I can't guarantee this is still the correct operation.")
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(u8::shrink(&self.0).map(Channel))
    }
}

impl TryFrom<u8> for Channel {
    type Error = ChannelError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 16 {
            Ok(Channel(value))
        } else {
            Err(Self::Error::TooBig)
        }
    }
}

impl From<Channel> for u8 {
    fn from(value: Channel) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelError {
    TooBig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMessage {
    Voice {
        channel: Channel,
        data: VoiceMessageData,
    },
    Mode(ModeMessage),
}

impl From<&ChannelMessage> for Vec<u8> {
    fn from(value: &ChannelMessage) -> Self {
        match value {
            ChannelMessage::Mode(m) => vec![m.into()],
            ChannelMessage::Voice { channel, data } => data.to_be_bytes(channel),
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for U7 {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        g.choose(Vec::<u8>::from_iter(0..=127).as_slice())
            .expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
            .try_into()
            .expect("We're choosing numbers explicitly within the valid range")
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(u8::shrink(&self.0).map(|x| U7::try_from(x).expect("Since the previous value was valid, the new value should be no larger and therefore also valid")))
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for VoiceMessageData {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match VoiceMessage::arbitrary(g) {
            VoiceMessage::NoteOff => VoiceMessageData::NoteOff {
                note_number: U7::arbitrary(g),
                velocity: U7::arbitrary(g),
            },
            VoiceMessage::NoteOn => VoiceMessageData::NoteOn {
                note_number: U7::arbitrary(g),
                velocity: U7::arbitrary(g),
            },
            VoiceMessage::PolyKeyPressure => VoiceMessageData::PolyKeyPressure {
                note_number: U7::arbitrary(g),
                pressure: U7::arbitrary(g),
            },
            VoiceMessage::ControlChange => VoiceMessageData::ControlChange,
            VoiceMessage::ProgramChange => VoiceMessageData::ProgramChange {
                program_number: U7::arbitrary(g),
            },
            VoiceMessage::ChannelPressure => VoiceMessageData::ChannelPressure {
                pressure: U7::arbitrary(g),
            },
            VoiceMessage::PitchBend => VoiceMessageData::PitchBend {
                change: [U7::arbitrary(g), U7::arbitrary(g)],
            },
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for ChannelMessage {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let voice = |mut g: &mut quickcheck::Gen| ChannelMessage::Voice {
            channel: Channel::arbitrary(&mut g),
            data: VoiceMessageData::arbitrary(&mut g),
        };
        let mode =
            |mut g: &mut quickcheck::Gen| ChannelMessage::Mode(ModeMessage::arbitrary(&mut g));
        g.choose([
            voice,
            mode,
        ].as_slice())
        .expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
        (g)
    }
}

backed_enum! {
  enum VoiceMessage(u8, VoiceMessageError) {
    NoteOff = 0x80,
    NoteOn = 0x90,
    PolyKeyPressure = 0xA0,
    ControlChange = 0xB0,
    ProgramChange = 0xC0,
    ChannelPressure = 0xD0,
    PitchBend = 0xE0,
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceMessageData {
    NoteOff { note_number: U7, velocity: U7 },
    NoteOn { note_number: U7, velocity: U7 },
    PolyKeyPressure { note_number: U7, pressure: U7 },
    ControlChange,
    ProgramChange { program_number: U7 },
    ChannelPressure { pressure: U7 },
    PitchBend { change: [U7; 2] },
}

impl VoiceMessageData {
    /// Return value is guaranteed to be 0b????0000.
    fn get_first_nibble(&self) -> u8 {
        let result = match self {
            VoiceMessageData::NoteOff { .. } => VoiceMessage::NoteOff.into(),
            VoiceMessageData::NoteOn { .. } => VoiceMessage::NoteOn.into(),
            VoiceMessageData::PolyKeyPressure { .. } => VoiceMessage::PolyKeyPressure.into(),
            VoiceMessageData::ControlChange => VoiceMessage::ControlChange.into(),
            VoiceMessageData::ProgramChange { .. } => VoiceMessage::ProgramChange.into(),
            VoiceMessageData::ChannelPressure { .. } => VoiceMessage::ChannelPressure.into(),
            VoiceMessageData::PitchBend { .. } => VoiceMessage::PitchBend.into(),
        };
        assert!(
            result & 0b1111_0000 == result,
            "This should only return the high nibble!"
        );
        result
    }

    pub fn to_be_bytes(&self, channel: &Channel) -> Vec<u8> {
        let make_first_byte = |voice_message: VoiceMessage| {
            let voice_message_byte = u8::from(voice_message);
            let channel_byte = channel.0;
            assert!(
                voice_message_byte & 0b1111_0000 == voice_message_byte,
                "VoiceMessage values should only set the high nibble"
            );
            assert!(
                channel_byte & 0b0000_1111 == channel_byte,
                "Channel values should only set the low nibble"
            );
            voice_message_byte | channel_byte
        };
        match self {
            VoiceMessageData::NoteOff {
                note_number,
                velocity,
            } => vec![
                make_first_byte(VoiceMessage::NoteOff),
                u8::from(note_number),
                u8::from(velocity),
            ],
            VoiceMessageData::NoteOn {
                note_number,
                velocity,
            } => vec![
                make_first_byte(VoiceMessage::NoteOn),
                u8::from(note_number),
                u8::from(velocity),
            ],
            VoiceMessageData::PolyKeyPressure {
                note_number,
                pressure,
            } => vec![
                make_first_byte(VoiceMessage::PolyKeyPressure),
                u8::from(note_number),
                u8::from(pressure),
            ],
            VoiceMessageData::ControlChange => vec![
                make_first_byte(VoiceMessage::ControlChange),
                // todo!("More control change data required, surely"),
            ],
            VoiceMessageData::ProgramChange { program_number } => vec![
                make_first_byte(VoiceMessage::ProgramChange),
                u8::from(program_number),
            ],
            VoiceMessageData::ChannelPressure { pressure } => vec![
                make_first_byte(VoiceMessage::ChannelPressure),
                u8::from(pressure),
            ],
            VoiceMessageData::PitchBend { change } => vec![
                make_first_byte(VoiceMessage::PitchBend),
                u8::from(change[0]),
                u8::from(change[1]),
            ],
        }
    }
}

impl Parse for Vlq {
    type ParseError = VlqError;

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), Self::ParseError> {
        if bytes.is_empty() {
            return Err(VlqError::NotEnoughBytes);
        }
        let mut remainder = bytes;
        let mut result: u32 = 0;
        let mut i: u8 = 0;
        loop {
            if remainder.is_empty() {
                return Err(VlqError::NotEnoughBytes);
            }
            i += 1;
            result <<= 7;
            result += u32::from(remainder[0] & 0x7F);
            let should_continue = i < 4 && remainder[0] & 0x80 > 0;
            remainder = &remainder[1..];
            if !should_continue {
                break;
            }
        }
        Ok((Vlq::try_from(result)?, remainder))
    }
}

impl Parse for VoiceMessageData {
    type ParseError = VoiceMessageDataError;

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), Self::ParseError> {
        let get_byte_at = |index: usize| -> Result<u8, VoiceMessageDataError> {
            bytes
                .get(index)
                .ok_or(VoiceMessageDataError::NotEnoughBytes)
                .map(|x| *x)
        };

        let u7_from_byte_at = |index: usize,
                               error_constructor: fn(U7Error) -> VoiceMessageDataError|
         -> Result<U7, VoiceMessageDataError> {
            U7::try_from(get_byte_at(index)?).map_err(error_constructor)
        };

        match get_byte_at(0)? & 0b1111_0000 {
            x if x == VoiceMessage::NoteOff.into() => Ok((
                Self::NoteOff {
                    note_number: u7_from_byte_at(1, VoiceMessageDataError::NoteNumber)?,
                    velocity: u7_from_byte_at(2, VoiceMessageDataError::Velocity)?,
                },
                &bytes[3..], // Safe because bytes[2] exists, so bytes[3..] is at least []
            )),
            x if x == VoiceMessage::NoteOn.into() => Ok((
                Self::NoteOn {
                    note_number: u7_from_byte_at(1, VoiceMessageDataError::NoteNumber)?,
                    velocity: u7_from_byte_at(2, VoiceMessageDataError::Velocity)?,
                },
                &bytes[3..], // Safe because bytes[2] exists, so bytes[3..] is at least []
            )),
            x if x == VoiceMessage::PolyKeyPressure.into() => Ok((
                Self::PolyKeyPressure {
                    note_number: u7_from_byte_at(1, VoiceMessageDataError::NoteNumber)?,
                    pressure: u7_from_byte_at(2, VoiceMessageDataError::Pressure)?,
                },
                &bytes[3..], // Safe because bytes[2] exists, so bytes[3..] is at least []
            )),
            x if x == VoiceMessage::ControlChange.into() => todo!("VoiceMessage::ControlChange"),
            x if x == VoiceMessage::ProgramChange.into() => Ok((
                Self::ProgramChange {
                    program_number: u7_from_byte_at(1, VoiceMessageDataError::ProgramNumber)?,
                },
                &bytes[2..],
            )),
            x if x == VoiceMessage::ChannelPressure.into() => Ok((
                Self::ChannelPressure {
                    pressure: u7_from_byte_at(1, VoiceMessageDataError::Pressure)?,
                },
                &bytes[2..], // Safe because bytes[1] exists, so bytes[2..] is at least []
            )),
            x if x == VoiceMessage::PitchBend.into() => Ok((
                Self::PitchBend {
                    change: [
                        u7_from_byte_at(1, VoiceMessageDataError::PitchBend)?,
                        u7_from_byte_at(2, VoiceMessageDataError::PitchBend)?,
                    ],
                },
                &bytes[3..], // Safe because bytes[2] exists, so bytes[3..] is at least []
            )),
            _ => Err(VoiceMessageDataError::MessageType),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VoiceMessageDataError {
    NotEnoughBytes,
    MessageType,
    NoteNumber(U7Error),
    Velocity(U7Error),
    ProgramNumber(U7Error),
    Pressure(U7Error),
    PitchBend(U7Error),
}

impl Display for VoiceMessageDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for VoiceMessageDataError {}

impl std::ops::BitOr<Channel> for VoiceMessage {
    type Output = u8;

    fn bitor(self, rhs: Channel) -> Self::Output {
        u8::from(self) | rhs.0
    }
}

impl std::ops::BitOr<VoiceMessage> for Channel {
    type Output = u8;

    fn bitor(self, rhs: VoiceMessage) -> Self::Output {
        self.0 | u8::from(rhs)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct U7(u8);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum U7Error {
    Overflow,
}

impl TryFrom<u8> for U7 {
    type Error = U7Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 0x7f {
            Ok(Self(value))
        } else {
            Err(Self::Error::Overflow)
        }
    }
}

impl TryFrom<&u8> for U7 {
    type Error = U7Error;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        Self::try_from(*value)
    }
}

impl From<U7> for u8 {
    fn from(value: U7) -> Self {
        value.0
    }
}

impl From<&U7> for u8 {
    fn from(value: &U7) -> Self {
        value.0
    }
}

impl std::ops::BitOr<U7> for U7 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::try_from(self.0 | rhs.0)
            .expect("or-ing two seven bit numbers is guaranteed to produce a seven bit result")
    }
}

impl std::ops::BitOr<u8> for U7 {
    type Output = Option<Self>;

    fn bitor(self, rhs: u8) -> Self::Output {
        Self::try_from(self.0 | rhs).ok()
    }
}

impl std::ops::BitOr<U7> for u8 {
    type Output = u8;

    fn bitor(self, rhs: U7) -> Self::Output {
        self | rhs.0
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
  } {
    NotEnoughBytes,
  }
}

impl ModeMessage {
    fn to_be_bytes(self, channel: u8) -> [u8; 2] {
        assert!(channel < 16);
        [VoiceMessage::ControlChange as u8 | channel, self.into()]
    }
}

impl Parse for ModeMessage {
    type ParseError = ModeMessageError;

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), Self::ParseError> {
        let message =
            ModeMessage::try_from(*bytes.first().ok_or(ModeMessageError::NotEnoughBytes)?)?;
        Ok((message, &bytes[1..]))
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
    const MAX_BIT_SIZE: usize = 24;
    pub fn new(microseconds_per_quarter_note: u32) -> Option<Self> {
        if microseconds_per_quarter_note > (1 << Self::MAX_BIT_SIZE) - 1 {
            None
        } else {
            Some(Self(microseconds_per_quarter_note))
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Tempo {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        loop {
            if let Some(x) = Self::new(u32::arbitrary(g)) {
                return x;
            }
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(u32::shrink(&self.0).map(|x| {
            Self::new(x).expect("x should always be decreasing, because we called shrink")
        }))
    }
}

backed_enum! {
  pub enum KeyType(u8, KeyTypeError) {
    Major = 0,
    Minor = 1,
  }
}

backed_enum! {
  pub enum SharpsOrFlats(i8, SharpsOrFlatsError) {
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
        sharps_or_flats: SharpsOrFlats,
        key_type: KeyType,
    },
    SequencerSpecificEvent(Vec<u8>),
}

impl Parse for MetaMessage {
    type ParseError = MetaMessageError;

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), Self::ParseError> {
        if bytes.len() < 2 {
            return Err(MetaMessageError::NotEnoughBytes);
        }

        let extract_text_event = |ctor: fn(String) -> Self| {
            let len = usize::from(bytes[1]);
            if bytes.len() < 2 + len {
                return Err(MetaMessageError::NotEnoughBytes);
            }
            Ok((
                ctor(String::from_utf8(bytes[2..(2 + len)].to_vec())?),
                &bytes[2 + len..],
            ))
        };

        if bytes[0] == 0x00 && bytes[1] == 0x02 {
            Ok((
                Self::SequenceNumber(u16::from_be_bytes([
                    *bytes.get(2).ok_or(MetaMessageError::NotEnoughBytes)?,
                    *bytes.get(3).ok_or(MetaMessageError::NotEnoughBytes)?,
                ])),
                &bytes[4..],
            ))
        } else if bytes[0] == 0x01 {
            extract_text_event(Self::TextEvent)
        } else if bytes[0] == 0x02 {
            extract_text_event(Self::CopyrightNotice)
        } else if bytes[0] == 0x03 {
            extract_text_event(Self::SequenceName)
        } else if bytes[0] == 0x04 {
            extract_text_event(Self::InstrumentName)
        } else if bytes[0] == 0x05 {
            extract_text_event(Self::Lyric)
        } else if bytes[0] == 0x06 {
            extract_text_event(Self::Marker)
        } else if bytes[0] == 0x07 {
            extract_text_event(Self::CuePoint)
        } else if bytes[0] == 0x20 && bytes[1] == 0x01 {
            Ok((
                Self::ChannelPrefix(*bytes.get(2).ok_or(MetaMessageError::NotEnoughBytes)?),
                &bytes[3..],
            ))
        } else if bytes[0] == 0x2F && bytes[1] == 0x00 {
            Ok((Self::EndOfTrack, &bytes[2..]))
        } else if bytes[0] == 0x51 && bytes[1] == 0x03 {
            if bytes.len() < 5 {
                Err(MetaMessageError::NotEnoughBytes)
            } else {
                Ok((
                    Self::SetTempo(
                        Tempo::new(u32::from_be_bytes([0x00, bytes[2], bytes[3], bytes[4]]))
                            .expect("Top byte is zero"),
                    ),
                    &bytes[5..],
                ))
            }
        } else if bytes[0] == 0x54 && bytes[1] == 0x05 {
            if bytes.len() < 7 {
                Err(MetaMessageError::NotEnoughBytes)
            } else {
                Ok((
                    Self::SMPTEOffset {
                        hour: bytes[2],
                        minute: bytes[3],
                        second: bytes[4],
                        frame: bytes[5],
                        hundredths_of_a_frame: bytes[6],
                    },
                    &bytes[7..],
                ))
            }
        } else if bytes[0] == 0x58 && bytes[1] == 0x04 {
            if bytes.len() < 6 {
                Err(MetaMessageError::NotEnoughBytes)
            } else {
                Ok((
                    Self::TimeSignature {
                        numerator: bytes[2],
                        denominator: bytes[3],
                        cc: bytes[4],
                        bb: bytes[5],
                    },
                    &bytes[6..],
                ))
            }
        } else if bytes[0] == 0x59 && bytes[1] == 0x02 {
            if bytes.len() < 4 {
                Err(MetaMessageError::NotEnoughBytes)
            } else {
                Ok((
                    Self::KeySignature {
                        sharps_or_flats: SharpsOrFlats::try_from(bytes[2] as i8)?,
                        key_type: KeyType::try_from(bytes[3])?,
                    },
                    &bytes[4..],
                ))
            }
        } else if bytes[0] == 0x7F {
            let len = usize::from(bytes[1]);
            if bytes.len() < 2 + len {
                return Err(MetaMessageError::NotEnoughBytes);
            }
            Ok((
                Self::SequencerSpecificEvent(Vec::from(&bytes[2..2 + len])),
                &bytes[2 + len..],
            ))
        } else {
            Err(MetaMessageError::Unrecognized([bytes[0], bytes[1]]))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MetaMessageError {
    NotEnoughBytes,
    Unrecognized([u8; 2]),
    TextIsNotUtf8(FromUtf8Error),
    KeyType(KeyTypeError),
    SharpsOrFlats(SharpsOrFlatsError),
}

impl From<KeyTypeError> for MetaMessageError {
    fn from(value: KeyTypeError) -> Self {
        Self::KeyType(value)
    }
}

impl From<SharpsOrFlatsError> for MetaMessageError {
    fn from(value: SharpsOrFlatsError) -> Self {
        Self::SharpsOrFlats(value)
    }
}

impl From<FromUtf8Error> for MetaMessageError {
    fn from(value: FromUtf8Error) -> Self {
        Self::TextIsNotUtf8(value)
    }
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
                debug_assert!(tempo.0 < (1 << 24));
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
            } => vec![0x59, 0x02, *sharps_or_flats as u8, *key_type as u8],
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
            Event::Midi(channel_message) => channel_message.into(),
            Event::Sysex(SysexMessage { length, bytes }) => {
                concat_vecs!(vec![0xF0], Vec::<u8>::from(length), bytes)
            }
            Event::Meta(message) => {
                concat_vecs!(vec![0xFF], Vec::<u8>::from(message))
            }
        }
    }
}

impl Parse for Chunk {
    type ParseError = ChunkError;

    fn parse(value: &[u8]) -> Result<(Self, &[u8]), Self::ParseError> {
        if value.len() < 10 {
            return Err(ChunkError::NotEnoughBytes);
        } else if value[0..4] != Vec::<u8>::from(crate::chunk::ChunkType::Track) {
            return Err(ChunkError::ChunkType);
        }

        let chunk_length =
            usize::try_from(u32::from_be_bytes([value[4], value[5], value[6], value[7]]))
                .expect("usize >= u32");
        if value.len() - 8 < chunk_length {
            return Err(ChunkError::NotEnoughBytes);
        }

        let mut events = EventsList(Vec::new());
        let mut remainder = &value[8..8 + chunk_length];
        while !remainder.is_empty() {
            let event: MTrkEvent;
            (event, remainder) = MTrkEvent::parse(remainder)?;
            events.0.push(event);
        }
        Ok((Chunk { events }, &value[8 + chunk_length..]))
    }
}

trait Parse
where
    Self: Sized,
{
    type ParseError;

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), Self::ParseError>;
}

impl Parse for MTrkEvent {
    type ParseError = MTrkEventError;

    fn parse(bytes: &[u8]) -> Result<(MTrkEvent, &[u8]), MTrkEventError> {
        if bytes.len() < 4 {
            return Err(MTrkEventError::NotEnoughBytes);
        }
        let (delta_time, remainder) = Vlq::parse(bytes)?;
        let (event, remainder) = Event::parse(remainder)?;
        Ok((MTrkEvent { delta_time, event }, remainder))
    }
}

impl Parse for Event {
    type ParseError = EventError;

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), Self::ParseError> {
        if bytes.is_empty() {
            Err(EventError::NotEnoughBytes)
        } else if bytes[0] == 0xF0 {
            todo!()
        } else if bytes[0] == 0xF7 {
            todo!()
        } else if bytes[0] == 0xFF {
            let (message, remainder) = MetaMessage::parse(&bytes[1..])?;
            Ok((Event::Meta(message), remainder))
        } else if let Ok(message) = VoiceMessage::try_from(bytes[0] & 0b1111_0000) {
            let channel = Channel::try_from(bytes[0] & 0b0000_1111).expect("We just did a bitwise operation that guarantees we're passing in a u8 < 16, so that should be valid. If it's not, I can't guarantee this is still the correct operation.");
            let (data, remainder) = VoiceMessageData::parse(bytes)?;
            assert!(u8::from(message) == data.get_first_nibble(), "We're really using VoiceMessage::try_from() as a marker check here; if this isn't true, I can no longer guarantee this is the correct operation.");
            Ok((
                Event::Midi(ChannelMessage::Voice { channel, data }),
                remainder,
            ))
        } else {
            let (message, remainder) = ModeMessage::parse(bytes)?;
            Ok((Event::Midi(ChannelMessage::Mode(message)), remainder))
        }
    }
}

impl From<ModeMessageError> for EventError {
    fn from(e: ModeMessageError) -> Self {
        Self::ModeMessage(e)
    }
}

impl From<MTrkEventError> for ChunkError {
    fn from(e: MTrkEventError) -> Self {
        Self::MTrkEventError(e)
    }
}

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
            let sequence_number =
                |mut g: &mut quickcheck::Gen| MetaMessage::SequenceNumber(u16::arbitrary(&mut g));
            let text_event =
                |mut g: &mut quickcheck::Gen| MetaMessage::TextEvent(String::arbitrary(&mut g));
            let copyright_notice = |mut g: &mut quickcheck::Gen| {
                MetaMessage::CopyrightNotice(String::arbitrary(&mut g))
            };
            let sequence_name =
                |mut g: &mut quickcheck::Gen| MetaMessage::SequenceName(String::arbitrary(&mut g));
            let instrument_name = |mut g: &mut quickcheck::Gen| {
                MetaMessage::InstrumentName(String::arbitrary(&mut g))
            };
            let lyric = |mut g: &mut quickcheck::Gen| MetaMessage::Lyric(String::arbitrary(&mut g));
            let marker =
                |mut g: &mut quickcheck::Gen| MetaMessage::Marker(String::arbitrary(&mut g));
            let cue_point =
                |mut g: &mut quickcheck::Gen| MetaMessage::CuePoint(String::arbitrary(&mut g));
            let channel_prefix = |mut g: &mut quickcheck::Gen| {
                let mut prefix = u8::arbitrary(&mut g);
                while prefix >= 16 {
                    prefix = u8::arbitrary(&mut g);
                }
                MetaMessage::ChannelPrefix(prefix)
            };
            let end_of_track = |_: &mut quickcheck::Gen| MetaMessage::EndOfTrack;
            let set_tempo =
                |mut g: &mut quickcheck::Gen| MetaMessage::SetTempo(Tempo::arbitrary(&mut g));
            // TODO: SMPTEOffset is a little _too_ arbitrary; e.g. hundredths_of_a_frame should really never exceed 99...
            let smpte_offset = |mut g: &mut quickcheck::Gen| MetaMessage::SMPTEOffset {
                hour: u8::arbitrary(&mut g),
                minute: u8::arbitrary(&mut g),
                second: u8::arbitrary(&mut g),
                frame: u8::arbitrary(&mut g),
                hundredths_of_a_frame: u8::arbitrary(&mut g),
            };
            // TODO: TimeSignature is a little _too_ arbitrary; e.g. a denominator of 255 would represent a 2^-255th note
            let time_signature = |mut g: &mut quickcheck::Gen| MetaMessage::TimeSignature {
                numerator: u8::arbitrary(&mut g),
                denominator: u8::arbitrary(&mut g),
                cc: u8::arbitrary(&mut g),
                bb: u8::arbitrary(&mut g),
            };
            let key_signature = |mut g: &mut quickcheck::Gen| MetaMessage::KeySignature {
                sharps_or_flats: SharpsOrFlats::arbitrary(&mut g),
                key_type: KeyType::arbitrary(&mut g),
            };
            let sequencer_specific_event = |mut g: &mut quickcheck::Gen| {
                MetaMessage::SequencerSpecificEvent(Vec::<u8>::arbitrary(&mut g))
            };
            g.choose([
                sequence_number,
                text_event,
                copyright_notice,
                sequence_name,
                instrument_name,
                lyric,
                marker,
                cue_point,
                channel_prefix,
                end_of_track,
                set_tempo,
                smpte_offset,
                time_signature,
                key_signature,
                sequencer_specific_event,
            ].as_slice())
            .expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
            (g)
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                MetaMessage::SequenceNumber(x) => {
                    Box::new(x.shrink().map(MetaMessage::SequenceNumber))
                }
                MetaMessage::TextEvent(x) => Box::new(x.shrink().map(MetaMessage::TextEvent)),
                MetaMessage::CopyrightNotice(x) => {
                    Box::new(x.shrink().map(MetaMessage::CopyrightNotice))
                }
                MetaMessage::SequenceName(x) => Box::new(x.shrink().map(MetaMessage::SequenceName)),
                MetaMessage::InstrumentName(x) => {
                    Box::new(x.shrink().map(MetaMessage::InstrumentName))
                }
                MetaMessage::Lyric(x) => Box::new(x.shrink().map(MetaMessage::Lyric)),
                MetaMessage::Marker(x) => Box::new(x.shrink().map(MetaMessage::Marker)),
                MetaMessage::CuePoint(x) => Box::new(x.shrink().map(MetaMessage::CuePoint)),
                MetaMessage::ChannelPrefix(x) => {
                    Box::new(x.shrink().map(MetaMessage::ChannelPrefix))
                }
                MetaMessage::EndOfTrack => Box::new(std::iter::once(MetaMessage::EndOfTrack)),
                MetaMessage::SetTempo(x) => Box::new(x.shrink().map(MetaMessage::SetTempo)),
                MetaMessage::SMPTEOffset { .. } => quickcheck::empty_shrinker(),
                MetaMessage::TimeSignature { .. } => quickcheck::empty_shrinker(),
                MetaMessage::KeySignature { .. } => quickcheck::empty_shrinker(),
                MetaMessage::SequencerSpecificEvent(x) => {
                    Box::new(x.shrink().map(MetaMessage::SequencerSpecificEvent))
                }
            }
        }
    }

    impl Arbitrary for SysexMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let bytes = Vec::<u8>::arbitrary(&mut quickcheck::Gen::new(
                if g.size() < crate::vlq::MAX_REPRESENTABLE as usize {
                    g.size()
                } else {
                    crate::vlq::MAX_REPRESENTABLE as usize
                },
            ));
            let length = Vlq::try_from(bytes.len() as u32)
                .expect("This should always be smaller than MAX_REPRESENTABLE");

            SysexMessage { length, bytes }
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

    impl Arbitrary for Chunk {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Chunk {
                events: EventsList(Vec::<MTrkEvent>::arbitrary(g)),
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
            let event_bytes = Vec::<u8>::from(&event.event);
            // debug_assert!(event_bytes.len() == 1, "We're forcing this to be true but... at some point we might not? Who am I kidding, this assert is entirely here for symmetry purposes.");

            payload_bytes.extend(dt_bytes);
            payload_bytes.extend(event_bytes);
        }

        let expected: Vec<u8> = concat_vecs!(
          vec![b'M', b'T', b'r', b'k'],
          u32::try_from(payload_bytes.len()).expect(r#"Payload size must never exceed a u32, per the spec: "Each chunk has ... a 32-bit length" (p. 3)."#).to_be_bytes(),
          payload_bytes
        );
        assert_eq!(Vec::<u8>::from(Chunk::new(events)), expected);
    }

    #[test]
    fn chunks_from_brandenburg_concerto() {
        let mut remainder: &[u8] = &crate::test_data::brandenburg::DATA[14..];
        let mut result: Vec<Chunk> = Vec::new();
        while !remainder.is_empty() {
            let chunk: Chunk;
            (chunk, remainder) =
                Chunk::parse(remainder).expect("Should successfully parse the file");
            result.push(chunk);
        }

        assert!(true, "We managed to parse the file without crashing");
    }

    #[quickcheck]
    fn round_trip_to_bytes(chunk: Chunk) {
        if false {
            todo!("Still some significant work involved in pushing this forward");
            let chunk_bytes = Vec::<u8>::from(&chunk);
            let (parsed_chunk, remainder) =
                Chunk::parse(&chunk_bytes).expect("Chunk should be parseable");
            assert_eq!(chunk, parsed_chunk);
            assert!(remainder.is_empty());
        }
    }
}
