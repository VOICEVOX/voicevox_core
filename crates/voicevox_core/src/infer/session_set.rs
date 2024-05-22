use std::{collections::HashMap, fmt::Display, marker::PhantomData, sync::Arc};

use anyhow::bail;
use enum_map::{Enum as _, EnumMap};
use itertools::Itertools as _;

use crate::error::ErrorRepr;

use super::{
    model_file, InferenceDomain, InferenceInputSignature, InferenceOperation, InferenceRuntime,
    InferenceSessionOptions, InferenceSignature, ParamInfo,
};

pub(crate) struct InferenceSessionSet<R: InferenceRuntime, D: InferenceDomain>(
    EnumMap<D::Operation, Arc<std::sync::Mutex<R::Session>>>,
);

impl<R: InferenceRuntime, D: InferenceDomain> InferenceSessionSet<R, D> {
    pub(crate) fn new(
        model_bytes: &EnumMap<D::Operation, Vec<u8>>,
        options: &EnumMap<D::Operation, InferenceSessionOptions>,
    ) -> anyhow::Result<Self> {
        let mut sessions = model_bytes
            .iter()
            .map(|(op, model_bytes)| {
                let (expected_input_param_infos, expected_output_param_infos) =
                    <D::Operation as InferenceOperation>::PARAM_INFOS[op];

                let (sess, actual_input_param_infos, actual_output_param_infos) =
                    R::new_session(|| model_file::decrypt(model_bytes), options[op])?;

                check_param_infos(expected_input_param_infos, &actual_input_param_infos)?;
                check_param_infos(expected_output_param_infos, &actual_output_param_infos)?;

                Ok((op.into_usize(), std::sync::Mutex::new(sess).into()))
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
                        Some(ndim) => "[]".repeat(ndim),
                        None => "[]...".to_owned(),
                    };
                    format!("{name}: {dt}{brackets}")
                })
                .join(", ")
        }
    }
}

impl<R: InferenceRuntime, D: InferenceDomain> InferenceSessionSet<R, D> {
    pub(crate) fn get<I>(&self) -> InferenceSessionCell<R, I>
    where
        I: InferenceInputSignature,
        I::Signature: InferenceSignature<Domain = D>,
    {
        InferenceSessionCell {
            inner: self.0[I::Signature::OPERATION].clone(),
            marker: PhantomData,
        }
    }
}

pub(crate) struct InferenceSessionCell<R: InferenceRuntime, I> {
    inner: Arc<std::sync::Mutex<R::Session>>,
    marker: PhantomData<fn(I)>,
}

impl<R: InferenceRuntime, I: InferenceInputSignature> InferenceSessionCell<R, I> {
    pub(crate) fn run(
        self,
        input: I,
    ) -> crate::Result<<I::Signature as InferenceSignature>::Output> {
        let inner = &mut self.inner.lock().unwrap();
        (|| R::run(input.make_run_context::<R>(inner)?)?.try_into())()
            .map_err(ErrorRepr::InferenceFailed)
            .map_err(Into::into)
    }
}
