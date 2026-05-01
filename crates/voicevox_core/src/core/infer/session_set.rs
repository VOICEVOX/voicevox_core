use std::{collections::HashMap, fmt::Display, marker::PhantomData, sync::Arc};

use anyhow::bail;
use enum_map::{Enum as _, EnumMap};
use itertools::Itertools as _;

use crate::error::ErrorRepr;

use super::{
    super::voice_model::ModelBytes, InferenceDomain, InferenceInputSignature, InferenceOperation,
    InferenceRuntime, InferenceSessionOptions, InferenceSignature, ParamInfo,
};

pub(crate) struct InferenceSessionSet<R: InferenceRuntime, D: InferenceDomain>(
    EnumMap<D::Operation, Arc<R::Session>>,
);

impl<R: InferenceRuntime, D: InferenceDomain> InferenceSessionSet<R, D> {
    pub(in super::super) fn new(
        rt: &R,
        model_bytes: &EnumMap<D::Operation, ModelBytes>,
        options: &EnumMap<D::Operation, InferenceSessionOptions>,
    ) -> anyhow::Result<Self> {
        let mut sessions = model_bytes
            .iter()
            .map(|(op, model_bytes)| {
                let (expected_input_param_infos, expected_output_param_infos) =
                    <D::Operation as InferenceOperation>::PARAM_INFOS[op];

                let (sess, actual_input_param_infos, actual_output_param_infos) =
                    rt.new_session(model_bytes, options[op])?;

                check_param_infos(expected_input_param_infos, &actual_input_param_infos)?;
                check_param_infos(expected_output_param_infos, &actual_output_param_infos)?;

                Ok((op.into_usize(), sess.into()))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        return Ok(Self(EnumMap::<D::Operation, _>::from_fn(|k| {
            sessions.remove(&k.into_usize()).expect("should exist")
        })));

        fn check_param_infos<D: PartialEq + Display>(
            expected: &[ParamInfo<D>],
            actual: &[ParamInfo<D>],
        ) -> anyhow::Result<()> {
            if !(expected.len() == actual.len()
                && itertools::zip_eq(expected, actual)
                    .all(|(expected, actual)| expected.accepts(actual)))
            {
                let expected = display_param_infos(expected);
                let actual = display_param_infos(actual);
                bail!("expected {{{expected}}}, got {{{actual}}}")
            }
            Ok(())
        }

        fn display_param_infos(infos: &[ParamInfo<impl Display>]) -> impl Display {
            infos
                .iter()
                .map(|ParamInfo { name, dt, ndim }| {
                    let brackets = match *ndim {
                        Some(ndim) => &"[]".repeat(ndim),
                        None => "[]...",
                    };
                    format!("{name}: {dt}{brackets}")
                })
                .join(", ")
        }
    }
}

impl<R: InferenceRuntime, D: InferenceDomain> InferenceSessionSet<R, D> {
    pub(in super::super) fn get<I>(&self) -> InferenceSessionCell<R, I>
    where
        I: InferenceInputSignature<Signature: InferenceSignature<Domain = D>>,
    {
        InferenceSessionCell {
            inner: self.0[I::Signature::OPERATION].clone(),
            marker: PhantomData,
        }
    }
}

pub(in super::super) struct InferenceSessionCell<R: InferenceRuntime, I> {
    inner: Arc<R::Session>,
    marker: PhantomData<fn(I)>,
}

impl<R: InferenceRuntime, I: InferenceInputSignature> InferenceSessionCell<R, I> {
    pub(in super::super) async fn run<A: super::AsyncExt>(
        self,
        input: I,
        cancellable: A::Cancellable,
    ) -> crate::Result<<I::Signature as InferenceSignature>::Output> {
        async {
            let ctx = input.make_run_context::<R>(self.inner.clone())?;
            A::run_session::<R>(ctx, cancellable).await?.try_into()
        }
        .await
        .map_err(|source| ErrorRepr::RunModel { note: None, source })
        .map_err(Into::into)
    }
}
