mod talk;

pub(crate) use self::talk::{
    DecodeInput, DecodeOutput, PredictDurationInput, PredictDurationOutput, PredictIntonationInput,
    PredictIntonationOutput, TalkDomain, TalkOperation,
};

pub(crate) struct InferenceDomainMap<V: InferenceDomainMapValues + ?Sized> {
    pub(crate) talk: V::Talk,
}

pub(crate) trait InferenceDomainMapValues {
    type Talk;
}

impl<T> InferenceDomainMapValues for (T,) {
    type Talk = T;
}

impl<A> InferenceDomainMapValues for [A] {
    type Talk = A;
}

impl<V: InferenceDomainMapValues> InferenceDomainMapValues for Option<V> {
    type Talk = Option<V::Talk>;
}
