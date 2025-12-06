use std::{fmt, num::NonZero};

use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};

pub(crate) const DEFAULT_SAMPLING_RATE: u32 = 24000;

#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub struct SamplingRate(NonZero<u32>);

impl SamplingRate {
    pub(super) fn new(n: u32) -> Option<Self> {
        NonZero::new(n)
            .filter(|n| n.get() % DEFAULT_SAMPLING_RATE == 0)
            .map(Self)
    }

    pub(crate) fn get(self) -> u32 {
        self.0.get()
    }
}

impl Default for SamplingRate {
    fn default() -> Self {
        Self::new(DEFAULT_SAMPLING_RATE).unwrap()
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
                SamplingRate::new(n)
                    .ok_or_else(|| de::Error::invalid_value(Unexpected::Unsigned(n.into()), &self))
            }
        }
    }
}
