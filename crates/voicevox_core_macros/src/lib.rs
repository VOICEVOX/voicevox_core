#![warn(rust_2018_idioms)]

mod inference_domain;

use syn::parse_macro_input;

#[proc_macro_derive(InferenceDomain, attributes(inference_domain))]
pub fn derive_inference_domain(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = &parse_macro_input!(input);
    from_syn(inference_domain::derive_inference_domain(input))
}

#[proc_macro_derive(InferenceInputSignature, attributes(inference_input_signature))]
pub fn derive_inference_input_signature(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = &parse_macro_input!(input);
    from_syn(inference_domain::derive_inference_input_signature(input))
}

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
