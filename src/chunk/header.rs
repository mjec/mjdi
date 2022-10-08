#[derive(Debug, PartialEq, Eq)]
pub struct Chunk {
    format: Format,
    ntrks: u16,
    division: Division,
}

impl Chunk {
    /// Create a new Header Chunk. (!) Panics if ntrks == 0 (!)
    pub fn new(format: Format, ntrks: u16, division: Division) -> Self {
        assert!(
            ntrks > 0,
            "Cannot create a header for a file with fewer than 1 track"
        );
        Self {
            format,
            ntrks,
            division,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChunkError {
    InvalidSize,
    InvalidType,
    InvalidLength,
    InvalidFormat(FormatError),
    InvalidNumberOfTracks,
    InvalidDivision(DivisionError),
}

impl TryFrom<Vec<u8>> for Chunk {
    type Error = ChunkError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != 14 {
            Err(ChunkError::InvalidSize)
        } else if value[0..4] != [b'M', b'T', b'h', b'd'] {
            Err(ChunkError::InvalidType)
        } else if value[4..8] != [0, 0, 0, 6] {
            Err(ChunkError::InvalidLength)
        } else if u16::from_be_bytes([value[10], value[11]]) == 0 {
            Err(ChunkError::InvalidNumberOfTracks)
        } else {
            Ok(Chunk {
                format: Format::try_from(u16::from_be_bytes([value[8], value[9]]))?,
                ntrks: u16::from_be_bytes([value[10], value[11]]),
                division: Division::try_from(u16::from_be_bytes([value[12], value[13]]))?,
            })
        }
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 14 {
            Err(ChunkError::InvalidSize)
        } else if value[0..4] != [b'M', b'T', b'h', b'd'] {
            eprintln!("{:?}", value);
            Err(ChunkError::InvalidType)
        } else if value[4..8] != [0, 0, 0, 6] {
            Err(ChunkError::InvalidLength)
        } else if u16::from_be_bytes([value[10], value[11]]) == 0 {
            Err(ChunkError::InvalidNumberOfTracks)
        } else {
            Ok(Chunk {
                format: Format::try_from(u16::from_be_bytes([value[8], value[9]]))?,
                ntrks: u16::from_be_bytes([value[10], value[11]]),
                division: Division::try_from(u16::from_be_bytes([value[12], value[13]]))?,
            })
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Format {
    SingleMultiChannelTrack = 0,
    OneOrMoreSimultaneousTracks = 1,
    OneOrMoreIndependentTracks = 2,
}

#[derive(PartialEq, Eq, Debug)]
pub enum FormatError {
    InvalidValue,
}

impl From<FormatError> for ChunkError {
    fn from(e: FormatError) -> Self {
        Self::InvalidFormat(e)
    }
}

impl TryFrom<u16> for Format {
    type Error = FormatError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == (Format::SingleMultiChannelTrack as u16) => {
                Ok(Format::SingleMultiChannelTrack)
            }
            x if x == (Format::OneOrMoreSimultaneousTracks as u16) => {
                Ok(Format::OneOrMoreSimultaneousTracks)
            }
            x if x == (Format::OneOrMoreIndependentTracks as u16) => {
                Ok(Format::OneOrMoreIndependentTracks)
            }
            _ => Err(FormatError::InvalidValue),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Division {
    TicksPerQuarterNote(u16),
    SubdivisionsOfASecond {
        timecode_format: SMPTETimecodeFormat,
        ticks_per_frame: u8,
    },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SMPTETimecodeFormat {
    NegTwentyFour = -24,
    NegTwentyFive = -25,
    NegTwentyNine = -29,
    NegThirty = -30,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SMPTETimecodeFormatError {
    InvalidValue,
}

impl TryFrom<i8> for SMPTETimecodeFormat {
    type Error = SMPTETimecodeFormatError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            x if x == (SMPTETimecodeFormat::NegTwentyFour as i8) => {
                Ok(SMPTETimecodeFormat::NegTwentyFour)
            }
            x if x == (SMPTETimecodeFormat::NegTwentyFive as i8) => {
                Ok(SMPTETimecodeFormat::NegTwentyFive)
            }
            x if x == (SMPTETimecodeFormat::NegTwentyNine as i8) => {
                Ok(SMPTETimecodeFormat::NegTwentyNine)
            }
            x if x == (SMPTETimecodeFormat::NegThirty as i8) => Ok(SMPTETimecodeFormat::NegThirty),
            _ => Err(SMPTETimecodeFormatError::InvalidValue),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DivisionError {
    TicksPerQuarterNoteMustBeGreaterThanZero,
    TicksPerFrameMustBeGreaterThanZero,
    SMPTETimecodeFormatError(SMPTETimecodeFormatError),
}

impl From<DivisionError> for ChunkError {
    fn from(e: DivisionError) -> Self {
        Self::InvalidDivision(e)
    }
}

impl From<SMPTETimecodeFormatError> for DivisionError {
    fn from(e: SMPTETimecodeFormatError) -> Self {
        DivisionError::SMPTETimecodeFormatError(e)
    }
}

impl TryFrom<u16> for Division {
    type Error = DivisionError;

    fn try_from(bytes: u16) -> Result<Self, Self::Error> {
        if bytes == 0 {
            return Err(DivisionError::TicksPerQuarterNoteMustBeGreaterThanZero);
        }
        let inverse_mask = !Self::TICKS_PER_QUARTER_NOTE_MASK;

        if (bytes & inverse_mask) == 0 {
            return Ok(Self::TicksPerQuarterNote(
                bytes & Self::TICKS_PER_QUARTER_NOTE_MASK,
            ));
        }

        assert!(
          bytes & inverse_mask == inverse_mask,
          "Bitwise operations should make this invariant; either the top bit is set (i.e. bytes & inverse_mask == inverse_mask) or it's not (bytes & inverse_mask == 0). But if TICKS_PER_QUARTER_NOTE_MASK isn't just looking at the top bit, this might be wrong!"
        );

        let timecode_format = SMPTETimecodeFormat::try_from(bytes.to_be_bytes()[0] as i8)?;

        if bytes.to_be_bytes()[1] == 0 {
            return Err(DivisionError::TicksPerFrameMustBeGreaterThanZero);
        }

        Ok(Self::SubdivisionsOfASecond {
            timecode_format,
            ticks_per_frame: bytes.to_be_bytes()[1],
        })
    }
}

impl Division {
    const TICKS_PER_QUARTER_NOTE_MASK: u16 = 0b0111_1111_1111_1111u16;
    fn high_byte(&self) -> u8 {
        match self {
            Division::TicksPerQuarterNote(n) => {
                (n & Self::TICKS_PER_QUARTER_NOTE_MASK).to_be_bytes()[0]
            }
            Division::SubdivisionsOfASecond {
                timecode_format: negative,
                ..
            } => *negative as u8,
        }
    }

    fn low_byte(&self) -> u8 {
        match self {
            Division::TicksPerQuarterNote(n) => {
                (n & Self::TICKS_PER_QUARTER_NOTE_MASK).to_be_bytes()[1]
            }
            Division::SubdivisionsOfASecond {
                ticks_per_frame, ..
            } => *ticks_per_frame,
        }
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
            self.ntrks.to_be_bytes()[0],
            self.ntrks.to_be_bytes()[1],
            self.division.high_byte(),
            self.division.low_byte(),
        ];
        eprintln!("{:?}", output);
        output.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{Arbitrary, TestResult};
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn division_values_from_spec() {
        assert_eq!(
            Division::try_from(1u16).expect("It's in the spec!"),
            Division::TicksPerQuarterNote(1)
        );
        assert_eq!(
            Division::try_from(0xE250u16).expect("It's in the spec!"),
            Division::SubdivisionsOfASecond {
                timecode_format: SMPTETimecodeFormat::NegThirty,
                ticks_per_frame: 80,
            }
        );
    }

    #[quickcheck]
    fn division_ticks_per_quarter_note(value: u16) -> TestResult {
        if value == 0 {
            TestResult::from_bool(
                Division::try_from(value)
                    == Err(DivisionError::TicksPerQuarterNoteMustBeGreaterThanZero),
            )
        } else if value < !Division::TICKS_PER_QUARTER_NOTE_MASK && value > 0 {
            TestResult::from_bool(
                Division::try_from(value) == Ok(Division::TicksPerQuarterNote(value)),
            )
        } else if value < !Division::TICKS_PER_QUARTER_NOTE_MASK {
            TestResult::from_bool(
                Division::try_from(value) == Err(DivisionError::TicksPerFrameMustBeGreaterThanZero),
            )
        } else {
            TestResult::discard()
        }
    }

    impl Arbitrary for SMPTETimecodeFormat {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            g.choose(&[
              Self::NegTwentyFour,
              Self::NegTwentyFive,
              Self::NegTwentyNine,
              Self::NegThirty,
            ])
            .expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
            .clone()
        }
    }

    #[quickcheck]
    fn smpte_timecode_format_round_trips(timecode_format: SMPTETimecodeFormat) {
        assert_eq!(
            SMPTETimecodeFormat::try_from(timecode_format.clone() as i8),
            Ok(timecode_format.clone())
        );
        assert_eq!(
            SMPTETimecodeFormat::try_from(((timecode_format.clone() as i8) as u8) as i8),
            Ok(timecode_format.clone())
        );
    }

    #[quickcheck]
    fn smpte_timecode_format_likely_invalid(timecode_format: i8) -> TestResult {
        if timecode_format >= 0 {
            TestResult::discard()
        } else {
            match timecode_format {
                -24 => TestResult::from_bool(
                    SMPTETimecodeFormat::try_from(timecode_format)
                        == Ok(SMPTETimecodeFormat::NegTwentyFour),
                ),
                -25 => TestResult::from_bool(
                    SMPTETimecodeFormat::try_from(timecode_format)
                        == Ok(SMPTETimecodeFormat::NegTwentyFive),
                ),
                -29 => TestResult::from_bool(
                    SMPTETimecodeFormat::try_from(timecode_format)
                        == Ok(SMPTETimecodeFormat::NegTwentyNine),
                ),
                -30 => TestResult::from_bool(
                    SMPTETimecodeFormat::try_from(timecode_format)
                        == Ok(SMPTETimecodeFormat::NegThirty),
                ),
                x => TestResult::from_bool(
                    SMPTETimecodeFormat::try_from(x) == Err(SMPTETimecodeFormatError::InvalidValue),
                ),
            }
        }
    }

    #[quickcheck]
    /// Called narrow because it checks a narrower range of invalid values than the broad version
    fn division_ticks_per_frame_narrow(
        timecode_format: SMPTETimecodeFormat,
        ticks_per_frame: u8,
    ) -> TestResult {
        let value: u16 = u16::from_be_bytes([timecode_format as u8, ticks_per_frame]);
        if ticks_per_frame == 0 {
            TestResult::from_bool(
                Division::try_from(value) == Err(DivisionError::TicksPerFrameMustBeGreaterThanZero),
            )
        } else {
            TestResult::from_bool(
                Division::try_from(value)
                    == Ok(Division::SubdivisionsOfASecond {
                        timecode_format,
                        ticks_per_frame,
                    }),
            )
        }
    }

    #[quickcheck]
    // Called broad because it checks a wider range of invalid values than the narrow version
    fn division_ticks_per_frame_broad(value: u16) -> TestResult {
        if value <= Division::TICKS_PER_QUARTER_NOTE_MASK {
            TestResult::discard()
        } else if SMPTETimecodeFormat::try_from(value.to_be_bytes()[0] as i8).is_err() {
            TestResult::from_bool(
                Division::try_from(value)
                    == Err(DivisionError::SMPTETimecodeFormatError(
                        SMPTETimecodeFormatError::InvalidValue,
                    )),
            )
        } else if value.to_be_bytes()[1] == 0 {
            TestResult::from_bool(
                Division::try_from(value) == Err(DivisionError::TicksPerFrameMustBeGreaterThanZero),
            )
        } else {
            TestResult::from_bool(
                Division::try_from(value)
                    == Ok(Division::SubdivisionsOfASecond {
                        timecode_format: SMPTETimecodeFormat::try_from(
                            value.to_be_bytes()[0] as i8,
                        )
                        .expect("Guaranteed to be Ok() because we checked if it is_err() above"),
                        ticks_per_frame: value.to_be_bytes()[1],
                    }),
            )
        }
    }

    #[quickcheck]
    /// This should produce a successful test result for ANY u16, covering the entire behavior of
    /// Division::try_from(value: u16). That's cool and all, but too much of its domain is invalid
    /// for it to be a super useful test in the `cargo t` case (where tests should be fast). It
    /// still runs there (and has caught bugs for me there), but it's really most useful when run
    /// for an extended period as a fuzzer.
    fn division_fuzz(value: u16) -> TestResult {
        if value == 0 {
            TestResult::from_bool(
                Division::try_from(value)
                    == Err(DivisionError::TicksPerQuarterNoteMustBeGreaterThanZero),
            )
        } else if value < !Division::TICKS_PER_QUARTER_NOTE_MASK {
            TestResult::from_bool(
                Division::try_from(value) == Ok(Division::TicksPerQuarterNote(value)),
            )
        } else if SMPTETimecodeFormat::try_from(value.to_be_bytes()[0] as i8).is_err() {
            TestResult::from_bool(
                Division::try_from(value)
                    == Err(DivisionError::SMPTETimecodeFormatError(
                        SMPTETimecodeFormatError::InvalidValue,
                    )),
            )
        } else if value.to_be_bytes()[1] == 0 {
            TestResult::from_bool(
                Division::try_from(value) == Err(DivisionError::TicksPerFrameMustBeGreaterThanZero),
            )
        } else {
            TestResult::from_bool(
                Division::try_from(value)
                    == Ok(Division::SubdivisionsOfASecond {
                        timecode_format: SMPTETimecodeFormat::try_from(
                            value.to_be_bytes()[0] as i8,
                        )
                        .expect("Guaranteed to be Ok() because we checked if it is_err() above"),
                        ticks_per_frame: value.to_be_bytes()[1],
                    }),
            )
        }
    }

    impl Arbitrary for Division {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            if *g.choose(&[true, false]).expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
            {
              let mut ticks = 0;
              while ticks == 0 {
                ticks = u16::arbitrary(g) & Division::TICKS_PER_QUARTER_NOTE_MASK;
              }
                Division::TicksPerQuarterNote(ticks)
            } else {
                let mut ticks_per_frame = 0u8;
                while ticks_per_frame == 0 {
                    ticks_per_frame = u8::arbitrary(g);
                }
                Division::SubdivisionsOfASecond {
                    timecode_format: SMPTETimecodeFormat::arbitrary(g),
                    ticks_per_frame,
                }
            }
        }
    }

    #[derive(Clone, Debug)]
    struct FourteenBytes {
        data: [u8; 14],
    }

    impl Arbitrary for FourteenBytes {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut result = FourteenBytes { data: [0u8; 14] };
            let alphabet = vec![
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
                0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
                0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45,
                0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53,
                0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x61,
                0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
                0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D,
                0x7E, 0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B,
                0x8C, 0x8D, 0x8E, 0x8F, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99,
                0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
                0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF, 0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5,
                0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC0, 0xC1, 0xC2, 0xC3,
                0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF, 0xD0, 0xD1,
                0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
                0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED,
                0xEE, 0xEF, 0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB,
                0xFC, 0xFD, 0xFE, 0xFF,
            ];
            for i in 0..14 {
                result.data[i] = *g.choose(&alphabet).expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
            }
            result
        }
    }

    impl Arbitrary for Format {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            g
            .choose(&[Format::SingleMultiChannelTrack, Format::OneOrMoreSimultaneousTracks,Format::OneOrMoreIndependentTracks])
            .expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
            .clone()
        }
    }

    #[quickcheck]
    fn chunk_roundtrips(format: Format, ntrks: u16, division: Division) -> TestResult {
        if ntrks == 0 {
            TestResult::discard()
        } else {
            TestResult::from_bool(
                Chunk::try_from(
                    Chunk::new(format.clone(), ntrks, division.clone())
                        .into_iter()
                        .collect::<Vec<u8>>(),
                ) == Ok(Chunk::new(format.clone(), ntrks, division.clone())),
            )
        }
    }

    #[quickcheck]
    /// This test always passes, as long as it doesn't panic.
    fn chunk_fuzz(value: FourteenBytes) -> TestResult {
        match Chunk::try_from(&value.data[0..14]) {
            _ => TestResult::passed(),
        }
    }
}
