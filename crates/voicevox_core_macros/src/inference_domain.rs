use indexmap::IndexMap;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
    Attribute, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Field, Fields, Generics,
    ItemType, Type, Variant,
};

pub(crate) fn derive_inference_operation(
    input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {
        attrs,
        vis,
        ident: operation_ty_name,
        generics,
        data,
        ..
    } = input;

    deny_generics(generics)?;

    let AssocTypeDomain(domain_ty) = attrs
        .iter()
        .find(|a| a.path().is_ident("inference_operation"))
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing `#[inference_operation(…)]`",
            )
        })?
        .parse_args()?;

    let variants = unit_enum_variants(data)?
        .into_iter()
        .map(|(attrs, variant_name)| {
            let AssocTypes { input, output } = attrs
                .iter()
                .find(|a| a.path().is_ident("inference_operation"))
                .ok_or_else(|| {
                    syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "missing `#[inference_operation(…)]`",
                    )
                })?
                .parse_args()?;

            Ok((variant_name, (input, output)))
        })
        .collect::<syn::Result<IndexMap<_, _>>>()?;

    let variant_names = &variants.keys().collect::<Vec<_>>();

    let signatures = variants
        .iter()
        .map(|(variant_name, (input_ty, output_ty))| {
            quote! {
                #vis enum #variant_name {}

                impl crate::infer::InferenceSignature for #variant_name {
                    type Domain = #domain_ty;
                    type Input = #input_ty;
                    type Output = #output_ty;

                    const OPERATION: <Self::Domain as crate::infer::InferenceDomain>::Operation =
                        #operation_ty_name :: #variant_name;
                }
            }
        });

    return Ok(quote! {
        impl crate::infer::InferenceOperation for #operation_ty_name {
            const PARAM_INFOS: ::enum_map::EnumMap<
                Self,
                (
                    &'static [crate::infer::ParamInfo<crate::infer::InputScalarKind>],
                    &'static [crate::infer::ParamInfo<crate::infer::OutputScalarKind>],
                ),
            > = ::enum_map::EnumMap::from_array([
                #((
                    <#variant_names as crate::infer::InferenceSignature>::Input::PARAM_INFOS,
                    <#variant_names as crate::infer::InferenceSignature>::Output::PARAM_INFOS
                )),*
            ]);
        }

        #(#signatures)*
    });

    struct AssocTypeDomain(Type);

    impl Parse for AssocTypeDomain {
        fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
            let ItemType { ident, ty, .. } = input.parse()?;

            if ident != "Domain" {
                return Err(syn::Error::new(ident.span(), "expected `Domain`"));
            }
            Ok(Self(*ty))
        }
    }

    struct AssocTypes {
        input: Type,
        output: Type,
    }

    impl Parse for AssocTypes {
        fn parse(stream: ParseStream<'_>) -> syn::Result<Self> {
            let mut input = None;
            let mut output = None;

            while !stream.is_empty() {
                let ItemType {
                    ident,
                    generics,
                    ty,
                    ..
                } = stream.parse()?;

                deny_generics(&generics)?;

                *match &*ident.to_string() {
                    "Input" => &mut input,
                    "Output" => &mut output,
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            "expected `Input` or `Output`",
                        ))
                    }
                } = Some(*ty);
            }

            let input =
                input.ok_or_else(|| syn::Error::new(stream.span(), "missing `type Input = …;`"))?;

            let output = output
                .ok_or_else(|| syn::Error::new(stream.span(), "missing `type Output = …;`"))?;

            Ok(Self { input, output })
        }
    }

    fn deny_generics(generics: &Generics) -> syn::Result<()> {
        if !generics.params.is_empty() {
            return Err(syn::Error::new(generics.params.span(), "must be empty"));
        }
        if let Some(where_clause) = &generics.where_clause {
            return Err(syn::Error::new(where_clause.span(), "must be empty"));
        }
        Ok(())
    }
}

