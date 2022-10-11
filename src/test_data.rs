#![cfg(test)]

pub(crate) mod brandenburg {
    use std::num::NonZeroU16;

    pub(crate) const DATA: &[u8; 81420] = include_bytes!("../brandenburg.mid");

    pub(crate) fn expected_header() -> crate::chunk::header::Chunk {
        crate::chunk::header::Chunk::new(
            crate::chunk::header::Format::OneOrMoreSimultaneousTracks,
            NonZeroU16::new(11).unwrap(),
            crate::chunk::header::Division::TicksPerQuarterNote(NonZeroU16::new(1024).unwrap()),
        )
    }
}
