use std::{
    any,
    collections::HashMap,
    ffi::CString,
    fmt::{Debug, Display},
    mem,
    ops::{Deref, DerefMut},
    path::Path,
    sync::{Arc, LazyLock},
};

use camino::Utf8Path;
use duplicate::duplicate_item;
use easy_ext::ext;
use ref_cast::ref_cast_custom;
use tracing::warn;
use voicevox_core::{InitializeOptions, Result, SpeakerMeta, VoiceModelId};

use crate::{
    helpers::CApiResult, OpenJtalkRc, VoicevoxOnnxruntime, VoicevoxSynthesizer, VoicevoxUserDict,
    VoicevoxVoiceModelFile,
};

// FIXME: 中身(Rust API)を直接操作するかラッパーメソッド越しにするのかが混在していて、一貫性を
// 欠いている

impl VoicevoxOnnxruntime {
    #[cfg(feature = "load-onnxruntime")]
    pub(crate) fn lib_versioned_filename() -> &'static std::ffi::CStr {
        to_cstr!(voicevox_core::blocking::Onnxruntime::LIB_VERSIONED_FILENAME)
    }

    #[cfg(feature = "load-onnxruntime")]
    pub(crate) fn lib_unversioned_filename() -> &'static std::ffi::CStr {
        to_cstr!(voicevox_core::blocking::Onnxruntime::LIB_UNVERSIONED_FILENAME)
    }

    #[ref_cast_custom]
    fn new(rust: &voicevox_core::blocking::Onnxruntime) -> &Self;

    pub(crate) fn get() -> Option<&'static Self> {
        voicevox_core::blocking::Onnxruntime::get().map(Self::new)
    }

    #[cfg(feature = "load-onnxruntime")]
    pub(crate) fn load_once(filename: &std::ffi::CStr) -> CApiResult<&'static Self> {
        use crate::helpers::ensure_utf8;

        let inner = voicevox_core::blocking::Onnxruntime::load_once()
            .filename(ensure_utf8(filename)?)
            .exec()?;
        Ok(Self::new(inner))
    }

    #[cfg(feature = "link-onnxruntime")]
    pub(crate) fn init_once() -> CApiResult<&'static Self> {
        let inner = voicevox_core::blocking::Onnxruntime::init_once()?;
        Ok(Self::new(inner))
    }
}

#[cfg(feature = "load-onnxruntime")]
macro_rules! to_cstr {
    ($s:expr) => {{
        const __RUST_STR: &str = $s;
        static __C_STR: &[u8] = const_format::concatcp!(__RUST_STR, '\0').as_bytes();

        std::ffi::CStr::from_bytes_with_nul(__C_STR)
            .unwrap_or_else(|e| panic!("{__RUST_STR:?} should not contain `\\0`: {e}"))
    }};
}
#[cfg(feature = "load-onnxruntime")]
use to_cstr;

impl OpenJtalkRc {
    pub(crate) fn new(open_jtalk_dic_dir: impl AsRef<Utf8Path>) -> Result<&'static Self> {
        let body = voicevox_core::blocking::OpenJtalk::new(open_jtalk_dic_dir)?;
        Ok(<Self as CApiObject>::new(body))
    }
}

impl VoicevoxSynthesizer {
    pub(crate) fn new(
        onnxruntime: &'static VoicevoxOnnxruntime,
        open_jtalk: &OpenJtalkRc,
        options: &InitializeOptions,
    ) -> Result<&'static Self> {
        let body = voicevox_core::blocking::Synthesizer::new(
            &onnxruntime.0,
            (*open_jtalk.body()).clone(),
            options,
        )?;
        Ok(<Self as CApiObject>::new(body))
    }

    pub(crate) fn onnxruntime(&self) -> &'static VoicevoxOnnxruntime {
        VoicevoxOnnxruntime::new(self.body().onnxruntime())
    }

    pub(crate) fn load_voice_model(
        &self,
        model: &voicevox_core::blocking::VoiceModelFile,
    ) -> CApiResult<()> {
        self.body().load_voice_model(model)?;
        Ok(())
    }

    pub(crate) fn unload_voice_model(&self, model_id: VoiceModelId) -> Result<()> {
        self.body().unload_voice_model(model_id)?;
        Ok(())
    }

    pub(crate) fn metas(&self) -> CString {
        metas_to_json(&self.body().metas())
    }
}

impl VoicevoxVoiceModelFile {
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<&'static Self> {
        let model = voicevox_core::blocking::VoiceModelFile::open(path)?;
        Ok(Self::new(model))
    }

    pub(crate) fn metas(&self) -> CString {
        metas_to_json(self.body().metas())
    }
}

fn metas_to_json(metas: &[SpeakerMeta]) -> CString {
    let metas = serde_json::to_string(metas).expect("should not fail");
    CString::new(metas).expect("should not contain NUL")
}

