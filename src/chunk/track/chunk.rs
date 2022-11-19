use std::{error::Error, fmt::Display};

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
    ChunkLength { expected: u32 },
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
            &ChannelMessage::Mode(ModeMessage::AllNotesOff) => vec![ModeMessage::AllNotesOff as u8],
            _ => todo!("lol"),
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
            VoiceMessage::NoteOn => VoiceMessageData::NoteOn,
            VoiceMessage::PolyKeyPressure => VoiceMessageData::PolyKeyPressure,
            VoiceMessage::ControlChange => VoiceMessageData::ControlChange,
            VoiceMessage::ProgramChange => VoiceMessageData::ProgramChange,
            VoiceMessage::ChannelPressure => VoiceMessageData::ChannelPressure,
            VoiceMessage::PitchBend => VoiceMessageData::PitchBend,
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for ChannelMessage {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let voice = ChannelMessage::Voice {
            channel: Channel::arbitrary(g),
            data: VoiceMessageData::arbitrary(g),
        };
        let mode = ChannelMessage::Mode(ModeMessage::arbitrary(g));
        g.choose(&[
            voice,
            mode,
        ])
        .expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
        .clone()
    }

    // fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
    //     match self {
    //         ChannelMessageWithoutChannel::Voice(x) => {
    //             Box::new(x.shrink().map(ChannelMessageWithoutChannel::Voice))
    //         }
    //         ChannelMessageWithoutChannel::Mode(x) => {
    //             Box::new(x.shrink().map(ChannelMessageWithoutChannel::Mode))
    //         }
    //     }
    // }
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

impl VoiceMessage {
    const NOTE_OFF: u8 = 0x80;

