backed_enum!(pub enum Format(u16, FormatError) {
  SingleMultiChannelTrack = 0,
  OneOrMoreSimultaneousTracks = 1,
  OneOrMoreIndependentTracks = 2,
});

impl From<Format> for Vec<u8> {
    fn from(format: Format) -> Self {
        Vec::from((format as u16).to_be_bytes())
    }
}
