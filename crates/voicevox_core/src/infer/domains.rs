mod talk;

pub(crate) use self::talk::{
    DecodeInput, DecodeOutput, PredictDurationInput, PredictDurationOutput, PredictIntonationInput,
    PredictIntonationOutput, TalkDomain, TalkOperation,
};

use super::InferenceDomainAssociation;

pub(crate) struct ByInferenceDomain<A: InferenceDomainAssociation> {
    pub(crate) talk: A::Target<TalkDomain>,
}
