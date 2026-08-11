mod frame_decode;
mod singing_teacher;
pub(crate) mod streaming_talk;
pub(crate) mod talk;

use std::fmt::Debug;

use educe::Educe;
use serde::{Deserialize, Deserializer};

pub(crate) use self::{
    frame_decode::{FrameDecodeDomain, FrameDecodeOperation, SfDecodeInput, SfDecodeOutput},
    singing_teacher::{
        PredictSingConsonantLengthInput, PredictSingConsonantLengthOutput, PredictSingF0Input,
        PredictSingF0Output, PredictSingVolumeInput, PredictSingVolumeOutput, SingingTeacherDomain,
        SingingTeacherOperation,
    },
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
        bound = "V: InferenceDomainMapValues, V::Talk: Clone, V::StreamingTalk: Clone, V::SingingTeacher: Clone, V::FrameDecode: Clone"
    ),
    Debug(
        bound = "V: InferenceDomainMapValues, V::Talk: Debug, V::StreamingTalk: Debug, V::SingingTeacher: Debug, V::FrameDecode: Debug"
    )
)]
pub(crate) struct InferenceDomainMap<V: InferenceDomainMapValues + ?Sized> {
    pub(crate) talk: V::Talk,
    pub(crate) streaming_talk: V::StreamingTalk,
    pub(crate) singing_teacher: V::SingingTeacher,
    pub(crate) frame_decode: V::FrameDecode,
}

impl<T, X, S, F> InferenceDomainMap<(T, X, S, F)> {
    pub(in super::super) fn each_ref(&self) -> InferenceDomainMap<(&T, &X, &S, &F)> {
        let talk = &self.talk;
        let streaming_talk = &self.streaming_talk;
        let singing_teacher = &self.singing_teacher;
        let frame_decode = &self.frame_decode;
        InferenceDomainMap {
            talk,
            streaming_talk,
            singing_teacher,
            frame_decode,
        }
    }

    pub(in super::super) fn map<
        T2,
        X2,
        S2,
        F2,
        Ft: FnOnce(T) -> T2,
        Fx: FnOnce(X) -> X2,
        Fs: FnOnce(S) -> S2,
        Ff: FnOnce(F) -> F2,
    >(
        self,
        fs: InferenceDomainMap<(Ft, Fx, Fs, Ff)>,
    ) -> InferenceDomainMap<(T2, X2, S2, F2)> {
        let talk = (fs.talk)(self.talk);
        let streaming_talk = (fs.streaming_talk)(self.streaming_talk);
        let singing_teacher = (fs.singing_teacher)(self.singing_teacher);
        let frame_decode = (fs.frame_decode)(self.frame_decode);
        InferenceDomainMap {
            talk,
            streaming_talk,
            singing_teacher,
            frame_decode,
        }
    }
}

impl<T, X, S, F, E> InferenceDomainMap<(Result<T, E>, Result<X, E>, Result<S, E>, Result<F, E>)> {
    pub(in super::super) fn collect(self) -> Result<InferenceDomainMap<(T, X, S, F)>, E> {
        let talk = self.talk?;
        let streaming_talk = self.streaming_talk?;
        let singing_teacher = self.singing_teacher?;
        let frame_decode = self.frame_decode?;
        Ok(InferenceDomainMap {
            talk,
            streaming_talk,
            singing_teacher,
            frame_decode,
        })
    }
}

impl<'de, V: InferenceDomainMapValues + ?Sized> Deserialize<'de> for InferenceDomainMap<V>
where
    V::Talk: Deserialize<'de>,
    V::StreamingTalk: Deserialize<'de>,
    V::SingingTeacher: Deserialize<'de>,
    V::FrameDecode: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Repr {
            talk,
            streaming_talk,
            singing_teacher,
            frame_decode,
        } = Repr::deserialize(deserializer)?;
        return Ok(Self {
            talk,
            streaming_talk,
            singing_teacher,
            frame_decode,
        });

        #[derive(Deserialize)]
        struct Repr<T, E, S, F> {
            talk: T,
            streaming_talk: E,
            singing_teacher: S,
            frame_decode: F,
        }
    }
}

macro_rules! inference_domain_map {
    ($elem:expr $(,)?) => {
        crate::core::infer::domains::InferenceDomainMap {
            talk: $elem,
            streaming_talk: $elem,
            singing_teacher: $elem,
            frame_decode: $elem,
        }
    };
}
pub(crate) use inference_domain_map;

pub(crate) trait InferenceDomainMapValues {
    type Talk;
    type StreamingTalk;
    type SingingTeacher;
    type FrameDecode;
}

impl<T, X, S, F> InferenceDomainMapValues for (T, X, S, F) {
    type Talk = T;
    type StreamingTalk = X;
    type SingingTeacher = S;
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
                where $arg = crate::core::infer::domains::SingingTeacherDomain as crate::core::infer::InferenceDomain
            ),
            ::macros::substitute_type!(
                $body
                where $arg = crate::core::infer::domains::FrameDecodeDomain as crate::core::infer::InferenceDomain
            ),
        )
    };
}
pub(crate) use inference_domain_map_values;
