#![warn(rust_2018_idioms)]

mod inference_domain;

use syn::parse_macro_input;

/// `voicevox_core`内で、`crate::infer::InferenceDomain`を実装する。
///
/// # Example
///
/// ```
/// use enum_map::Enum;
/// use macros::InferenceDomain;
///
/// #[derive(Clone, Copy, Enum, InferenceDomain)]
/// pub(crate) enum InferenceKind {
///     #[inference_domain(
///         type Input = PredictDurationInput;
///         type Output = PredictDurationOutput;
///     )]
///     PredictDuration,
///
///     #[inference_domain(
///         type Input = PredictIntonationInput;
///         type Output = PredictIntonationOutput;
///     )]
///     PredictIntonation,
///
///     #[inference_domain(
///         type Input = DecodeInput;
///         type Output = DecodeOutput;
///     )]
///     Decode,
/// }
/// ```
#[cfg(not(doctest))]
#[proc_macro_derive(InferenceDomain, attributes(inference_domain))]
pub fn derive_inference_domain(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = &parse_macro_input!(input);
    from_syn(inference_domain::derive_inference_domain(input))
}

/// `voicevox_core`内で、`crate::infer::InferenceInputSignature`を実装する。
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

/// `voicevox_core`内で、`crate::infer::InferenceInputSignature`を実装する。
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

fn from_syn(result: syn::Result<proc_macro2::TokenStream>) -> proc_macro::TokenStream {
    result.unwrap_or_else(|e| e.to_compile_error()).into()
}
