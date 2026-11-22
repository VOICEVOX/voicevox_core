mod frame_decode;
pub(crate) mod singing_teacher_v1;
pub(crate) mod singing_teacher_v2;
pub(crate) mod streaming_talk;
pub(crate) mod talk;

use std::fmt::Debug;

use educe::Educe;
use serde::{Deserialize, Deserializer};

pub(crate) use self::{
    frame_decode::{FrameDecodeDomain, FrameDecodeOperation, SfDecodeInput, SfDecodeOutput},
    singing_teacher_v1::{SingingTeacherV1Domain, SingingTeacherV1Operation},
    singing_teacher_v2::{SingingTeacherV2Domain, SingingTeacherV2Operation},
    streaming_talk::{
        GenerateFullIntermediateInput, GenerateFullIntermediateOutput, RenderAudioSegmentInput,
        RenderAudioSegmentOutput, StreamingTalkDomain, StreamingTalkOperation,
    },
    talk::{DecodeInput, DecodeOutput, TalkDomain, TalkOperation},
};

#[derive(Educe)]
// TODO: `bounds`に`V: ?Sized`も入れようとすると、よくわからない理由で弾かれる。最新版のeduce
// でもそうなのか？また最新版でも駄目だとしたら、弾いている理由は何なのか？
#[educe(
    Clone(
        bound = "V: InferenceDomainMapValues, V::Talk: Clone, V::StreamingTalk: Clone, V::SingingTeacherV1: Clone, V::SingingTeacherV2: Clone, V::FrameDecode: Clone"
    ),
    Debug(
        bound = "V: InferenceDomainMapValues, V::Talk: Debug, V::StreamingTalk: Debug, V::SingingTeacherV1: Debug, V::SingingTeacherV2: Debug, V::FrameDecode: Debug"
    )
)]
pub(crate) struct InferenceDomainMap<V: InferenceDomainMapValues + ?Sized> {
    pub(crate) talk: V::Talk,
    pub(crate) streaming_talk: V::StreamingTalk,
    pub(crate) singing_teacher_v1: V::SingingTeacherV1,
    pub(crate) singing_teacher_v2: V::SingingTeacherV2,
    pub(crate) frame_decode: V::FrameDecode,
}

impl<T, X, S1, S2, F> InferenceDomainMap<(T, X, S1, S2, F)> {
    pub(in super::super) fn each_ref(&self) -> InferenceDomainMap<(&T, &X, &S1, &S2, &F)> {
        let talk = &self.talk;
        let streaming_talk = &self.streaming_talk;
        let singing_teacher_v1 = &self.singing_teacher_v1;
        let singing_teacher_v2 = &self.singing_teacher_v2;
        let frame_decode = &self.frame_decode;
        InferenceDomainMap {
            talk,
            streaming_talk,
            singing_teacher_v1,
            singing_teacher_v2,
            frame_decode,
        }
    }

    pub(in super::super) fn map<
        T2,
        X2,
        S1_2,
        S2_2,
        F2,
        Ft: FnOnce(T) -> T2,
        Fx: FnOnce(X) -> X2,
        Fs1: FnOnce(S1) -> S1_2,
        Fs2: FnOnce(S2) -> S2_2,
        Ff: FnOnce(F) -> F2,
    >(
        self,
        fs: InferenceDomainMap<(Ft, Fx, Fs1, Fs2, Ff)>,
    ) -> InferenceDomainMap<(T2, X2, S1_2, S2_2, F2)> {
        let talk = (fs.talk)(self.talk);
        let streaming_talk = (fs.streaming_talk)(self.streaming_talk);
        let singing_teacher_v1 = (fs.singing_teacher_v1)(self.singing_teacher_v1);
        let singing_teacher_v2 = (fs.singing_teacher_v2)(self.singing_teacher_v2);
        let frame_decode = (fs.frame_decode)(self.frame_decode);
        InferenceDomainMap {
            talk,
            streaming_talk,
            singing_teacher_v1,
            singing_teacher_v2,
            frame_decode,
        }
    }
}

