backed_enum!(pub enum Format(u16, FormatError) {
  SingleMultiChannelTrack = 0,
  OneOrMoreSimultaneousTracks = 1,
  OneOrMoreIndependentTracks = 2,
});