/// プロセスの終わりまでデストラクトされない、ユーザーにオブジェクトとして貸し出す1-bit長の構造体。
///
/// インスタンスは次のような形。
///
/// ```
/// pub struct VoicevoxSynthesizer {
///    _padding: MaybeUninit<[u8; 1]>,
/// }
/// ```
///
/// `RustApi`そのものではなくこのトレイトのインスタンスをユーザーに渡すようにし、次のユーザー操作に対するセーフティネットを実現する。
///
/// 1. オブジェクトが他スレッドでアクセスされている最中に"delete"を試みる
/// 2. "delete"後に他の通常のメソッド関数の利用を試みる
/// 3. "delete"後に"delete"を試みる
///
/// ただし次のゲッター関数に関しては機能しない。
///
/// - `voicevox_voice_model_file_id`
/// - `voicevox_voice_model_file_get_metas_json`
pub(crate) trait CApiObject: Default + Debug {
    type RustApiObject: 'static;

    fn heads() -> &'static boxcar::Vec<Self>;

    #[expect(clippy::type_complexity, reason = "コメント書いてる")]
    fn bodies() -> &'static std::sync::Mutex<
        HashMap<
            usize, // `heads`の要素へのポインタのアドレス
            Arc<
                parking_lot::RwLock<
                    Option<Self::RustApiObject>, // `RwLock`をdropする直前まで`Some`
                >,
            >,
        >,
    >;

    fn new(body: Self::RustApiObject) -> &'static Self {
        assert!(mem::size_of::<Self>() > 0);

        let i = Self::heads().push(Default::default());
        let this = &Self::heads()[i];
        let body = parking_lot::RwLock::new(body.into()).into();
        Self::lock_bodies().insert(this as *const _ as _, body);
        this
    }

    /// # Panics
    ///
    /// 次の場合にパニックする。
    ///
    /// * `self`に対して以前にこの関数を呼んでいた場合
    /// * `self`がまだ他で利用中である場合
    fn drop_body(&self) {
        let body = Self::lock_bodies()
            .remove(&(self as *const _ as _))
            .unwrap_or_else(|| self.panic_for_deleted());

        drop(
            body.try_write_arc()
                .unwrap_or_else(|| {
                    warn!("{} is still in use. Waiting before closing", self.display());
                    body.write_arc()
                })
                .take()
                .unwrap_or_else(|| self.panic_for_deleted()),
        );
    }

    /// # Panics
    ///
    /// `drop_body`を呼んでいるとパニックする。
    fn body(&self) -> impl Deref<Target = Self::RustApiObject> {
        let body = Self::lock_bodies()
            .get(&(self as *const _ as _))
            .unwrap_or_else(|| self.panic_for_deleted())
            .read_arc();
        voicevox_core::__internal::interop::raii::try_map_guard(body, |body| {
            body.as_ref().ok_or(())
        })
        .unwrap_or_else(|()| self.panic_for_deleted())
    }
}

#[ext]
impl<T: CApiObject> T {
    fn lock_bodies(
    ) -> impl DerefMut<Target = HashMap<usize, Arc<parking_lot::RwLock<Option<Self::RustApiObject>>>>>
    {
        Self::bodies().lock().unwrap_or_else(|e| panic!("{e}"))
    }

    fn panic_for_deleted(&self) -> ! {
        let display = self.display();
        panic!("{display}は既に破棄されています");
    }

    fn display(&self) -> impl Display + '_ {
        let type_name = any::type_name::<Self>()
            .split("::")
            .last()
            .expect("should not empty");
        format!("`{type_name}` ({self:018p})")
    }
}

#[duplicate_item(
    H                          B;
    [ OpenJtalkRc ]            [ voicevox_core::blocking::OpenJtalk ];
    [ VoicevoxUserDict ]       [ voicevox_core::blocking::UserDict ];
    [ VoicevoxSynthesizer ]    [ voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk> ];
    [ VoicevoxVoiceModelFile ] [ voicevox_core::blocking::VoiceModelFile ];
)]
impl CApiObject for H {
    type RustApiObject = B;

    fn heads() -> &'static boxcar::Vec<Self> {
        static HEADS: boxcar::Vec<H> = boxcar::Vec::new();
        &HEADS
    }

    fn bodies() -> &'static std::sync::Mutex<
        HashMap<usize, Arc<parking_lot::RwLock<Option<Self::RustApiObject>>>>,
    > {
        #[expect(
            clippy::type_complexity,
            reason = "`CApiObject::bodies`の方でコメント書いてる"
        )]
        static BODIES: LazyLock<
            std::sync::Mutex<HashMap<usize, Arc<parking_lot::RwLock<Option<B>>>>>,
        > = LazyLock::new(Default::default);
        &BODIES
    }
}
