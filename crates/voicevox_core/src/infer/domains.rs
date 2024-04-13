mod talk;

pub(crate) use self::talk::{
    DecodeInput, DecodeOutput, PredictDurationInput, PredictDurationOutput, PredictIntonationInput,
    PredictIntonationOutput, TalkDomain, TalkOperation,
};

use super::{InferenceDomainGroup, InferenceDomainMap, InferenceDomainMapValueProjection};

pub(crate) enum InferenceDomainGroupImpl {}

impl InferenceDomainGroup for InferenceDomainGroupImpl {
    type Map<V: InferenceDomainMapValueProjection> = InferenceDomainMapImpl<V>;
}

pub(crate) struct InferenceDomainMapImpl<V: InferenceDomainMapValueProjection> {
    pub(crate) talk: V::Target<TalkDomain>,
}

impl<V: InferenceDomainMapValueProjection> InferenceDomainMap for InferenceDomainMapImpl<V> {
    type Group = InferenceDomainGroupImpl;
    type ValueProjection = V;
}
