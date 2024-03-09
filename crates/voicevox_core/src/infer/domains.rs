mod talk;

pub(crate) use self::talk::{
    DecodeInput, DecodeOutput, PredictDurationInput, PredictDurationOutput, PredictIntonationInput,
    PredictIntonationOutput, TalkDomain, TalkOperation,
};

use super::{
    ConvertInferenceDomainAssociationTarget, InferenceDomainAssociation, InferenceDomainSet,
};

pub(crate) enum InferenceDomainSetImpl {}

impl InferenceDomainSet for InferenceDomainSetImpl {
    type ByInferenceDomain<A: InferenceDomainAssociation> = ByInferenceDomain<A>;

    fn try_ref_map<
        F: ConvertInferenceDomainAssociationTarget<Self, A1, A2, E>,
        A1: InferenceDomainAssociation,
        A2: InferenceDomainAssociation,
        E,
    >(
        by_domain: &Self::ByInferenceDomain<A1>,
        f: F,
    ) -> Result<Self::ByInferenceDomain<A2>, E> {
        let talk = f.try_ref_map(&by_domain.talk)?;
        Ok(ByInferenceDomain { talk })
    }
}

pub(crate) struct ByInferenceDomain<A: InferenceDomainAssociation> {
    pub(crate) talk: A::Target<TalkDomain>,
}
