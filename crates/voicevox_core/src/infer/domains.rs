mod talk;

pub(crate) use self::talk::{
    DecodeInput, DecodeOutput, PredictDurationInput, PredictDurationOutput, PredictIntonationInput,
    PredictIntonationOutput, TalkDomain, TalkOperation,
};

use super::{
    InferenceDomainAssociation, InferenceDomainAssociationTargetFunction,
    InferenceDomainAssociationTargetPredicate, InferenceDomainGroup, InferenceDomainMap,
};

pub(crate) enum InferenceDomainGroupImpl {}

impl InferenceDomainGroup for InferenceDomainGroupImpl {
    type Map<A: InferenceDomainAssociation> = InferenceDomainMapImpl<A>;
}

pub(crate) struct InferenceDomainMapImpl<A: InferenceDomainAssociation> {
    pub(crate) talk: A::Target<TalkDomain>,
}

impl<A: InferenceDomainAssociation> InferenceDomainMap for InferenceDomainMapImpl<A> {
    type Group = InferenceDomainGroupImpl;
    type Association = A;

    fn any(
        &self,
        p: impl InferenceDomainAssociationTargetPredicate<InputAssociation = Self::Association>,
    ) -> bool {
        p.test(&self.talk)
    }

    fn try_ref_map<
        F: InferenceDomainAssociationTargetFunction<
            Group = Self::Group,
            InputAssociation = Self::Association,
        >,
    >(
        &self,
        f: F,
    ) -> Result<<Self::Group as InferenceDomainGroup>::Map<F::OutputAssociation>, F::Error> {
        let talk = f.apply(&self.talk)?;
        Ok(InferenceDomainMapImpl { talk })
    }
}
