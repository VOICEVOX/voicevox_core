use std::{fmt, num::NonZero};

use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};

use crate::error::{InvalidQueryError, InvalidQueryErrorSource};

pub(crate) const DEFAULT_SAMPLING_RATE: u32 = DEFAULT_SAMPLING_RATE_.get();

const DEFAULT_SAMPLING_RATE_: NonZero<u32> = NonZero::new(24000).unwrap();

/// サンプリングレート（Hz）。
///
/// `24000`以外の値は現状推奨されない。
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, derive_more::Display, Serialize,
)]
pub struct SamplingRate(NonZero<u32>);

impl SamplingRate {
    pub fn new(n: u32) -> crate::Result<Self> {
        Self::new_(n).map_err(Into::into)
    }

    pub(super) fn new_(n: u32) -> Result<Self, InvalidQueryError> {
        let error = |source| InvalidQueryError {
            what: "サンプリングレート",
            value: Some(Box::new(n) as _),
            source: Some(source),
        };

        let n = NonZero::new(n)
            .ok_or_else(|| error(InvalidQueryErrorSource::IsNotMultipleOfBaseSamplingRate))?; // TODO: `IsZero`にする
        (n.get() % DEFAULT_SAMPLING_RATE == 0)
            .then_some(Self(n))
            .ok_or_else(|| error(InvalidQueryErrorSource::IsNotMultipleOfBaseSamplingRate))
    }

    pub fn get(self) -> NonZero<u32> {
        self.0
    }
}

impl Default for SamplingRate {
    fn default() -> Self {
        Self(DEFAULT_SAMPLING_RATE_)
    }
}

impl From<SamplingRate> for NonZero<u32> {
    fn from(sampling_rate: SamplingRate) -> Self {
        sampling_rate.0
    }
}

impl TryFrom<u32> for SamplingRate {
    type Error = crate::Error;

    fn try_from(n: u32) -> Result<Self, Self::Error> {
        Self::new(n)
    }
}

impl TryFrom<NonZero<u32>> for SamplingRate {
    type Error = crate::Error;

    fn try_from(n: NonZero<u32>) -> Result<Self, Self::Error> {
        Self::new(n.get())
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
                SamplingRate::new_(n)
                    .map_err(|_| de::Error::invalid_value(Unexpected::Unsigned(n.into()), &self))
            }
        }
    }
}
