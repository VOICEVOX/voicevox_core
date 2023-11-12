#![warn(rust_2018_idioms)]

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned as _,
    Data, DataEnum, DataStruct, DataUnion, DeriveInput, Field, Fields, Token, Type,
};

#[proc_macro_derive(InferenceGroup)]
pub fn derive_inference_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics crate::infer::InferenceGroup for #ident #ty_generics #where_clause {}
    }
    .into()
}

#[proc_macro_derive(InferenceInputSignature, attributes(input_signature))]
pub fn derive_inference_input_signature(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    return derive_inference_input_signature(&parse_macro_input!(input))
        .unwrap_or_else(|e| e.to_compile_error())
        .into();

    fn derive_inference_input_signature(
        input: &DeriveInput,
    ) -> syn::Result<proc_macro2::TokenStream> {
        let DeriveInput {
            attrs,
            ident,
            generics,
            data,
            ..
        } = input;

        let AssocTypeSignature(signature) = attrs
            .iter()
            .find(|a| a.path().is_ident("input_signature"))
            .ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "missing `#[input_signature(…)]`",
                )
            })?
            .parse_args()?;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let fields = struct_fields(data)?;

        let param_infos = fields
            .iter()
            .map(|(name, ty)| {
                let name = name.to_string();
                quote! {
                    crate::infer::ParamInfo {
                        name: ::std::borrow::Cow::Borrowed(#name),
                        dt: <
                            <#ty as crate::infer::ArrayExt>::Scalar as crate::infer::InputScalar
                        >::KIND,
                        ndim: <
                            <#ty as crate::infer::ArrayExt>::Dimension as ::ndarray::Dimension
                        >::NDIM,
                    },
                }
            })
            .collect::<proc_macro2::TokenStream>();

        let field_names = fields.iter().map(|(name, _)| name);

        Ok(quote! {
            impl #impl_generics crate::infer::InferenceInputSignature for #ident #ty_generics
            #where_clause
            {
                type Signature = #signature;

                const PARAM_INFOS: &'static [crate::infer::ParamInfo<
                    crate::infer::InputScalarKind
                >] = &[
                    #param_infos
                ];

                fn make_run_context<R: crate::infer::InferenceRuntime>(
                    self,
                    sess: &mut R::Session,
                ) -> R::RunContext<'_> {
                    let mut ctx = <R::RunContext<'_> as ::std::convert::From<_>>::from(sess);
                    #(
                        R::push_input(self.#field_names, &mut ctx);
                    )*
                    ctx
                }
            }
        })
    }

    struct AssocTypeSignature(syn::Ident);

    impl Parse for AssocTypeSignature {
        fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
            let key = input.parse::<syn::Ident>()?;
            if key != "Signature" {
                return Err(syn::Error::new(key.span(), "expected `Signature`"));
            }
            input.parse::<Token![=]>()?;
            let value = input.parse::<syn::Ident>()?;
            Ok(Self(value))
        }
    }
}

#[proc_macro_derive(InferenceOutputSignature)]
pub fn derive_inference_output_signature(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    return derive_inference_output_signature(&parse_macro_input!(input))
        .unwrap_or_else(|e| e.to_compile_error())
        .into();

    fn derive_inference_output_signature(
        input: &DeriveInput,
    ) -> syn::Result<proc_macro2::TokenStream> {
        let DeriveInput {
            ident,
            generics,
            data,
            ..
        } = input;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let fields = struct_fields(data)?;
        let num_fields = fields.len();

        let param_infos = fields
            .iter()
            .map(|(name, ty)| {
                let name = name.to_string();
                quote! {
                    crate::infer::ParamInfo {
                        name: ::std::borrow::Cow::Borrowed(#name),
                        dt: <
                            <#ty as crate::infer::ArrayExt>::Scalar as crate::infer::OutputScalar
                        >::KIND,
                        ndim: <
                            <#ty as crate::infer::ArrayExt>::Dimension as ::ndarray::Dimension
                        >::NDIM,
                    },
                }
            })
            .collect::<proc_macro2::TokenStream>();

        let field_names = fields.iter().map(|(name, _)| name);

        Ok(quote! {
            impl #impl_generics crate::infer::InferenceOutputSignature for #ident #ty_generics
            #where_clause
            {
                const PARAM_INFOS: &'static [crate::infer::ParamInfo<
                    crate::infer::OutputScalarKind
                >] = &[
                    #param_infos
                ];
            }

            impl #impl_generics
                ::std::convert::TryFrom<::std::vec::Vec<crate::infer::OutputTensor>>
                for #ident #ty_generics
            #where_clause
            {
                type Error = ::anyhow::Error;

                fn try_from(
                    tensors: ::std::vec::Vec<OutputTensor>,
                ) -> ::std::result::Result<Self, Self::Error> {
                    ::anyhow::ensure!(
                        tensors.len() == #num_fields,
                        "expected {} tensor(s), got {}",
                        #num_fields,
                        tensors.len(),
                    );

                    let tensors = &mut ::std::iter::IntoIterator::into_iter(tensors);
                    ::std::result::Result::Ok(Self {
                        #(
                            #field_names: ::std::convert::TryInto::try_into(
                                ::std::iter::Iterator::next(tensors)
                                    .expect("the length should have been checked"),
                            )?,
                        )*
                    })
                }
            }
        })
    }
}

fn struct_fields(data: &Data) -> syn::Result<Vec<(&syn::Ident, &Type)>> {
    let fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        Data::Struct(DataStruct { fields, .. }) => {
            return Err(syn::Error::new(fields.span(), "expect named fields"));
        }
        Data::Enum(DataEnum { enum_token, .. }) => {
            return Err(syn::Error::new(enum_token.span(), "expected an enum"));
        }
        Data::Union(DataUnion { union_token, .. }) => {
            return Err(syn::Error::new(union_token.span(), "expected an enum"));
        }
    };

    Ok(fields
        .named
        .iter()
        .map(|Field { ident, ty, .. }| (ident.as_ref().expect("should be named"), ty))
        .collect())
}
