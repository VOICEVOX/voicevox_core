#![warn(rust_2018_idioms)]

mod extract;
mod inference_domain;
mod manifest;

use syn::parse_macro_input;

/// Rust APIクレート内で、`crate::infer::InferenceDomain`の導出などを行う。
///
/// 次のことを行う。
///
/// - `InferenceDomain`の導出
/// - 各バリアントに対する`InferenceInputSignature`の実装を、型ごと生成
///
/// # Example
///
/// ```
/// use enum_map::Enum;
/// use macros::InferenceOperation;
///
/// pub(crate) enum TalkDomain {}
///
/// impl InferenceDomain for TalkDomain {
///     type Operation = TalkOperation;
///     // ...
/// }
///
/// #[derive(Clone, Copy, Enum, InferenceOperation)]
/// #[inference_operation(
///     type Domain = TalkDomain;
/// )]
/// pub(crate) enum TalkOperation {
///     #[inference_operation(
///         type Input = PredictDurationInput;
///         type Output = PredictDurationOutput;
///     )]
///     PredictDuration,
///
///     #[inference_operation(
///         type Input = PredictIntonationInput;
///         type Output = PredictIntonationOutput;
///     )]
///     PredictIntonation,
///
///     #[inference_operation(
///         type Input = DecodeInput;
///         type Output = DecodeOutput;
///     )]
///     Decode,
/// }
/// ```
#[cfg(not(doctest))]
#[proc_macro_derive(InferenceOperation, attributes(inference_operation))]
pub fn derive_inference_operation(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = &parse_macro_input!(input);
    from_syn(inference_domain::derive_inference_operation(input))
}

/// Rust APIクレート内で、`crate::infer::InferenceInputSignature`を導出する。
///
/// # Example
///
/// ```
/// use macros::InferenceInputSignature;
///
/// #[derive(InferenceInputSignature)]
/// #[inference_input_signature(
///     type Signature = PredictDuration;
/// )]
/// pub(crate) struct PredictDurationInput {
///     pub(crate) phoneme_list: Array1<i64>,
///     pub(crate) speaker_id: Array1<i64>,
/// }
/// ```
#[cfg(not(doctest))]
#[proc_macro_derive(InferenceInputSignature, attributes(inference_input_signature))]
pub fn derive_inference_input_signature(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = &parse_macro_input!(input);
    from_syn(inference_domain::derive_inference_input_signature(input))
}

/// Rust APIクレート内で`crate::infer::InferenceInputSignature`を、`TryFrom<OutputTensor>`ごと導出
/// する。
///
/// # Example
///
/// ```
/// use macros::InferenceOutputSignature;
///
/// #[derive(InferenceOutputSignature)]
/// pub(crate) struct PredictDurationOutput {
///     pub(crate) phoneme_length: Array1<f32>,
/// }
/// ```
#[cfg(not(doctest))]
#[proc_macro_derive(InferenceOutputSignature)]
pub fn derive_inference_output_signature(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = &parse_macro_input!(input);
    from_syn(inference_domain::derive_inference_output_signature(input))
}

/// 構造体のフィールドを取得できる`std::ops::Index`の実装を導出する。
///
/// # Example
///
/// ```
/// use macros::IndexForFields;
///
/// #[derive(IndexForFields)]
/// #[index_for_fields(TalkOperation)]
/// pub(crate) struct TalkManifest {
///     #[index_for_fields(TalkOperation::PredictDuration)]
///     pub(crate) predict_duration_filename: Arc<str>,
///
///     #[index_for_fields(TalkOperation::PredictIntonation)]
///     pub(crate) predict_intonation_filename: Arc<str>,
///
///     #[index_for_fields(TalkOperation::Decode)]
///     pub(crate) decode_filename: Arc<str>,
///
///     // …
/// }
/// ```
#[cfg(not(doctest))]
#[proc_macro_derive(IndexForFields, attributes(index_for_fields))]
pub fn derive_index_for_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = &parse_macro_input!(input);
    from_syn(manifest::derive_index_for_fields(input))
}

fn from_syn(result: syn::Result<proc_macro2::TokenStream>) -> proc_macro::TokenStream {
    result.unwrap_or_else(|e| e.to_compile_error()).into()
}
