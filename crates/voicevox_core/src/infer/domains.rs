mod talk;

pub(crate) use self::talk::{
    DecodeInput, DecodeOutput, PredictDurationInput, PredictDurationOutput, PredictIntonationInput,
    PredictIntonationOutput, TalkDomain, TalkOperation,
};

use super::{
    InferenceDomainGroup, InferenceDomainMap, InferenceDomainMapValuePredicate,
    InferenceDomainMapValueProjection, InferenceDomainMapValueTryFunction,
};

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

    fn any(
        &self,
        p: impl InferenceDomainMapValuePredicate<InputProjection = Self::ValueProjection>,
    ) -> bool {
        p.test(&self.talk)
    }

    fn ref_map<
        F: super::InferenceDomainMapValueFunction<
            Group = Self::Group,
            InputProjection = Self::ValueProjection,
        >,
    >(
        &self,
        f: F,
    ) -> <Self::Group as InferenceDomainGroup>::Map<F::OutputProjection> {
        let talk = f.apply(&self.talk);
        InferenceDomainMapImpl { talk }
    }

    fn try_ref_map<
        F: InferenceDomainMapValueTryFunction<
            Group = Self::Group,
            InputProjection = Self::ValueProjection,
        >,
    >(
        &self,
        f: F,
    ) -> Result<<Self::Group as InferenceDomainGroup>::Map<F::OutputProjection>, F::Error> {
        let talk = f.apply(&self.talk)?;
        Ok(InferenceDomainMapImpl { talk })
    }
}
