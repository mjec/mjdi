use std::num::{NonZeroU16, NonZeroU8};

use super::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Division {
    TicksPerQuarterNote(NonZeroU16),
    SubdivisionsOfASecond {
        timecode_format: SMPTETimecodeFormat,
        ticks_per_frame: NonZeroU8,
    },
}

impl From<Division> for Vec<u8> {
    fn from(div: Division) -> Self {
        match div {
            Division::TicksPerQuarterNote(n) => {
                debug_assert!(n.get() & Division::MARKER_BIT_MASK == 0);
                Vec::from(n.get().to_be_bytes())
            }
            Division::SubdivisionsOfASecond {
                timecode_format,
                ticks_per_frame,
            } => {
                debug_assert!(
                    (timecode_format as u16) << 8 & Division::MARKER_BIT_MASK
                        == Division::MARKER_BIT_MASK
                );
                vec![timecode_format as u8, ticks_per_frame.get()]
            }
        }
    }
}

backed_enum!(
  pub enum SMPTETimecodeFormat(i8, SMPTETimecodeFormatError) {
    TwentyFour = -24,
    TwentyFive = -25,
    ThirtyDropFrame = -29,
    Thirty = -30,
  }
);

#[derive(Debug, PartialEq, Eq)]
pub enum DivisionError {
    TicksPerQuarterNoteMustBeGreaterThanZero,
    TicksPerFrameMustBeGreaterThanZero,
    SMPTETimecodeFormatError(SMPTETimecodeFormatError),
}

impl From<DivisionError> for ChunkError {
    fn from(e: DivisionError) -> Self {
        Self::Division(e)
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
        if (bytes & Self::MARKER_BIT_MASK) == 0 {
            Ok(Self::TicksPerQuarterNote(
                NonZeroU16::new(bytes & !Self::MARKER_BIT_MASK).ok_or::<DivisionError>(
                    DivisionError::TicksPerQuarterNoteMustBeGreaterThanZero,
                )?,
            ))
        } else {
            Ok(Self::SubdivisionsOfASecond {
                timecode_format: SMPTETimecodeFormat::try_from(bytes.to_be_bytes()[0] as i8)?,
                ticks_per_frame: NonZeroU8::new(bytes.to_be_bytes()[1])
                    .ok_or(DivisionError::TicksPerFrameMustBeGreaterThanZero)?,
            })
        }
    }
}

impl Division {
    /// The on bit marks the format of this division:
    ///  0 => ticks per quarter note
    ///  1 => subdivisions of a second
    pub(super) const MARKER_BIT_MASK: u16 = 0b1000_0000_0000_0000;

    pub(crate) fn high_byte(&self) -> u8 {
        match self {
            Division::TicksPerQuarterNote(n) => (!Self::MARKER_BIT_MASK & n.get()).to_be_bytes()[0],
            Division::SubdivisionsOfASecond {
                timecode_format, ..
            } => *timecode_format as u8, // This always has Self::MARKER_BIT_MASK set because timecode_format is a negative i8.
                                         // Still, this is a slight code smell; here we rely on Self::MARKER_BIT_MASK being a u16
                                         // with only its top bit 1, AND timecode_format being a negative i8. Any change to the
                                         // representation or range of SMPTETimecodeFormat, or any change to Self::MARKER_BIT_MASK,
                                         // will invalidate this code. I would prefer some explicit signal of that, so changes to
                                         // those would require changes here.
                                         // Longer term, I think these methods will go away, to be replaced by `impl From<T> for &[u8]`
                                         // for all these types, including T = Division.
        }
    }

    pub(crate) fn low_byte(&self) -> u8 {
        match self {
            Division::TicksPerQuarterNote(n) => (!Self::MARKER_BIT_MASK & n.get()).to_be_bytes()[1],
            Division::SubdivisionsOfASecond {
                ticks_per_frame, ..
            } => ticks_per_frame.get(),
        }
    }
}
