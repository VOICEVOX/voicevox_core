//! 音声モデル（ VVM ファイル）。
//!
//! VVM ファイルの定義と形式は[ドキュメント](../../../docs/guide/dev/vvm.md)を参照。

use std::{
    collections::HashMap,
    fmt::{self, Debug},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context as _};
use derive_more::From;
use easy_ext::ext;
use enum_map::{Enum, EnumMap};
use futures_io::{AsyncBufRead, AsyncRead, AsyncSeek};
use futures_util::future::{OptionFuture, TryFutureExt as _};
use itertools::Itertools as _;
use ouroboros::self_referencing;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    asyncs::{Async, Mutex as _},
    error::{LoadModelError, LoadModelErrorKind, LoadModelResult},
    CharacterMeta, StyleMeta, StyleType, VoiceModelMeta,
};

use super::{
    infer::{
        domains::{
            inference_domain_map_values, ExperimentalTalkDomain, FrameDecodeDomain,
            InferenceDomainMap, SingingTeacherDomain, TalkDomain,
        },
        InferenceDomain,
    },
    manifest::{Manifest, ManifestDomains, ModelFile, ModelFileType, StyleIdToInnerVoiceId},
};

pub(super) type ModelBytesWithInnerVoiceIdsByDomain = inference_domain_map_values!(
    for<D> Option<(StyleIdToInnerVoiceId, EnumMap<D::Operation, ModelBytes>)>
);

/// 音声モデルID。
///
/// `Synthesizer`はこのIDをキーとして、音声モデルのロード・アンロードを行う。
///
/// 同じIDを持つ複数のVVMファイルがあるときは、ファイルとして新しい方を常に使うことが推奨される。[VOICEVOX/voicevox_vvm]で管理されているVVMでは、次の方針が取られている。
///
/// - VVMに含まれる声が変化せず、軽微な修正のみのときはIDを使い回してリリースする。
/// - VVMに含まれる声が明確に変化するかもしくは削除されるような実質的な変更のときは、新しいIDを割り振ってリリースする。
///
/// これ以外は未定であり、更なるルールについては[VOICEVOX/voicevox_vvm#19]で議論される予定。
///
/// [VOICEVOX/voicevox_vvm]: https://github.com/VOICEVOX/voicevox_vvm
/// [VOICEVOX/voicevox_vvm#19]: https://github.com/VOICEVOX/voicevox_vvm/issues/19
#[cfg_attr(doc, doc(alias = "VoicevoxVoiceModelId"))]
#[derive(
    PartialEq,
    Eq,
    Clone,
    Copy,
    Ord,
    Hash,
    PartialOrd,
    Deserialize,
    derive_more::Display,
    Debug,
    From,
)]
pub struct VoiceModelId(pub Uuid);

#[self_referencing]
pub(crate) struct Inner<A: Async> {
    header: Arc<VoiceModelHeader>,

    #[borrows(header)]
    #[not_covariant]
    inference_model_entries: InferenceDomainMap<InferenceModelEntries<'this>>,

    zip: A::Mutex<A::RoFile>,
}

