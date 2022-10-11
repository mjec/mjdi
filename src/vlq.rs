use std::fmt::{Debug, Display};

pub const MAX_REPRESENTABLE: u32 = 0x0FFFFFFF;

/// Variable length quantity encoding of integers.
/// Integers must be less than or equal to MAX_REPRESENTABLE.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Vlq {
    bytes: [u8; 4],
    size: usize,
}

impl Vlq {
    pub fn get(&self) -> u32 {
        u32::from(self)
    }
}

impl PartialOrd for Vlq {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        u32::from(self).partial_cmp(&u32::from(other))
    }
}

impl Ord for Vlq {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        u32::from(self).cmp(&u32::from(other))
    }
}

impl Display for Vlq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#010X}", u32::from(self))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VLQError {
    OverMaxSize,
}

impl TryFrom<u32> for Vlq {
    type Error = VLQError;

    fn try_from(mut n: u32) -> Result<Self, Self::Error> {
        if n > MAX_REPRESENTABLE {
            Err(Self::Error::OverMaxSize)
        } else {
            let mut bytes = [0u8; 4];
            let mut size: usize = 1;

            while n > 0x7F {
                bytes[size - 1] = (n & 0x7F) as u8 | 0x80;
                n >>= 7;
                size += 1;
            }
            bytes[size - 1] = 0x80 | (n & 0x7F) as u8;
            bytes[0] &= 0x7F;

            Ok(Self { bytes, size })
        }
    }
}

impl From<Vlq> for u32 {
    fn from(n: Vlq) -> u32 {
        let mut result: u32 = 0;
        for byte in n {
            result <<= 7;
            result |= (byte & 0x7F) as u32;
        }

        result
    }
}

impl From<&Vlq> for u32 {
    fn from(n: &Vlq) -> u32 {
        u32::from(*n)
    }
}

impl IntoIterator for Vlq {
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

    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;

    use super::*;

    impl Arbitrary for Vlq {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut u32_value: u32 = u32::arbitrary(g);
            while u32_value > crate::vlq::MAX_REPRESENTABLE {
                u32_value = u32::arbitrary(g);
            }

            Vlq::try_from(u32_value).expect("We guarantee that u32_value <= MAX_REPRESENTABLE")
        }
    }

    /// These parameters are copied from the MIDI specification.
    #[test]
    fn byte_representations_are_correct() {
        assert_eq!(
            Vlq::try_from(0x00000000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x00]
        );
        assert_eq!(
            Vlq::try_from(0x00000040)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x40]
        );
        assert_eq!(
            Vlq::try_from(0x0000007F)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x7F]
        );
        assert_eq!(
            Vlq::try_from(0x00000080)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x81, 0x00]
        );
        assert_eq!(
            Vlq::try_from(0x00002000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xC0, 0x00]
        );
        assert_eq!(
            Vlq::try_from(0x00003FFF)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xFF, 0x7F]
        );
        assert_eq!(
            Vlq::try_from(0x00004000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x81, 0x80, 0x00]
        );
        assert_eq!(
            Vlq::try_from(0x00100000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xC0, 0x80, 0x00]
        );
        assert_eq!(
            Vlq::try_from(0x001FFFFF)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xFF, 0xFF, 0x7F]
        );
        assert_eq!(
            Vlq::try_from(0x00200000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0x81, 0x80, 0x80, 0x00]
        );
        assert_eq!(
            Vlq::try_from(0x08000000)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xC0, 0x80, 0x80, 0x00]
        );
        assert_eq!(
            Vlq::try_from(0x0FFFFFFF)
                .expect("Value is in spec!")
                .into_iter()
                .collect::<Vec<u8>>(),
            vec![0xFF, 0xFF, 0xFF, 0x7F]
        );
    }

    #[quickcheck]
    fn round_trip_from_and_to_u32_works(n: u32) {
        if n > MAX_REPRESENTABLE {
            assert_eq!(Vlq::try_from(n), Err(VLQError::OverMaxSize));
        } else {
            assert_eq!(Vlq::try_from(n).map(u32::from), Ok(n));
        }
    }
}
