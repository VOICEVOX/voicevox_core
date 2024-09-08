mod talk;

use std::future::Future;

use educe::Educe;
use serde::{Deserialize, Deserializer};

pub(crate) use self::talk::{
    DecodeInput, DecodeOutput, PredictDurationInput, PredictDurationOutput, PredictIntonationInput,
    PredictIntonationOutput, TalkDomain, TalkOperation,
};

#[derive(Educe)]
// TODO: `bounds`に`V: ?Sized`も入れようとすると、よくわからない理由で弾かれる。最新版のeduce
// でもそうなのか？また最新版でも駄目だとしたら、弾いている理由は何なのか？
#[educe(Clone(bound = "V: InferenceDomainMapValues, V::Talk: Clone"))]
pub(crate) struct InferenceDomainMap<V: InferenceDomainMapValues + ?Sized> {
    pub(crate) talk: V::Talk,
}

impl<T> InferenceDomainMap<(T,)> {
    pub(crate) fn each_ref(&self) -> InferenceDomainMap<(&T,)> {
        InferenceDomainMap { talk: &self.talk }
    }

    pub(crate) fn map<T2, Ft: FnOnce(T) -> T2>(
        self,
        fs: InferenceDomainMap<(Ft,)>,
    ) -> InferenceDomainMap<(T2,)> {
        InferenceDomainMap {
            talk: (fs.talk)(self.talk),
        }
    }
}

impl<T, E> InferenceDomainMap<(Result<T, E>,)> {
    pub(crate) fn collect_results(self) -> Result<InferenceDomainMap<(T,)>, E> {
        let talk = self.talk?;
        Ok(InferenceDomainMap { talk })
    }
}

impl<Ft: Future<Output = Result<T, E>>, T, E> InferenceDomainMap<(Ft,)> {
    pub(crate) async fn collect_future_results(self) -> Result<InferenceDomainMap<(T,)>, E> {
        let talk = self.talk.await?;
        Ok(InferenceDomainMap { talk })
    }
}

impl<'de, V: InferenceDomainMapValues + ?Sized> Deserialize<'de> for InferenceDomainMap<V>
where
    V::Talk: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Repr { talk } = Repr::deserialize(deserializer)?;
        return Ok(Self { talk });

        #[derive(Deserialize)]
        struct Repr<T> {
            talk: T,
        }
    }
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
