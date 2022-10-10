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

backed_enum!(
  pub enum SMPTETimecodeFormat(i8, SMPTETimecodeFormatError) {
    NegTwentyFour = -24,
    NegTwentyFive = -25,
    NegTwentyNine = -29,
    NegThirty = -30,
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
        let inverse_mask = !Self::TICKS_PER_QUARTER_NOTE_MASK;

        if (bytes & inverse_mask) == 0 {
            return Ok(Self::TicksPerQuarterNote(
                NonZeroU16::new(bytes & Self::TICKS_PER_QUARTER_NOTE_MASK).ok_or::<DivisionError>(
                    DivisionError::TicksPerQuarterNoteMustBeGreaterThanZero,
                )?,
            ));
        }

        assert!(
          bytes & inverse_mask == inverse_mask,
          "Bitwise operations should make this invariant; either the top bit is set (i.e. bytes & inverse_mask == inverse_mask) or it's not (bytes & inverse_mask == 0). But if TICKS_PER_QUARTER_NOTE_MASK isn't just looking at the top bit, this might be wrong!"
        );

        let timecode_format = SMPTETimecodeFormat::try_from(bytes.to_be_bytes()[0] as i8)?;

        Ok(Self::SubdivisionsOfASecond {
            timecode_format,
            ticks_per_frame: NonZeroU8::new(bytes.to_be_bytes()[1])
                .ok_or(DivisionError::TicksPerFrameMustBeGreaterThanZero)?,
        })
    }
}

impl Division {
    pub(crate) const TICKS_PER_QUARTER_NOTE_MASK: u16 = 0b0111_1111_1111_1111u16;
    pub(crate) fn high_byte(&self) -> u8 {
        match self {
            Division::TicksPerQuarterNote(n) => {
                (n.get() & Self::TICKS_PER_QUARTER_NOTE_MASK).to_be_bytes()[0]
            }
            Division::SubdivisionsOfASecond {
                timecode_format: negative,
                ..
            } => *negative as u8,
        }
    }

    pub(crate) fn low_byte(&self) -> u8 {
        match self {
            Division::TicksPerQuarterNote(n) => {
                (n.get() & Self::TICKS_PER_QUARTER_NOTE_MASK).to_be_bytes()[1]
            }
            Division::SubdivisionsOfASecond {
                ticks_per_frame, ..
            } => ticks_per_frame.get(),
        }
    }
}
