use std::{
    collections::HashMap,
    ffi::CString,
    fmt::Debug,
    ops::DerefMut,
    path::Path,
    sync::{Arc, LazyLock},
};

use camino::Utf8Path;
use duplicate::duplicate_item;
use easy_ext::ext;
use ref_cast::ref_cast_custom;
use voicevox_core::{InitializeOptions, Result, VoiceModelId};

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
        let metas = &self.body().metas();
        CString::new(serde_json::to_string(metas).unwrap()).unwrap()
    }
}

impl VoicevoxVoiceModelFile {
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<&'static Self> {
        let model = voicevox_core::blocking::VoiceModelFile::open(path)?;
        let metas = CString::new(serde_json::to_string(model.metas()).unwrap()).unwrap();
        Ok(Self::new(VoiceModelFileWithMetas { model, metas }))
    }
}

pub(crate) struct VoiceModelFileWithMetas {
    pub(crate) model: voicevox_core::blocking::VoiceModelFile,
    pub(crate) metas: CString,
}

pub(crate) trait CApiObject: From<u32> + Into<u32> + Copy + Debug {
    type Body: 'static;

    fn heads() -> &'static boxcar::Vec<Self>;
    fn bodies() -> &'static std::sync::Mutex<HashMap<u32, Arc<Self::Body>>>;

    fn new(body: Self::Body) -> &'static Self {
        let i = Self::heads().push_with(|i| to_id(i).into());
        Self::lock_bodies().insert(to_id(i), body.into());
        return &Self::heads()[i];

        fn to_id(i: usize) -> u32 {
            i.try_into().expect("too large")
        }
    }

    /// # Panics
    ///
    /// 次の場合にパニックする。
    ///
    /// * `self`に対して以前にこの関数を呼んでいた場合
    /// * `self`がまだ他で利用中である場合
    fn drop_body(self) {
        let body = Self::lock_bodies()
            .remove(&self.into())
            .unwrap_or_else(|| self.panic_for_deleted());
        drop(Arc::into_inner(body).unwrap_or_else(|| self.panic_for_in_use()));
    }

    /// # Panics
    ///
    /// `drop_body`を呼んでいるとパニックする。
    fn body(self) -> Arc<Self::Body> {
        Self::lock_bodies()
            .get(&self.into())
            .unwrap_or_else(|| self.panic_for_deleted())
            .clone()
    }
}

#[ext]
impl<T: CApiObject> T {
    fn lock_bodies() -> impl DerefMut<Target = HashMap<u32, Arc<Self::Body>>> {
        Self::bodies().lock().unwrap_or_else(|e| panic!("{e}"))
    }

    fn panic_for_deleted(self) -> ! {
        panic!("`{self:?}`は既に破棄されています");
    }

    fn panic_for_in_use(self) -> ! {
        panic!("`{self:?}`が破棄されようとしましたが、これはまだ利用中です");
    }
}

#[duplicate_item(
    H                          B;
    [ OpenJtalkRc ]            [ voicevox_core::blocking::OpenJtalk ];
    [ VoicevoxUserDict ]       [ voicevox_core::blocking::UserDict ];
    [ VoicevoxSynthesizer ]    [ voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk> ];
    [ VoicevoxVoiceModelFile ] [ VoiceModelFileWithMetas ];
)]
impl CApiObject for H {
    type Body = B;

    fn heads() -> &'static boxcar::Vec<Self> {
        static HEADS: boxcar::Vec<H> = boxcar::Vec::new();
        &HEADS
    }

    fn bodies() -> &'static std::sync::Mutex<HashMap<u32, Arc<Self::Body>>> {
        static BODIES: LazyLock<std::sync::Mutex<HashMap<u32, Arc<B>>>> =
            LazyLock::new(Default::default);
        &BODIES
    }
}
