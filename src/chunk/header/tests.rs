#![cfg(test)]

use std::num::{NonZeroU16, NonZeroU8};

use quickcheck::{Arbitrary, TestResult};
use quickcheck_macros::quickcheck;

use super::*;

#[test]
fn division_values_from_spec() {
    assert_eq!(
        Division::try_from(1u16).expect("It's in the spec!"),
        Division::TicksPerQuarterNote(NonZeroU16::new(1).expect("Value is non-zero"))
    );
    assert_eq!(
        Division::try_from(0xE250u16).expect("It's in the spec!"),
        Division::SubdivisionsOfASecond {
            timecode_format: SMPTETimecodeFormat::NegThirty,
            ticks_per_frame: NonZeroU8::new(80).expect("Value is non-zero"),
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
    } else if value < Division::MARKER_BIT_MASK && value > 0 {
        TestResult::from_bool(
            Division::try_from(value)
                == Ok(Division::TicksPerQuarterNote(
                    NonZeroU16::new(value)
                        .expect("We already handle the value == 0 case in an earlier branch"),
                )),
        )
    } else if value < Division::MARKER_BIT_MASK {
        TestResult::from_bool(
            Division::try_from(value) == Err(DivisionError::TicksPerFrameMustBeGreaterThanZero),
        )
    } else {
        TestResult::discard()
    }
}

#[quickcheck]
/// Called narrow because it checks a narrower range of invalid values than the broad version
fn division_ticks_per_frame_narrow(timecode_format: SMPTETimecodeFormat, ticks_per_frame: u8) {
    let value: u16 = u16::from_be_bytes([timecode_format as u8, ticks_per_frame]);
    if ticks_per_frame == 0 {
        assert_eq!(
            Division::try_from(value),
            Err(DivisionError::TicksPerFrameMustBeGreaterThanZero)
        );
    } else {
        assert_eq!(
            Division::try_from(value),
            Ok(Division::SubdivisionsOfASecond {
                timecode_format,
                ticks_per_frame:
                    NonZeroU8::new(ticks_per_frame).expect(
                        "We already handle the ticks_per_frame == 0 case in an earlier branch",
                    ),
            })
        );
    }
}

#[quickcheck]
// Called broad because it checks a wider range of invalid values than the narrow version
fn division_ticks_per_frame_broad(value: u16) -> TestResult {
    if value & Division::MARKER_BIT_MASK == 0 {
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
            Division::try_from(value) == Ok(Division::SubdivisionsOfASecond {
                timecode_format: SMPTETimecodeFormat::try_from(value.to_be_bytes()[0] as i8)
                    .expect("Guaranteed to be Ok() because we checked if it is_err() above"),
                ticks_per_frame: NonZeroU8::new(value.to_be_bytes()[1]).expect(
                    "We already handle the value.to_be_bytes()[1] == 0 case in an earlier branch",
                ),
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
fn division_fuzz(value: u16) {
    if value == 0 {
        assert_eq!(
            Division::try_from(value),
            Err(DivisionError::TicksPerQuarterNoteMustBeGreaterThanZero)
        );
    } else if value < Division::MARKER_BIT_MASK {
        assert_eq!(
            Division::try_from(value),
            Ok(Division::TicksPerQuarterNote(
                NonZeroU16::new(value)
                    .expect("We already handle the value == 0 case in an earlier branch"),
            ))
        );
    } else if SMPTETimecodeFormat::try_from(value.to_be_bytes()[0] as i8).is_err() {
        assert_eq!(
            Division::try_from(value),
            Err(DivisionError::SMPTETimecodeFormatError(
                SMPTETimecodeFormatError::InvalidValue,
            ))
        );
    } else if value.to_be_bytes()[1] == 0 {
        assert_eq!(
            Division::try_from(value),
            Err(DivisionError::TicksPerFrameMustBeGreaterThanZero)
        );
    } else {
        assert_eq!(
            Division::try_from(value),
            Ok(Division::SubdivisionsOfASecond {
                timecode_format: SMPTETimecodeFormat::try_from(value.to_be_bytes()[0] as i8)
                    .expect("Guaranteed to be Ok() because we checked if it is_err() above"),
                ticks_per_frame: NonZeroU8::new(value.to_be_bytes()[1]).expect(
                    "We already handle the value.to_be_bytes()[1] == 0 case in an earlier branch",
                ),
            })
        );
    }
}

impl Arbitrary for Division {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if *g.choose(&[true, false]).expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
            {
              let mut ticks: Option<NonZeroU16> = None;
              while ticks.is_none() {
                ticks = NonZeroU16::new(NonZeroU16::arbitrary(g).get() & !Division::MARKER_BIT_MASK);
              }
                Division::TicksPerQuarterNote(ticks.expect("We are guaranteed this is_some() by the preceding while loop condition"))
            } else {
                Division::SubdivisionsOfASecond {
                    timecode_format: SMPTETimecodeFormat::arbitrary(g),
                    ticks_per_frame: NonZeroU8::arbitrary(g),
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
        for i in 0..14 {
            result.data[i] = *g.choose(Vec::from_iter(0u8..=255u8).as_slice()).expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
        }
        result
    }
}

#[quickcheck]
fn chunk_roundtrips(format: Format, ntrks: NonZeroU16, division: Division) {
    let chunk = Chunk::new(format, ntrks, division);
    assert_eq!(
        Chunk::try_from(chunk.clone().into_iter().collect::<Vec<u8>>().as_slice()),
        Ok(chunk),
    );
}

#[quickcheck]
/// This test always passes, as long as it doesn't panic.
fn chunk_fuzz(value: FourteenBytes) -> TestResult {
    match Chunk::try_from(&value.data[0..14]) {
        _ => TestResult::passed(),
    }
}
