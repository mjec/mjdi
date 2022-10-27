/// Generates an enum that is backed by a particular representation with an appropriate TryFrom implementation
/// (which can fail with an error of type $error_type_name). The main benefit of using this macro is a guarantee
/// that the TryFrom implementation is exhaustive and matches the Enum.
/// This also generates a quickcheck::Arbitrary implementation for the enum under cfg(test).
macro_rules! backed_enum {
  ($(#[$meta:meta])* $vis:vis enum $enum_name:ident($repr:ty, $error_type_name:ident) {
    $($name:ident $(= $val:expr)?,)+
  }) => {
    $(#[$meta])*
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[repr($repr)]
    $vis enum $enum_name {
      $($name $(= $val)?,)*
    }

    #[derive(Debug, PartialEq, Eq)]
    $vis enum $error_type_name {
      InvalidValue,
    }

    impl std::convert::TryFrom<$repr> for $enum_name {
      type Error = $error_type_name;

      fn try_from(value: $repr) -> Result<Self, Self::Error> {
        match value {
          $(x if x == $enum_name::$name as $repr => Ok($enum_name::$name), )*
          _ => Err(Self::Error::InvalidValue),
        }
      }
    }

    #[cfg(test)]
    impl quickcheck::Arbitrary for $enum_name {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
          g.choose(&[
                $($enum_name::$name,)*
              ])
              .expect("Slice is non-empty, so a non-None value is guaranteed: https://docs.rs/quickcheck/1.0.3/quickcheck/struct.Gen.html#method.choose")
              .clone()
        }
    }
  }
}

/// Create a Vec<u8> by repeatedly extending with arguments.
/// i.e. concat_vecs!(vec![0], vec![1]) == vec![0u8, 1u8]
/// or even better, pre-allocate capacity for the elements:
///   concat_vecs!(2; vec![0], vec![1]) == vec![0u8, 1u8]
macro_rules! concat_vecs {
  ($($vec:expr),+) => {
    {
      let mut result: Vec<u8> = Vec::new();
      $(result.extend($vec);)*
      result
    }
  };

  ($cap:expr; $($vec:expr),+) => {
    {
      let mut result: Vec<u8> = Vec::with_capacity($cap);
      $(result.extend($vec);)*
      result
    }
  };
}
