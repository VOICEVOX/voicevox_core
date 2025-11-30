use std::num::NonZero;

pub(crate) const DEFAULT_SAMPLING_RATE: u32 = 24000;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct SamplingRate(NonZero<u32>);

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
