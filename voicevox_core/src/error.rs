use thiserror::Error;

/*
 * 新しいエラーを定義したら、必ずc_export.rsにあるVoicevoxResultCodeに対応するコードを定義し、
 * internal.rsにある変換関数に変換処理を加えること
 */

#[derive(Error, Debug)]
pub enum Error {
    #[error("openjtalkの辞書が読み込まれていません")]
    NotLoadedOpenjtalkDict,
}
