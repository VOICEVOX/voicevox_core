mod assert_cdylib;
mod float_assert;
mod log_mask;
mod snapshots;
mod symbols;
mod testcases;

// voicevox_core_c_apiのcdylibを対象にテストを行う。
//
// C APIの定義を変更する場合:
// 1. symbols.rsの実装を変更する。
// 2. 1.によってコンパイルが通らなくなったら、適宜修正する。
//
// テストを追加する場合:
// 1. testcases/{テスト名}.rsを追加し、testcases.rsでマウントする。
// 2. testcases/{テスト名}.rsに`struct TestCase`を追加する。
// 3. `struct TestCase`に`trait assert_cdylib::TestCase`を、`#[typetag::serde(name = "…")]`の形でimplする。
// 4. `struct TestCase`の具体値を`case!`で登録する。

fn main() -> anyhow::Result<()> {
    return assert_cdylib::exec::<TestContext>();

    enum TestContext {}

    impl assert_cdylib::TestContext for TestContext {
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
    }
}