    fn to_be_bytes(&self, channel: u8) -> [u8; 2] {
        // signature is  wrong
        assert!(channel < 16);
        todo!("VoiceMessage::to_be_bytes");
        [*self as u8 | channel, *self as u8] // Not even close to right
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceMessageData {
    NoteOff { note_number: U7, velocity: U7 },
    NoteOn,
    PolyKeyPressure,
    ControlChange,
    ProgramChange,
    ChannelPressure,
    PitchBend,
}

impl VoiceMessageData {
    /// Return value is guaranteed to be 0b????0000.
    pub fn get_first_nibble(&self) -> u8 {
        let result = match self {
            VoiceMessageData::NoteOff { .. } => VoiceMessage::NoteOff.into(),
            VoiceMessageData::NoteOn => VoiceMessage::NoteOn.into(),
            VoiceMessageData::PolyKeyPressure => VoiceMessage::PolyKeyPressure.into(),
            VoiceMessageData::ControlChange => VoiceMessage::ControlChange.into(),
            VoiceMessageData::ProgramChange => VoiceMessage::ProgramChange.into(),
            VoiceMessageData::ChannelPressure => VoiceMessage::ChannelPressure.into(),
            VoiceMessageData::PitchBend => VoiceMessage::PitchBend.into(),
        };
        assert!(
            result & 0b1111_0000 == result,
            "This should only return the high nibble!"
        );
        result
    }

    pub fn to_be_bytes(&self, channel: Channel) -> Vec<u8> {
        let mut result = Vec::with_capacity(4);
        todo!();
        result
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
                               error_construtor: fn(U7Error) -> VoiceMessageDataError|
         -> Result<U7, VoiceMessageDataError> {
            U7::try_from(get_byte_at(index)?).map_err(error_construtor)
        };

        match get_byte_at(0)? & 0b1111_0000 {
            x if x == VoiceMessage::NoteOff.into() => Ok((
                Self::NoteOff {
                    note_number: u7_from_byte_at(1, VoiceMessageDataError::NoteNumber)?,
                    velocity: u7_from_byte_at(2, VoiceMessageDataError::Velocity)?,
                },
                &bytes[3..], // Safe because bytes[2] exists, so bytes[3..] is at least []
            )),
            x if x == VoiceMessage::NoteOn.into() => todo!("VoiceMessage::NoteOn"),
            x if x == VoiceMessage::PolyKeyPressure.into() => {
                todo!("VoiceMessage::PolyKeyPressure")
            }
            x if x == VoiceMessage::ControlChange.into() => todo!("VoiceMessage::ControlChange"),
            x if x == VoiceMessage::ProgramChange.into() => todo!("VoiceMessage::ProgramChange"),
            x if x == VoiceMessage::ChannelPressure.into() => {
                todo!("VoiceMessage::ChannelPressure")
            }
            x if x == VoiceMessage::PitchBend.into() => todo!("VoiceMessage::PitchBend"),
            otherwise => Err(VoiceMessageDataError::MessageType),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VoiceMessageDataError {
    NotEnoughBytes,
    MessageType,
    NoteNumber(U7Error),
    Velocity(U7Error),
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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
    fn to_be_bytes(&self, channel: u8) -> [u8; 2] {
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
        if microseconds_per_quarter_note
            > (2 ^ u32::try_from(Self::MAX_BIT_SIZE).expect("This will always fit in a u32, c'mon"))
                - 1
        {
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
            if let Some(x) = Self::new(u32::arbitrary(&mut quickcheck::Gen::new(
                if g.size() < Tempo::MAX_BIT_SIZE {
                    g.size()
                } else {
                    Tempo::MAX_BIT_SIZE
                },
            ))) {
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
        if bytes.len() < 3 {
            Err(MetaMessageError::NotEnoughBytes)
        } else {
            todo!()
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MetaMessageError {
    NotEnoughBytes,
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

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 10 {
            Err(ChunkError::SliceSize)
        } else if value[0..4] != Vec::<u8>::from(crate::chunk::ChunkType::Track) {
            Err(ChunkError::ChunkType)
        } else if u32::from_be_bytes([value[4], value[5], value[6], value[7]])
            != (value.len() - 8)
                .try_into()
                .map_err(|_| ChunkError::SliceSize)?
        {
            Err(ChunkError::ChunkLength {
                expected: 8 + u32::from_be_bytes([value[4], value[5], value[6], value[7]]),
            })
        } else {
            let mut events = EventsList(Vec::new());
            let mut remainder = &value[8..];
            loop {
                events.0.push(match Option::<MTrkEvent>::parse(remainder)? {
                    (Some(event), inner_remainder) => {
                        remainder = inner_remainder;
                        event
                    }
                    (None, _) => break,
                });
            }
            Ok(Chunk { events })
        }
    }
}

trait Parse
where
    Self: Sized,
{
    type ParseError;

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), Self::ParseError>;
}

impl Parse for Option<MTrkEvent> {
    type ParseError = MTrkEventError;

    fn parse(bytes: &[u8]) -> Result<(Option<MTrkEvent>, &[u8]), MTrkEventError> {
        if bytes.is_empty() {
            return Ok((None, bytes));
        }
        if bytes.len() < 5 {
            return Err(MTrkEventError::NotEnoughBytes);
        }
        let delta_time =
            Vlq::try_from(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))?;
        let (event, remainder) = Event::parse(&bytes[4..])?;
        Ok((Some(MTrkEvent { delta_time, event }), remainder))
    }
}

impl Parse for Event {
    type ParseError = EventError;

    fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), Self::ParseError> {
        if bytes.len() < 1 {
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
            debug_assert!(u8::from(message) == data.get_first_nibble(), "We're really using VoiceMessage::try_from() as a marker check here; if this isn't true, I can no longer guarantee this is the correct operation.");
            Ok((
                Event::Midi(ChannelMessage::Voice { channel, data }),
                remainder,
            ))
        } else {
            let (message, remainder) = ModeMessage::parse(bytes)?;
            Ok((Event::Midi(ChannelMessage::Mode(message)), remainder))
            // dbg!(bytes);
            // // Event::Midi(ChannelMessage { channel: 1 , message: ChannelMessageWithoutChannel::Mode(ModeMessage::AllNotesOff)})
            // todo!("Need to implement all the message parsers now, starting with channel modes on p 6 of the detailed spec pdf")
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
            let sequence_number = MetaMessage::SequenceNumber(u16::arbitrary(g));
            let text_event = MetaMessage::TextEvent(String::arbitrary(g));
            let copyright_notice = MetaMessage::CopyrightNotice(String::arbitrary(g));
            let sequence_name = MetaMessage::SequenceName(String::arbitrary(g));
            let instrument_name = MetaMessage::InstrumentName(String::arbitrary(g));
            let lyric = MetaMessage::Lyric(String::arbitrary(g));
            let marker = MetaMessage::Marker(String::arbitrary(g));
            let cue_point = MetaMessage::CuePoint(String::arbitrary(g));
            let channel_prefix = {
                let mut prefix = u8::arbitrary(g);
                while prefix >= 16 {
                    prefix = u8::arbitrary(g);
                }
                MetaMessage::ChannelPrefix(prefix)
            };
            let end_of_track = MetaMessage::EndOfTrack;
            let set_tempo = MetaMessage::SetTempo(Tempo::arbitrary(g));
            // TODO: SMPTEOffset is a little _too_ arbitrary; e.g. hundredths_of_a_frame should really never exceed 99...
            let smpte_offset = MetaMessage::SMPTEOffset {
                hour: u8::arbitrary(g),
                minute: u8::arbitrary(g),
                second: u8::arbitrary(g),
                frame: u8::arbitrary(g),
                hundredths_of_a_frame: u8::arbitrary(g),
            };
            // TODO: TimeSignature is a little _too_ arbitrary; e.g. a denominator of 255 would represent a 2^-255th note
            let time_signature = MetaMessage::TimeSignature {
                numerator: u8::arbitrary(g),
                denominator: u8::arbitrary(g),
                cc: u8::arbitrary(g),
                bb: u8::arbitrary(g),
            };
            let key_signature = MetaMessage::KeySignature {
                sharps_or_flats: SharpsOrFlats::arbitrary(g),
                key_type: KeyType::arbitrary(g),
            };
            let sequencer_specific_event =
                MetaMessage::SequencerSpecificEvent(Vec::<u8>::arbitrary(g));
            g.choose(&[
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
            ])
            .expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
            .clone()
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
                MetaMessage::SMPTEOffset {
                    hour,
                    minute,
                    second,
                    frame,
                    hundredths_of_a_frame,
                } => todo!(),
                MetaMessage::TimeSignature {
                    numerator,
                    denominator,
                    cc,
                    bb,
                } => todo!(),
                MetaMessage::KeySignature {
                    sharps_or_flats,
                    key_type,
                } => todo!(),
                MetaMessage::SequencerSpecificEvent(x) => {
                    Box::new(x.shrink().map(MetaMessage::SequencerSpecificEvent))
                }
            }
        }
    }

    // impl Arbitrary for ChannelMessage {
    //     fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    //         let mut channel = u8::arbitrary(g);
    //         while channel >= 16 {
    //             channel = u8::arbitrary(g);
    //         }
    //         ChannelMessage {
    //             message: ChannelMessageWithoutChannel::arbitrary(g),
    //             channel,
    //         }
    //     }
    // }

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

        let mut expected: Vec<u8> = concat_vecs!(
          vec![b'M', b'T', b'r', b'k'],
          u32::try_from(payload_bytes.len()).expect(r#"Payload size must never exceed a u32, per the spec: "Each chunk has ... a 32-bit length" (p. 3)."#).to_be_bytes(),
          payload_bytes
        );
        assert_eq!(Vec::<u8>::from(Chunk::new(events)), expected);
    }

    #[test]
    fn chunk_from_brandenburg_concerto() {
        assert_eq!(
            Chunk::try_from(&crate::test_data::brandenburg::DATA[14..(14 + 35)]),
            Ok(crate::test_data::brandenburg::expected_track())
        );
    }
}