impl<A: Async> Inner<A> {
    async fn open(path: impl AsRef<Path>) -> crate::Result<Self> {
        const MANIFEST_FILENAME: &str = "manifest.json";

        let path = path.as_ref();

        let error = |context, source| LoadModelError {
            path: path.to_owned(),
            context,
            source: Some(source),
        };

        let mut zip = A::open_zip(path)
            .await
            .map_err(|source| error(LoadModelErrorKind::OpenZipFile, source))?;

        let indices = zip.entry_indices_by_utf8_filenames();
        let find_entry_index = |filename: &str| {
            indices
                .get(filename)
                .with_context(|| "could not find `{filename}`")
                .copied()
        };

        let manifest = &async {
            let idx = find_entry_index(MANIFEST_FILENAME)?;
            zip.read_file(idx).await
        }
        .await
        .map_err(|source| {
            error(
                LoadModelErrorKind::ReadZipEntry {
                    filename: MANIFEST_FILENAME.to_owned(),
                },
                source,
            )
        })?;
        let manifest = serde_json::from_slice::<Manifest>(manifest)
            .map_err(|source| error(LoadModelErrorKind::InvalidModelFormat, source.into()))?;

        let metas = &async {
            let idx = find_entry_index(manifest.metas_filename())?;
            zip.read_file(idx).await
        }
        .await
        .map_err(|source| {
            error(
                LoadModelErrorKind::ReadZipEntry {
                    filename: manifest.metas_filename().clone(),
                },
                source,
            )
        })?;

        let header = VoiceModelHeader::new(manifest, metas, path)?.into();

        return InnerTryBuilder {
            header,
            inference_model_entries_builder: |header| {
                let VoiceModelHeader { manifest, .. } = &**header;
                manifest
                    .domains()
                    .each_ref()
                    .map(InferenceDomainMap {
                        talk: |talk| {
                            talk.as_ref()
                                .map(|manifest| {
                                    let indices = EnumMap::from_fn(|k| &manifest[k]).try_map(
                                        |_, ModelFile { filename, .. }| find_entry_index(filename),
                                    )?;
                                    Ok(InferenceModelEntry { indices, manifest })
                                })
                                .transpose()
                                .map_err(move |source| {
                                    error(
                                        LoadModelErrorKind::ReadZipEntry {
                                            filename: MANIFEST_FILENAME.to_owned(),
                                        },
                                        source,
                                    )
                                })
                        },
                        experimental_talk: |talk| {
                            talk.as_ref()
                                .map(|manifest| {
                                    let indices = EnumMap::from_fn(|k| &manifest[k]).try_map(
                                        |_, ModelFile { filename, .. }| find_entry_index(filename),
                                    )?;
                                    Ok(InferenceModelEntry { indices, manifest })
                                })
                                .transpose()
                                .map_err(move |source| {
                                    error(
                                        LoadModelErrorKind::ReadZipEntry {
                                            filename: MANIFEST_FILENAME.to_owned(),
                                        },
                                        source,
                                    )
                                })
                        },
                        singing_teacher: |singing_teacher| {
                            singing_teacher
                                .as_ref()
                                .map(|manifest| {
                                    let indices = EnumMap::from_fn(|k| &manifest[k]).try_map(
                                        |_, ModelFile { filename, .. }| find_entry_index(filename),
                                    )?;
                                    Ok(InferenceModelEntry { indices, manifest })
                                })
                                .transpose()
                                .map_err(move |source| {
                                    error(
                                        LoadModelErrorKind::ReadZipEntry {
                                            filename: MANIFEST_FILENAME.to_owned(),
                                        },
                                        source,
                                    )
                                })
                        },
                        frame_decode: |frame_decode| {
                            frame_decode
                                .as_ref()
                                .map(|manifest| {
                                    let indices = EnumMap::from_fn(|k| &manifest[k]).try_map(
                                        |_, ModelFile { filename, .. }| find_entry_index(filename),
                                    )?;
                                    Ok(InferenceModelEntry { indices, manifest })
                                })
                                .transpose()
                                .map_err(move |source| {
                                    error(
                                        LoadModelErrorKind::ReadZipEntry {
                                            filename: MANIFEST_FILENAME.to_owned(),
                                        },
                                        source,
                                    )
                                })
                        },
                    })
                    .collect()
                    .map_err(crate::Error::from)
            },
            zip: zip.into_inner().into_inner().into(),
        }
        .try_build();

        #[ext]
        impl<K: Enum, V> EnumMap<K, V> {
            fn try_map<V2, E>(
                self,
                f: impl FnMut(K, V) -> Result<V2, E>,
            ) -> Result<EnumMap<K, V2>, E> {
                let mut elems = self
                    .map(f)
                    .into_iter()
                    .map(|(_, r)| r.map(Some))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(EnumMap::<K, _>::from_fn(|key| {
                    let key = key.into_usize();
                    elems[key].take().expect("each `key` should be distinct")
                }))
            }
        }
    }

    fn id(&self) -> VoiceModelId {
        self.borrow_header().manifest.id
    }

    fn metas(&self) -> &VoiceModelMeta {
        &self.borrow_header().metas
    }

    pub(crate) fn header(&self) -> &Arc<VoiceModelHeader> {
        self.borrow_header()
    }

