mod talk;

pub(crate) use self::talk::{
    DecodeInput, DecodeOutput, PredictDurationInput, PredictDurationOutput, PredictIntonationInput,
    PredictIntonationOutput, TalkDomain, TalkOperation,
};

use super::{
    ConvertInferenceDomainAssociationTarget, InferenceDomainAssociation, InferenceDomainGroup,
    InferenceDomainMap,
};

pub(crate) enum InferenceDomainGroupImpl {}

impl InferenceDomainGroup for InferenceDomainGroupImpl {
    type Map<A: InferenceDomainAssociation> = InferenceDomainMapImpl<A>;
}

pub(crate) struct InferenceDomainMapImpl<A: InferenceDomainAssociation> {
    pub(crate) talk: A::Target<TalkDomain>,
}

impl<A: InferenceDomainAssociation> InferenceDomainMap<A> for InferenceDomainMapImpl<A> {
    type Group = InferenceDomainGroupImpl;

    fn try_ref_map<
        F: ConvertInferenceDomainAssociationTarget<Self::Group, A, A2, E>,
        A2: InferenceDomainAssociation,
        E,
    >(
        &self,
        f: F,
    ) -> Result<<Self::Group as InferenceDomainGroup>::Map<A2>, E> {
        let talk = f.try_ref_map(&self.talk)?;
        Ok(InferenceDomainMapImpl { talk })
    }
}
