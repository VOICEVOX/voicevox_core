use test_util::c_api::VV_MODELS_ROOT_DIR;

mod assert_cdylib;
mod float_assert;
mod log_mask;
mod snapshots;
mod testcases;

// voicevox_core_c_apiのcdylibを対象にテストを行う。
//
// C APIの定義を変更した場合は、テスト実行前に`cargo xtask update-c-header`を実行すること。
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
        const FEATURES: &'static [&'static str] = &["onnxruntime-libloading"];
        const TARGET_DIR: &'static str = "../../target";
        const CDYLIB_NAME: &'static str = "voicevox_core";
        const RUNTIME_ENVS: &'static [(&'static str, &'static str)] =
            &[("VV_MODELS_ROOT_DIR", VV_MODELS_ROOT_DIR)];
    }
}
