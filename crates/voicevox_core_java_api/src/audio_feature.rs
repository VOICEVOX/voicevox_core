use std::sync::Arc;

use jni::{
    JNIEnv,
    objects::{JClass, JObject},
    sys::{jint, jlong},
};
use voicevox_core::{
    __internal::interop::{AudioFeatureOutOfBoundsError, AudioFeatureSliceExt as _},
    AudioFeature,
};

use crate::common::{JavaApiError, throw_if_err};

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_AudioFeature_rsLength<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jint {
    throw_if_err(env, 0, |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.AudioFeature.handle` must correspond to
            //   `Arc<voicevox_core::AudioFeature>`.
            type RustField = Arc<AudioFeature>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?;

        Ok(internal.len())
    })
    .try_into()
    .expect("`AudioFeature` is not considered to be so large")
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_AudioFeature_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.AudioFeature.handle` must correspond to
            //   `Arc<voicevox_core::AudioFeature>`.
            type RustField = Arc<AudioFeature>;
            env.take_rust_field::<_, _, RustField>(&this, "handle")
        }?;
        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_AudioFeature_00024Slice_rsValidateRange<
    'local,
>(
    env: JNIEnv<'local>,
    _: JClass<'local>,
    audio_feature: JObject<'local>,
    current_start_inclusive: jint,
    current_end_exclusive: jint,
    slicing_start_inclusive: jint,
    slicing_end_exclusive: jint,
) -> jlong {
    throw_if_err(env, 0, |env| {
        let current_start_inclusive =
            usize::try_from(current_start_inclusive).expect("the invariant should be held");

        let current_end_exclusive =
            usize::try_from(current_end_exclusive).expect("the invariant should be held");

        let slicing_start_inclusive = usize::try_from(slicing_start_inclusive).map_err(|_| {
            JavaApiError::IndexOutOfBounds(format!("startInclusive = {slicing_start_inclusive}"))
        })?;

        let slicing_end_exclusive = usize::try_from(slicing_end_exclusive).map_err(|_| {
            JavaApiError::IndexOutOfBounds(format!("endExclusive = {slicing_end_exclusive}"))
        })?;

        let audio_feature = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.AudioFeature.handle` must correspond to
            //   `Arc<voicevox_core::AudioFeature>`.
            type RustField = Arc<AudioFeature>;
            env.get_rust_field::<_, _, RustField>(&audio_feature, "handle")
        }?;

        let slice = &audio_feature.slice(current_start_inclusive..current_end_exclusive);
        let slice = slice
            .try_slice(slicing_start_inclusive..slicing_end_exclusive)
            .map_err(|err| {
                let exc = match err {
                    AudioFeatureOutOfBoundsError::StartOutOfBounds { .. }
                    | AudioFeatureOutOfBoundsError::EndOutOfBounds { .. } => {
                        JavaApiError::IndexOutOfBounds
                    }
                    AudioFeatureOutOfBoundsError::StartExceedsEnd { .. }
                    | AudioFeatureOutOfBoundsError::Unknown { .. } => JavaApiError::IllegalArgument,
                };
                exc(err.to_string())
            })?;

        let new_start = jint::try_from(slice.start_offset()).expect("should not be so large");
        let new_end =
            jint::try_from(slice.start_offset() + slice.len()).expect("should not be so large");

        Ok(jlong::from(new_end) << 32 | jlong::from(new_start))
    })
}
