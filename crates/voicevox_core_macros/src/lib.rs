#![warn(rust_2018_idioms)]
//! `voicevox_core`用の内部クレート。
//!
//! SemVerに従わない。

mod check;
mod extract;
mod inference_domain;
mod inference_domains;
mod mora_mappings;
mod python_api;

use syn::parse_macro_input;

/// Rust APIクレート内で、`crate::core::infer::InferenceDomain`の導出などを行う。
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

/// Rust APIクレート内で、`crate::core::infer::InferenceInputSignature`を導出する。
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

/// Rust APIクレート内で`crate::core::infer::InferenceInputSignature`を、`TryFrom<OutputTensor>`ごと導出
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

/// # Example
///
/// ```
/// type ManifestDomains =
///     (substitute_type!(Option<D::Manifest> where D = TalkDomain as InferenceDomain),);
/// ```
///
/// ↓
///
/// ```
/// type ManifestDomains = (Option<<TalkManifest as InferenceDomain>::Manifest>,);
/// //                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// //                             T ← <TalkManifest as InferenceDomain>
/// ```
#[cfg(not(doctest))]
#[proc_macro]
pub fn substitute_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    from_syn(inference_domains::substitute_type(input))
}

#[proc_macro_derive(MoraMappings, attributes(mora_mappings))]
pub fn derive_mora_mappings(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = &parse_macro_input!(input);
    from_syn(mora_mappings::derive_mora_mappings(input))
}

#[proc_macro]
pub fn pyproject_project_version(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    from_syn(python_api::pyproject_project_version(input))
}

fn from_syn(result: syn::Result<proc_macro2::TokenStream>) -> proc_macro::TokenStream {
    result.unwrap_or_else(|e| e.to_compile_error()).into()
}