    pub(crate) async fn read_inference_models(
        &self,
    ) -> LoadModelResult<InferenceDomainMap<ModelBytesWithInnerVoiceIdsByDomain>> {
        let path = &self.borrow_header().path;

        let error = |context, source| LoadModelError {
            path: path.to_owned(),
            context,
            source: Some(source),
        };

        let zip = &mut *self.borrow_zip().lock().await;
        let mut zip = async_zip::base::read::seek::ZipFileReader::with_bufreader(zip)
            .await
            .map_err(|source| error(LoadModelErrorKind::OpenZipFile, source.into()))?;

        macro_rules! read_file {
            ($entry:expr $(,)?) => {{
                let (index, ModelFile { r#type, filename }): (usize, ModelFile) = $entry;
                let bytes = zip
                    .read_file(index)
                    .map_err(move |source| {
                        error(
                            LoadModelErrorKind::ReadZipEntry {
                                filename: (*filename).to_owned(),
                            },
                            source,
                        )
                    })
                    .await?;
                ModelBytes::new(r#type, bytes)
            }};
        }

        let InferenceDomainMap {
            talk,
            experimental_talk,
            singing_teacher,
            frame_decode,
        } = self.with_inference_model_entries(|inference_model_entries| {
            inference_model_entries.each_ref().map(InferenceDomainMap {
                talk: |talk| {
                    talk.as_ref()
                        .map(|InferenceModelEntry { indices, manifest }| {
                            (
                                indices.map(|op, i| (i, manifest[op].clone())),
                                manifest.style_id_to_inner_voice_id.clone(),
                            )
                        })
                },
                experimental_talk: |talk| {
                    talk.as_ref()
                        .map(|InferenceModelEntry { indices, manifest }| {
                            (
                                indices.map(|op, i| (i, manifest[op].clone())),
                                manifest.style_id_to_inner_voice_id.clone(),
                            )
                        })
                },
                singing_teacher: |singing_teacher| {
                    singing_teacher
                        .as_ref()
                        .map(|InferenceModelEntry { indices, manifest }| {
                            (
                                indices.map(|op, i| (i, manifest[op].clone())),
                                manifest.style_id_to_inner_voice_id.clone(),
                            )
                        })
                },
                frame_decode: |frame_decode| {
                    frame_decode
                        .as_ref()
                        .map(|InferenceModelEntry { indices, manifest }| {
                            (
                                indices.map(|op, i| (i, manifest[op].clone())),
                                manifest.style_id_to_inner_voice_id.clone(),
                            )
                        })
                },
            })
        });

        // TODO: Rust 1.85にしたらasync closureに戻す
        let talk = OptionFuture::from(talk.map(|(entries, style_id_to_inner_voice_id)| async {
            let [predict_duration, predict_intonation, decode] = entries.into_array();

            let predict_duration = read_file!(predict_duration);
            let predict_intonation = read_file!(predict_intonation);
            let decode = read_file!(decode);

            let model_bytes = EnumMap::from_array([predict_duration, predict_intonation, decode]);

            Ok((style_id_to_inner_voice_id, model_bytes))
        }))
        .await
        .transpose()?;

        // TODO: Rust 1.85にしたらasync closureに戻す
        let experimental_talk = OptionFuture::from(experimental_talk.map(
            |(entries, style_id_to_inner_voice_id)| async {
                let [predict_duration, predict_intonation, predict_spectrogram, run_vocoder] =
                    entries.into_array();

                let predict_duration = read_file!(predict_duration);
                let predict_intonation = read_file!(predict_intonation);
                let predict_spectrogram = read_file!(predict_spectrogram);
                let run_vocoder = read_file!(run_vocoder);

                let model_bytes = EnumMap::from_array([
                    predict_duration,
                    predict_intonation,
                    predict_spectrogram,
                    run_vocoder,
                ]);

                Ok((style_id_to_inner_voice_id, model_bytes))
            },
        ))
        .await
        .transpose()?;

        // TODO: Rust 1.85にしたらasync closureに戻す
        let singing_teacher = OptionFuture::from(singing_teacher.map(
            |(entries, style_id_to_inner_voice_id)| async {
                let [predict_sing_consonant_length, predict_sing_f0, predict_sing_volume] =
                    entries.into_array();

                let predict_sing_consonant_length = read_file!(predict_sing_consonant_length);
                let predict_sing_f0 = read_file!(predict_sing_f0);
                let predict_sing_volume = read_file!(predict_sing_volume);

                let model_bytes = EnumMap::from_array([
                    predict_sing_consonant_length,
                    predict_sing_f0,
                    predict_sing_volume,
                ]);

                Ok((style_id_to_inner_voice_id, model_bytes))
            },
        ))
        .await
        .transpose()?;

        // TODO: Rust 1.85にしたらasync closureに戻す
        let frame_decode = OptionFuture::from(frame_decode.map(
            |(entries, style_id_to_inner_voice_id)| async {
                let [sf_decode] = entries.into_array();

                let sf_decode = read_file!(sf_decode);

                let model_bytes = EnumMap::from_array([sf_decode]);

                Ok((style_id_to_inner_voice_id, model_bytes))
            },
        ))
        .await
        .transpose()?;

        Ok(InferenceDomainMap {
            talk,
            experimental_talk,
            singing_teacher,
            frame_decode,
        })
    }
}

impl<A: Async> Inner<A> {
    fn fill_debug_struct_body(&self, mut fmt: fmt::DebugStruct<'_, '_>) -> fmt::Result
    where
        A::Mutex<A::RoFile>: Debug,
    {
        fmt.field("header", self.header());
        self.with_inference_model_entries(|inference_model_entries| {
            fmt.field("inference_model_entries", inference_model_entries)
        });
        self.with_zip(|zip| fmt.field("zip", zip));
        fmt.finish()
    }
}

type InferenceModelEntries<'manifest> =
    inference_domain_map_values!(for<D> Option<InferenceModelEntry<D, &'manifest D::Manifest>>);

#[derive(derive_more::Debug)]
#[debug(bound(D::Operation: Debug))]
struct InferenceModelEntry<D: InferenceDomain, M> {
    indices: EnumMap<D::Operation, usize>,
    manifest: M,
}

#[ext]
impl<A: Async> A {
    async fn open_zip(
        path: &Path,
    ) -> anyhow::Result<
        async_zip::base::read::seek::ZipFileReader<futures_util::io::BufReader<A::RoFile>>,
    > {
        let zip = Self::open_file_ro(path).await.with_context(|| {
            // fs-errのと同じにする
            format!("failed to open file `{}`", path.display())
        })?;
        let zip = async_zip::base::read::seek::ZipFileReader::with_bufreader(zip).await?;
        Ok(zip)
    }
}

// `BufReader`はasync_zip v0.0.16では不要、v0.0.17では必要
#[ext]
impl<R: AsyncRead + AsyncSeek + Unpin>
    async_zip::base::read::seek::ZipFileReader<futures_util::io::BufReader<R>>
{
    async fn with_bufreader(rdr: R) -> async_zip::error::Result<Self>
    where
        Self: Sized, // trivial
    {
        Self::new(futures_util::io::BufReader::new(rdr)).await
    }
}

#[ext]
impl<R: AsyncBufRead + AsyncSeek + Unpin> async_zip::base::read::seek::ZipFileReader<R> {
    fn entry_indices_by_utf8_filenames(&self) -> HashMap<String, usize> {
        self.file()
            .entries()
            .iter()
            .enumerate()
            .flat_map(|(i, e)| e.filename().as_str().map(|s| (s.to_owned(), i)))
            .collect()
    }

    async fn read_file(&mut self, index: usize) -> anyhow::Result<Vec<u8>> {
        let mut rdr = self.reader_with_entry(index).await?;
        let mut buf = Vec::with_capacity(rdr.entry().uncompressed_size() as usize);
        rdr.read_to_end_checked(&mut buf).await?;
        Ok(buf)
    }
}

// FIXME: "header"といいつつ、VVMのファイルパスを持っている状態になっている。
/// 音声モデルが持つ、各モデルファイルの実体を除く情報。
///
/// モデルの`[u8]`と分けて`Status`に渡す。
#[derive(Debug)]
pub(crate) struct VoiceModelHeader {
    pub(super) manifest: Manifest,
    /// メタ情報。
    ///
    /// `manifest`が対応していない`StyleType`のスタイルは含まれるべきではない。
    pub(super) metas: VoiceModelMeta,
    pub(super) path: PathBuf,
}

impl VoiceModelHeader {
    fn new(manifest: Manifest, metas: &[u8], path: &Path) -> LoadModelResult<Self> {
        let error = |context, source| LoadModelError {
            path: path.to_owned(),
            context,
            source: Some(source),
        };

        let metas = serde_json::from_slice::<VoiceModelMeta>(metas).map_err(|source| {
            error(
                LoadModelErrorKind::InvalidModelFormat,
                anyhow::Error::from(source)
                    .context(format!("{}が不正です", manifest.metas_filename())),
            )
        })?;

        manifest
            .domains()
            .check_acceptable(&metas)
            .map_err(|style_type| {
                error(
                    LoadModelErrorKind::InvalidModelFormat,
                    anyhow!(
                        "{metas_filename}には`{style_type}`のスタイルが存在しますが、manifest.json\
                         での対応がありません",
                        metas_filename = manifest.metas_filename(),
                    ),
                )
            })?;

        Ok(Self {
            manifest,
            metas,
            path: path.to_owned(),
        })
    }
}

pub(crate) enum ModelBytes {
    Onnx(Vec<u8>),
    VvBin(Vec<u8>),
}

impl ModelBytes {
    fn new(kind: ModelFileType, bytes: Vec<u8>) -> Self {
        (match kind {
            ModelFileType::Onnx => Self::Onnx,
            ModelFileType::VvBin => Self::VvBin,
        })(bytes)
    }
}

impl InferenceDomainMap<ManifestDomains> {
    /// manifestとして対応していない`StyleType`に対してエラーを発する。
    ///
    /// `Status`はこのバリデーションを信頼し、`InferenceDomain`の不足時にパニックする。
    fn check_acceptable(&self, metas: &[CharacterMeta]) -> std::result::Result<(), StyleType> {
        let err = metas
            .iter()
            .flat_map(|CharacterMeta { styles, .. }| styles)
            .map(|StyleMeta { r#type, .. }| *r#type)
            .unique()
            .find(|&style_type| !self.accepts(style_type));

        match err {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    /// メタ情報にタイプが`style_type`のスタイルが含まれることを許容するかどうか。
    ///
    /// 例えば`self.singing_teacher`が`None`のとき、`StyleType::Sing`に対して`false`を返す。
    fn accepts(&self, style_type: StyleType) -> bool {
        let Self {
            talk,
            experimental_talk,
            singing_teacher,
            frame_decode,
        } = self;

        // `StyleType::Talk`は`TalkDomain`と`ExperimentalTalkDomain`の二つに対応する
        assert_eq!(
            TalkDomain::style_types(),
            ExperimentalTalkDomain::style_types(),
        );

        return TalkDomain::contains(style_type)
            .implies(|| talk.is_some() || experimental_talk.is_some())
            && SingingTeacherDomain::contains(style_type).implies(|| singing_teacher.is_some())
            && FrameDecodeDomain::contains(style_type).implies(|| frame_decode.is_some());

        #[ext]
        impl<D: InferenceDomain> D {
            fn contains(style_type: StyleType) -> bool {
                Self::style_types().contains(&style_type)
            }
        }

        #[ext]
        impl bool {
            fn implies(self, other: impl FnOnce() -> Self) -> Self {
                !self || other()
            }
        }
    }
}

pub(crate) mod blocking {
    use std::{
        fmt::{self, Debug},
        path::Path,
    };

    use crate::{asyncs::SingleTasked, future::FutureExt as _, VoiceModelMeta};

    use super::{Inner, VoiceModelId};

    /// 音声モデルファイル。
    ///
    /// VVMファイルと対応する。
    #[cfg_attr(doc, doc(alias = "VoicevoxVoiceModelFile"))]
    pub struct VoiceModelFile(Inner<SingleTasked>);

    impl self::VoiceModelFile {
        /// VVMファイルを開く。
        #[cfg_attr(doc, doc(alias = "voicevox_voice_model_file_open"))]
        pub fn open(path: impl AsRef<Path>) -> crate::Result<Self> {
            Inner::open(path).block_on().map(Self)
        }

        /// VVMファイルを閉じる。
        pub fn close(self) -> (VoiceModelId, VoiceModelMeta) {
            let heads = self.0.into_heads();
            (*heads.header.manifest.id(), heads.header.metas.clone())
        }

        pub(crate) fn inner(&self) -> &Inner<SingleTasked> {
            &self.0
        }

        /// ID。
        ///
        /// 詳細は[`VoiceModelId`]を参照。
        #[cfg_attr(doc, doc(alias = "voicevox_voice_model_file_id"))]
        pub fn id(&self) -> VoiceModelId {
            self.0.id()
        }

        /// メタ情報。
        #[cfg_attr(doc, doc(alias = "voicevox_voice_model_file_create_metas_json"))]
        pub fn metas(&self) -> &VoiceModelMeta {
            self.0.metas()
        }
    }

    impl Debug for VoiceModelFile {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let fmt = fmt.debug_struct("VoiceModelFile");
            self.0.fill_debug_struct_body(fmt)
        }
    }
}

pub(crate) mod nonblocking {
    use std::{
        fmt::{self, Debug},
        path::Path,
    };

    use crate::{asyncs::BlockingThreadPool, Result, VoiceModelMeta};

    use super::{Inner, VoiceModelId};

    /// 音声モデルファイル。
    ///
    /// VVMファイルと対応する。
    ///
    /// # Performance
    ///
    /// [blocking]クレートにより動いている。詳しくは[`nonblocking`モジュールのドキュメント]を参照。
    ///
    /// [blocking]: https://docs.rs/crate/blocking
    /// [`nonblocking`モジュールのドキュメント]: crate::nonblocking
    pub struct VoiceModelFile(Inner<BlockingThreadPool>);

    impl self::VoiceModelFile {
        /// VVMファイルを開く。
        pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
            Inner::open(path).await.map(Self)
        }

        /// VVMファイルを閉じる。
        pub async fn close(self) -> (VoiceModelId, VoiceModelMeta) {
            let heads = self.0.into_heads();
            heads.zip.into_inner().close().await;
            (*heads.header.manifest.id(), heads.header.metas.clone())
        }

        pub(crate) fn inner(&self) -> &Inner<BlockingThreadPool> {
            &self.0
        }

        /// ID。
        ///
        /// 詳細は[`VoiceModelId`]を参照。
        pub fn id(&self) -> VoiceModelId {
            self.0.id()
        }

        /// メタ情報。
        pub fn metas(&self) -> &VoiceModelMeta {
            self.0.metas()
        }
    }

    impl Debug for VoiceModelFile {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let fmt = fmt.debug_struct("VoiceModelFile");
            self.0.fill_debug_struct_body(fmt)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use crate::{CharacterMeta, StyleType};

    use super::super::{
        infer::domains::InferenceDomainMap,
        manifest::{
            ExperimentalTalkManifest, FrameDecodeManifest, ManifestDomains, SingingTeacherManifest,
            TalkManifest,
        },
    };

    #[rstest]
    #[case(
        &InferenceDomainMap {
            talk: None,
            experimental_talk: None,
            singing_teacher: None,
            frame_decode: None,
        },
        &[],
        Ok(())
    )]
    #[case(
        &InferenceDomainMap {
            talk: Some(TalkManifest::default()),
            experimental_talk: Some(ExperimentalTalkManifest::default()),
            singing_teacher: Some(SingingTeacherManifest::default()),
            frame_decode: Some(FrameDecodeManifest::default()),
        },
        &[character(&[StyleType::Talk])],
        Ok(())
    )]
    #[case(
        &InferenceDomainMap {
            talk: Some(TalkManifest::default()),
            experimental_talk: Some(ExperimentalTalkManifest::default()),
            singing_teacher: Some(SingingTeacherManifest::default()),
            frame_decode: Some(FrameDecodeManifest::default()),
        },
        &[character(&[StyleType::Talk, StyleType::Sing])],
        Ok(())
    )]
    #[case(
        &InferenceDomainMap {
            talk: None,
            experimental_talk: None,
            singing_teacher: None,
            frame_decode: None,
        },
        &[character(&[StyleType::Talk])],
        Err(())
    )]
    fn check_acceptable_works(
        #[case] manifest: &InferenceDomainMap<ManifestDomains>,
        #[case] metas: &[CharacterMeta],
        #[case] expected: std::result::Result<(), ()>,
    ) {
        let actual = manifest.check_acceptable(metas).map_err(|_| ());
        assert_eq!(expected, actual);
    }

    fn character(style_types: &'static [StyleType]) -> CharacterMeta {
        let styles = style_types
            .iter()
            .map(|style_type| {
                json!({
                    "id": 0,
                    "name": "style1",
                    "type": style_type,
                    "order": null
                })
            })
            .collect::<Vec<_>>();

        serde_json::from_value(json!({
            "name": "dummy",
            "styles": styles,
            "version": "0.0.1",
            "speaker_uuid": "574bc678-8370-44be-b941-08e46e7b47d7",
            "order": null
        }))
        .unwrap()
    }
}
