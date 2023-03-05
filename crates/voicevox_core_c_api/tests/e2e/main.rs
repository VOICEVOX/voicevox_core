use std::ffi::{c_char, c_int, CStr};

use assert_cmd::assert::AssertResult;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};

use crate::snapshots::SNAPSHOTS;

use self::assert_cdylib::{TestCase, Utf8Output};

mod assert_cdylib;
mod float_assert;
mod mask;
mod snapshots;

// voicevox_core_c_apiのcdylibを対象にテストを行う。
//
// C APIの定義を変更する場合:
// 1. `Symbols`のメンバー及びコンストラクタの実装を変更する。
// 2. 1.によってコンパイルが通らなくなったら、適宜修正する。
//
// テストを追加する場合:
// 1. テストケースを表わすstructを`main`の中に追加する。
// 2. structに`#[typetag::serde]`を使って`TestCase`をimplする。
// 3. structの値を`TestSuite::TESTCASES`に追加する。

fn main() -> anyhow::Result<()> {
    return assert_cdylib::exec::<TestSuite>();

    enum TestSuite {}

    impl assert_cdylib::TestSuite for TestSuite {
        const TARGET_DIR: &'static str = "../../target";
        const CDYLIB_NAME: &'static str = "voicevox_core";
        const BUILD_ENVS: &'static [(&'static str, &'static str)] = &[
            // 他の単体テストが動いているときにonnxruntime-sysの初回ビルドを行うと、Windows環境だと
            // `$ORT_OUT_DIR`のハックが問題を起こす。そのためこのハック自体を無効化する
            //
            // featuresの差分を出さないように`cargo build`することができればonnxruntime-sysの
            // ビルド自体がされないのだが、このバイナリから`cargo build`の状況を知るのは無理に近い
            ("ORT_OUT_DIR", ""),
            // DirectMLとCUDAは無効化
            ("ORT_USE_CUDA", "0"),
        ];
        const RUNTIME_ENVS: &'static [(&'static str, &'static str)] =
            &[("VV_MODELS_ROOT_DIR", "../../model")];
        const TESTCASES: &'static [&'static dyn TestCase] = &[&CompatibleEngine];
    }

    struct Symbols<'lib> {
        initialize: Symbol<'lib, unsafe extern "C" fn(bool, c_int, bool) -> bool>,
        load_model: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
        is_model_loaded: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
        finalize: Symbol<'lib, unsafe extern "C" fn()>,
        metas: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
        supported_devices: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
        yukarin_s_forward:
            Symbol<'lib, unsafe extern "C" fn(i64, *mut i64, *mut i64, *mut f32) -> bool>,
        yukarin_sa_forward: Symbol<
            'lib,
            unsafe extern "C" fn(
                i64,
                *mut i64,
                *mut i64,
                *mut i64,
                *mut i64,
                *mut i64,
                *mut i64,
                *mut i64,
                *mut f32,
            ) -> bool,
        >,
        decode_forward: Symbol<
            'lib,
            unsafe extern "C" fn(i64, i64, *mut f32, *mut f32, *mut i64, *mut f32) -> bool,
        >,
    }

    impl<'lib> Symbols<'lib> {
        unsafe fn new(lib: &'lib Library) -> Result<Self, libloading::Error> {
            macro_rules! new(($($name:ident),* $(,)?) => {
                Self {
                    $(
                        $name: lib.get(stringify!($name).as_ref())?,
                    )*
                }
            });

            Ok(new!(
                initialize,
                load_model,
                is_model_loaded,
                finalize,
                metas,
                supported_devices,
                yukarin_s_forward,
                yukarin_sa_forward,
                decode_forward,
            ))
        }
    }

    #[derive(Serialize, Deserialize)]
    struct CompatibleEngine;

    #[typetag::serde]
    impl TestCase for CompatibleEngine {
        unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
            let Symbols {
                initialize,
                load_model,
                is_model_loaded,
                finalize,
                metas,
                supported_devices,
                yukarin_s_forward,
                yukarin_sa_forward,
                decode_forward,
            } = Symbols::new(lib)?;

            let metas_json = {
                let metas_json = metas();
                let metas_json = CStr::from_ptr(metas_json).to_str()?;
                metas_json.parse::<serde_json::Value>()?;
                metas_json
            };

            let supported_devices = {
                let supported_devices = supported_devices();
                CStr::from_ptr(supported_devices)
                    .to_str()?
                    .parse::<serde_json::Value>()?
            };

            assert!(initialize(false, 0, false));

            assert!(!is_model_loaded(SPEAKER_ID));
            assert!(load_model(SPEAKER_ID));
            assert!(is_model_loaded(SPEAKER_ID));

            // テスト用テキストは"t e s u t o"

            let phoneme_length = {
                let mut phoneme_length = [0.; 8];
                assert!(yukarin_s_forward(
                    8,
                    [0, 37, 14, 35, 6, 37, 30, 0].as_mut_ptr(),
                    &mut { SPEAKER_ID } as *mut i64,
                    phoneme_length.as_mut_ptr(),
                ));
                phoneme_length
            };

            let intonation_list = {
                let mut intonation_list = [0.; 5];
                assert!(yukarin_sa_forward(
                    5,
                    [0, 14, 6, 30, 0].as_mut_ptr(),
                    [-1, 37, 35, 37, -1].as_mut_ptr(),
                    [0, 1, 0, 0, 0].as_mut_ptr(),
                    [0, 1, 0, 0, 0].as_mut_ptr(),
                    [0, 1, 0, 0, 0].as_mut_ptr(),
                    [0, 0, 0, 1, 0].as_mut_ptr(),
                    &mut { SPEAKER_ID } as *mut i64,
                    intonation_list.as_mut_ptr(),
                ));
                intonation_list
            };

            let wave = {
                let mut wave = [0.; 256 * F0_LENGTH];
                assert!(decode_forward(
                    F0_LENGTH as _,
                    PHONEME_SIZE as _,
                    {
                        let mut f0 = [0.; F0_LENGTH];
                        f0[9..24].fill(5.905218);
                        f0[37..60].fill(5.565851);
                        f0
                    }
                    .as_mut_ptr(),
                    {
                        let mut phoneme = [0.; PHONEME_SIZE * F0_LENGTH];
                        let mut set_one = |index, range| {
                            for i in range {
                                phoneme[i * PHONEME_SIZE + index] = 1.;
                            }
                        };
                        set_one(0, 0..9);
                        set_one(37, 9..13);
                        set_one(14, 13..24);
                        set_one(35, 24..30);
                        set_one(6, 30..37);
                        set_one(37, 37..45);
                        set_one(30, 45..60);
                        set_one(0, 60..69);
                        phoneme
                    }
                    .as_mut_ptr(),
                    &mut { SPEAKER_ID } as *mut i64,
                    wave.as_mut_ptr(),
                ));
                wave
            };

            std::assert_eq!(include_str!("../../../../model/metas.json"), metas_json);

            std::assert_eq!(
                voicevox_core::SUPPORTED_DEVICES.to_json(),
                supported_devices,
            );

            float_assert::close_l1(
                &phoneme_length,
                &SNAPSHOTS.compatible_engine.yukarin_s_forward,
                0.01,
            );

            float_assert::close_l1(
                &intonation_list,
                &SNAPSHOTS.compatible_engine.yukarin_sa_forward,
                0.01,
            );

            assert!(wave.iter().copied().all(f32::is_normal));

            finalize();
            return Ok(());

            const SPEAKER_ID: i64 = 0;
            const F0_LENGTH: usize = 69;
            const PHONEME_SIZE: usize = 45;
        }

        fn assert_output(&self, output: Utf8Output) -> AssertResult {
            output
                .mask_timestamps()
                .mask_windows_video_cards()
                .assert()
                .try_success()?
                .try_stdout("")?
                .try_stderr(&*SNAPSHOTS.compatible_engine.stderr)
        }
    }
}
