use std::{fmt, num::NonZero};

use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};

pub(crate) const DEFAULT_SAMPLING_RATE: u32 = 24000;

/// サンプリングレート（Hz）。
///
/// `24000`以外の値は現状推奨されない。
#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub struct SamplingRate(NonZero<u32>);

impl SamplingRate {
    pub fn new(n: NonZero<u32>) -> Option<Self> {
        (n.get() % DEFAULT_SAMPLING_RATE == 0).then_some(Self(n))
    }

    pub fn get(self) -> NonZero<u32> {
        self.0
    }
}

impl Default for SamplingRate {
    fn default() -> Self {
        const _: () = assert!(DEFAULT_SAMPLING_RATE > 0);
        Self(NonZero::new(DEFAULT_SAMPLING_RATE).expect("should have been asserted"))
    }
}

impl<'de> Deserialize<'de> for SamplingRate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return deserializer.deserialize_u32(Visitor);

        struct Visitor;

        impl de::Visitor<'_> for Visitor {
            type Value = SamplingRate;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(fmt, "a non-zero multiple of {DEFAULT_SAMPLING_RATE}")
            }

            fn visit_u32<E>(self, n: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                NonZero::new(n)
                    .and_then(SamplingRate::new)
                    .ok_or_else(|| de::Error::invalid_value(Unexpected::Unsigned(n.into()), &self))
            }
        }
    }
}
