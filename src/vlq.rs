use std::fmt::{Debug, Display};

pub const MAX_REPRESENTABLE: u32 = 0x0FFFFFFF;

/// Variable length quantity encoding of integers.
/// Integers must be less than or equal to MAX_REPRESENTABLE.
#[derive(Debug, PartialEq, Eq, Ord, Clone, Copy)]
pub struct VLQ {
    bytes: [u8; 4],
    size: usize,
}

impl PartialOrd for VLQ {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        u32::from(self).partial_cmp(&u32::from(other))
    }
}

impl Display for VLQ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#010X}", u32::from(self))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VLQError {
    OverMaxSize,
}

impl TryFrom<u32> for VLQ {
    type Error = VLQError;

    fn try_from(mut n: u32) -> Result<Self, Self::Error> {
        if n > MAX_REPRESENTABLE {
            Err(Self::Error::OverMaxSize)
        } else {
            let mut bytes = [0u8; 4];
            let mut size: usize = 1;

            while n > 0x7F {
                bytes[size - 1] = (n & 0x7F) as u8 | 0x80;
                n = n >> 7;
                size = size + 1;
            }
            bytes[size - 1] = 0x80 | (n & 0x7F) as u8;
            bytes[0] = bytes[0] & 0x7F;

            Ok(Self { bytes, size })
        }
    }
}

impl From<VLQ> for u32 {
    fn from(n: VLQ) -> u32 {
        let mut result: u32 = 0;
        for byte in n {
            result = result << 7;
            result = result | (byte & 0x7F) as u32;
        }

        result
    }
}

impl From<&VLQ> for u32 {
    fn from(n: &VLQ) -> u32 {
        u32::from(*n)
    }
}

impl IntoIterator for VLQ {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut result = Vec::<u8>::with_capacity(4);
        for i in 0..self.size {
            result.push(self.bytes[i]);
        }
        result.reverse();
        result.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use quickcheck_macros::quickcheck;

    use super::*;

    /// These parameters are copied from the MIDI specification.
    #[test]
    fn byte_representations_are_correct() {
        assert_eq!(
            VLQ::try_from(0x00000000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x00]
        );
        assert_eq!(
            VLQ::try_from(0x00000040)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x40]
        );
        assert_eq!(
            VLQ::try_from(0x0000007F)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x7F]
        );
        assert_eq!(
            VLQ::try_from(0x00000080)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x81, 0x00]
        );
        assert_eq!(
            VLQ::try_from(0x00002000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xC0, 0x00]
        );
        assert_eq!(
            VLQ::try_from(0x00003FFF)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xFF, 0x7F]
        );
        assert_eq!(
            VLQ::try_from(0x00004000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x81, 0x80, 0x00]
        );
        assert_eq!(
            VLQ::try_from(0x00100000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xC0, 0x80, 0x00]
        );
        assert_eq!(
            VLQ::try_from(0x001FFFFF)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xFF, 0xFF, 0x7F]
        );
        assert_eq!(
            VLQ::try_from(0x00200000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x81, 0x80, 0x80, 0x00]
        );
        assert_eq!(
            VLQ::try_from(0x08000000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xC0, 0x80, 0x80, 0x00]
        );
        assert_eq!(
            VLQ::try_from(0x0FFFFFFF)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xFF, 0xFF, 0xFF, 0x7F]
        );
    }

    #[quickcheck]
    fn round_trip_from_and_to_u32_works(n: u32) {
        if n > MAX_REPRESENTABLE {
            assert_eq!(VLQ::try_from(n), Err(VLQError::OverMaxSize));
        } else {
            assert_eq!(VLQ::try_from(n).map(|x| u32::from(x)), Ok(n));
        }
    }
}