impl<T, X, S1, S2, F, E>
    InferenceDomainMap<(
        Result<T, E>,
        Result<X, E>,
        Result<S1, E>,
        Result<S2, E>,
        Result<F, E>,
    )>
{
    #[expect(
        clippy::type_complexity,
        reason = "Clippy is just complaining about the 5-element tuple"
    )]
    pub(in super::super) fn collect(self) -> Result<InferenceDomainMap<(T, X, S1, S2, F)>, E> {
        let talk = self.talk?;
        let streaming_talk = self.streaming_talk?;
        let singing_teacher_v1 = self.singing_teacher_v1?;
        let singing_teacher_v2 = self.singing_teacher_v2?;
        let frame_decode = self.frame_decode?;
        Ok(InferenceDomainMap {
            talk,
            streaming_talk,
            singing_teacher_v1,
            singing_teacher_v2,
            frame_decode,
        })
    }
}

impl<'de, V: InferenceDomainMapValues + ?Sized> Deserialize<'de> for InferenceDomainMap<V>
where
    V::Talk: Deserialize<'de>,
    V::StreamingTalk: Deserialize<'de>,
    V::SingingTeacherV1: Deserialize<'de>,
    V::SingingTeacherV2: Deserialize<'de>,
    V::FrameDecode: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Repr {
            talk,
            streaming_talk,
            singing_teacher_v1,
            singing_teacher_v2,
            frame_decode,
        } = Repr::deserialize(deserializer)?;
        return Ok(Self {
            talk,
            streaming_talk,
            singing_teacher_v1,
            singing_teacher_v2,
            frame_decode,
        });

        #[derive(Deserialize)]
        struct Repr<T, E, S1, S2, F> {
            talk: T,
            streaming_talk: E,
            singing_teacher_v1: S1,
            singing_teacher_v2: S2,
            frame_decode: F,
        }
    }
}

macro_rules! inference_domain_map {
    ($elem:expr $(,)?) => {
        crate::core::infer::domains::InferenceDomainMap {
            talk: $elem,
            streaming_talk: $elem,
            singing_teacher_v1: $elem,
            singing_teacher_v2: $elem,
            frame_decode: $elem,
        }
    };
}
pub(crate) use inference_domain_map;

pub(crate) trait InferenceDomainMapValues {
    type Talk;
    type StreamingTalk;
    type SingingTeacherV1;
    type SingingTeacherV2;
    type FrameDecode;
}

impl<T, X, S1, S2, F> InferenceDomainMapValues for (T, X, S1, S2, F) {
    type Talk = T;
    type StreamingTalk = X;
    type SingingTeacherV1 = S1;
    type SingingTeacherV2 = S2;
    type FrameDecode = F;
}

macro_rules! inference_domain_map_values {
    (for<$arg:ident> $body:ty) => {
        (
            ::macros::substitute_type!(
                $body
                where $arg = crate::core::infer::domains::TalkDomain as crate::core::infer::InferenceDomain
            ),
            ::macros::substitute_type!(
                $body
                where $arg = crate::core::infer::domains::StreamingTalkDomain as crate::core::infer::InferenceDomain
            ),
            ::macros::substitute_type!(
                $body
                where $arg = crate::core::infer::domains::SingingTeacherV1Domain as crate::core::infer::InferenceDomain
            ),
            ::macros::substitute_type!(
                $body
                where $arg = crate::core::infer::domains::SingingTeacherV2Domain as crate::core::infer::InferenceDomain
            ),
            ::macros::substitute_type!(
                $body
                where $arg = crate::core::infer::domains::FrameDecodeDomain as crate::core::infer::InferenceDomain
            ),
        )
    };
}
pub(crate) use inference_domain_map_values;