pub(crate) fn derive_inference_input_signature(
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
        .find(|a| a.path().is_ident("inference_input_signature"))
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing `#[inference_input_signature(…)]`",
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
                    dt: <<#ty as __ArrayExt>::Scalar as crate::infer::InputScalar>::KIND,
                    ndim: <<#ty as __ArrayExt>::Dimension as ::ndarray::Dimension>::NDIM,
                },
            }
        })
        .collect::<proc_macro2::TokenStream>();

    let field_names = fields.iter().map(|(name, _)| name);

    return Ok(quote! {
        impl #impl_generics crate::infer::InferenceInputSignature for #ident #ty_generics
        #where_clause
        {
            type Signature = #signature;

            const PARAM_INFOS: &'static [crate::infer::ParamInfo<
                crate::infer::InputScalarKind
            >] = {
                trait __ArrayExt {
                    type Scalar: crate::infer::InputScalar;
                    type Dimension: ::ndarray::Dimension + 'static;
                }

                impl<A: crate::infer::InputScalar, D: ::ndarray::Dimension + 'static> __ArrayExt
                    for ::ndarray::Array<A, D>
                {
                    type Scalar = A;
                    type Dimension = D;
                }

                &[#param_infos]
            };

            fn make_run_context<R: crate::infer::InferenceRuntime>(
                self,
                sess: &mut R::Session,
            ) -> ::anyhow::Result<R::RunContext<'_>> {
                let mut ctx = <R::RunContext<'_> as ::std::convert::From<_>>::from(sess);
                #(
                    __ArrayExt::push_to_ctx(self.#field_names, &mut ctx)?;
                )*
                return ::std::result::Result::Ok(ctx);

                trait __ArrayExt {
                    fn push_to_ctx(
                        self,
                        ctx: &mut impl crate::infer::PushInputTensor,
                    ) -> ::anyhow::Result<()>;
                }

                impl<A: crate::infer::InputScalar, D: ::ndarray::Dimension + 'static> __ArrayExt
                    for ::ndarray::Array<A, D>
                {
                    fn push_to_ctx(
                        self,
                        ctx: &mut impl crate::infer::PushInputTensor,
                    ) -> ::anyhow::Result<()> {
                        A::push_tensor_to_ctx(self, ctx)
                    }
                }
            }
        }
    });

    struct AssocTypeSignature(Type);

    impl Parse for AssocTypeSignature {
        fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
            let ItemType { ident, ty, .. } = input.parse()?;

            if ident != "Signature" {
                return Err(syn::Error::new(ident.span(), "expected `Signature`"));
            }
            Ok(Self(*ty))
        }
    }
}

pub(crate) fn derive_inference_output_signature(
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
                    dt: <<#ty as __ArrayExt>::Scalar as crate::infer::OutputScalar>::KIND,
                    ndim: <<#ty as __ArrayExt>::Dimension as ::ndarray::Dimension>::NDIM,
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
            >] = {
                trait __ArrayExt {
                    type Scalar: crate::infer::OutputScalar;
                    type Dimension: ::ndarray::Dimension + 'static;
                }

                impl<A: crate::infer::OutputScalar, D: ::ndarray::Dimension + 'static> __ArrayExt
                    for ::ndarray::Array<A, D>
                {
                    type Scalar = A;
                    type Dimension = D;
                }

                &[#param_infos]
            };
        }

        impl #impl_generics ::std::convert::TryFrom<::std::vec::Vec<crate::infer::OutputTensor>>
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
            return Err(syn::Error::new(enum_token.span(), "expected a struct"));
        }
        Data::Union(DataUnion { union_token, .. }) => {
            return Err(syn::Error::new(union_token.span(), "expected a struct"));
        }
    };

    Ok(fields
        .named
        .iter()
        .map(|Field { ident, ty, .. }| (ident.as_ref().expect("should be named"), ty))
        .collect())
}

fn unit_enum_variants(data: &Data) -> syn::Result<Vec<(&[Attribute], &syn::Ident)>> {
    let variants = match data {
        Data::Struct(DataStruct { struct_token, .. }) => {
            return Err(syn::Error::new(struct_token.span(), "expected an enum"));
        }
        Data::Enum(DataEnum { variants, .. }) => variants,
        Data::Union(DataUnion { union_token, .. }) => {
            return Err(syn::Error::new(union_token.span(), "expected an enum"));
        }
    };

    for Variant { fields, .. } in variants {
        if *fields != Fields::Unit {
            return Err(syn::Error::new(fields.span(), "must be unit"));
        }
    }

    Ok(variants
        .iter()
        .map(|Variant { attrs, ident, .. }| (&**attrs, ident))
        .collect())
}
