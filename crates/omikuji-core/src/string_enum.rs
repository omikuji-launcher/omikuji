macro_rules! string_enum {
    ($name:ident { $($variant:ident => $str:literal),+ $(,)? }) => {
        impl $name {
            pub const VARIANTS: &'static [&'static str] = &[$($str),+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $str),+
                }
            }
        }

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($str => Ok(Self::$variant),)+
                    _ => Err(()),
                }
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                use serde::de::Error;
                let s = String::deserialize(d)?;
                s.parse()
                    .map_err(|()| D::Error::unknown_variant(&s, Self::VARIANTS))
            }
        }
    };
}

pub(crate) use string_enum;
