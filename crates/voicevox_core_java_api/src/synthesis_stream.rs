use std::sync::{Arc, Mutex};

use jni::{
    JNIEnv,
    objects::JObject,
    sys::{JNI_FALSE, JNI_TRUE, jboolean, jbyteArray},
};
use ouroboros::self_referencing;
use voicevox_core::blocking::{OpenJtalk, Synthesizer};

use crate::common::{Closable, HasJavaClassIdent, throw_if_err};

pub(crate) type SynthesisStream = Arc<Closable<SynthesisStreamBody>>;

/// `Synthesizer`を借用する`SynthesisStream`を、所有権つきで保持するための構造体。
///
/// `Arc<Synthesizer>`を持つため、Java側で`Synthesizer`がdropされた後でも、このストリームは使い続けられる。
#[self_referencing]
struct SynthesisStreamInner {
    synthesizer: Arc<Synthesizer<OpenJtalk>>,

    #[borrows(synthesizer)]
    #[covariant]
    stream: voicevox_core::blocking::SynthesisStream<'this>,
}

/// [`Closable`]に入れるための、[`SynthesisStreamInner`]のラッパー。
///
/// [`Closable::read`]は共有参照しか返さないので、`next`のための可変参照は[`Mutex`]で得る。
pub(crate) struct SynthesisStreamBody(Mutex<SynthesisStreamInner>);

impl SynthesisStreamBody {
    pub(crate) fn try_new<E>(
        synthesizer: Arc<Synthesizer<OpenJtalk>>,
        stream: impl for<'this> FnOnce(
            &'this Arc<Synthesizer<OpenJtalk>>,
        )
            -> Result<voicevox_core::blocking::SynthesisStream<'this>, E>,
    ) -> Result<Self, E> {
        SynthesisStreamInner::try_new(synthesizer, stream).map(|inner| Self(Mutex::new(inner)))
    }
}

impl HasJavaClassIdent for SynthesisStreamBody {
    const JAVA_CLASS_IDENT: &str = "SynthesisStream";
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_SynthesisStream_rsHasNext<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jboolean {
    throw_if_err(env, JNI_FALSE, |env| {
        // SAFETY:
        // - The safety contract must be upheld by the caller.
        // - `jp.hiroshiba.voicevoxcore.blocking.SynthesisStream.handle` must correspond to
        //   `self::SynthesisStream`.
        let internal =
            unsafe { env.get_rust_field::<_, _, SynthesisStream>(&this, "handle") }?.clone();
        let internal = internal.read()?;

        let remaining = internal
            .0
            .lock()
            .unwrap_or_else(|e| panic!("{e}"))
            .borrow_stream()
            .size_hint()
            .0;

        Ok(if remaining > 0 { JNI_TRUE } else { JNI_FALSE })
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_SynthesisStream_rsNext<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jbyteArray {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        // SAFETY:
        // - The safety contract must be upheld by the caller.
        // - `jp.hiroshiba.voicevoxcore.blocking.SynthesisStream.handle` must correspond to
        //   `self::SynthesisStream`.
        let internal =
            unsafe { env.get_rust_field::<_, _, SynthesisStream>(&this, "handle") }?.clone();
        let internal = internal.read()?;

        let pcm = internal
            .0
            .lock()
            .unwrap_or_else(|e| panic!("{e}"))
            .with_stream_mut(|stream| stream.next());

        match pcm {
            Some(pcm) => Ok(env.byte_array_from_slice(&pcm?)?.into_raw()),
            None => Ok(std::ptr::null_mut()),
        }
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_SynthesisStream_rsClose<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        // `VoiceModelFile`と違い`take_rust_field`ではなく`get_rust_field`を使う。
        // `Closeable#close`は複数回呼べる必要があり、また`close`の後の`next`は
        // `IllegalStateException`にしたいため、`handle`は`rsDrop`まで残しておく。
        //
        // SAFETY:
        // - The safety contract must be upheld by the caller.
        // - `jp.hiroshiba.voicevoxcore.blocking.SynthesisStream.handle` must correspond to
        //   `self::SynthesisStream`.
        let internal =
            unsafe { env.get_rust_field::<_, _, SynthesisStream>(&this, "handle") }?.clone();
        internal.close();
        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_SynthesisStream_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        // SAFETY:
        // - The safety contract must be upheld by the caller.
        // - `jp.hiroshiba.voicevoxcore.blocking.SynthesisStream.handle` must correspond to
        //   `self::SynthesisStream`.
        unsafe { env.take_rust_field::<_, _, SynthesisStream>(&this, "handle") }?;
        Ok(())
    })
}
