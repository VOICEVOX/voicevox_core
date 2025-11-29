use derive_syn_parse::Parse;
use proc_macro2::Span;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
    Attribute, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Expr, Fields, Generics,
    ItemStatic, LitStr, Token, Variant,
};

pub(crate) fn derive_mora_mappings(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {
        attrs,
        ident: mora_kana_ty,
        generics,
        data,
        ..
    } = input;

    deny_generics(generics)?;

    let OutputStaticItems {
        mut mora_phonemes_to_mora_kana,
        mut mora_kana_to_mora_phonemes,
    } = find_attr(attrs, input.span())?.parse_args()?;

    let variants = unit_enum_variants(data)?
        .into_iter()
        .map(|(attrs, mora_kana_variant, span)| {
            let attr = find_attr(attrs, span)?;
            let VariantAttrs {
                consonant, vowel, ..
            } = attr.parse_args()?;
            Ok((mora_kana_variant, attr.span(), consonant, vowel))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    mora_phonemes_to_mora_kana.expr = {
        let variants = variants
            .iter()
            .map(|(mora_kana_variant, attr_span, consonant, vowel)| {
                let key = LitStr::new(&(consonant.value() + &vowel.value()), *attr_span);
                quote!(#key => #mora_kana_ty::#mora_kana_variant)
            });
        Box::new(syn::parse2(quote! {
            ::phf::phf_map! {
                #(#variants),*
            }
        })?)
    };

    mora_kana_to_mora_phonemes.expr = {
        let values = variants.iter().map(|(_, _, consonant, vowel)| {
            quote! {
                (
                    crate::engine::acoustic_feature_extractor::optional_consonant!(#consonant),
                    crate::engine::acoustic_feature_extractor::mora_tail!(#vowel)
                )
            }
        });
        Box::new(syn::parse2(quote! {
            ::enum_map::EnumMap::<#mora_kana_ty, _>::from_array([#(#values),*])
        })?)
    };

    return Ok(quote! {
        #mora_phonemes_to_mora_kana
        #mora_kana_to_mora_phonemes
    });

    struct OutputStaticItems {
        mora_phonemes_to_mora_kana: ItemStatic,
        mora_kana_to_mora_phonemes: ItemStatic,
    }

    impl Parse for OutputStaticItems {
        fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
            let mora_phonemes_to_mora_kana = parse(input, "mora_phonemes_to_mora_kana")?;
            let mora_kana_to_mora_phonemes = parse(input, "mora_kana_to_mora_phonemes")?;

            return Ok(Self {
                mora_phonemes_to_mora_kana,
                mora_kana_to_mora_phonemes,
            });

            fn parse(input: ParseStream<'_>, name: &'static str) -> syn::Result<ItemStatic> {
                let ident = input.parse::<syn::Ident>()?;
                if ident != name {
                    return Err(syn::Error::new(input.span(), format!("expected `{name}`")));
                }

                let content;
                braced!(content in input);
                let content = content.parse::<ItemStatic>()?;
                if !matches!(*content.expr, Expr::Infer(_)) {
                    return Err(syn::Error::new(content.expr.span(), "expected `_`"));
                }

                Ok(content)
            }
        }
    }

    #[derive(Parse)]
    struct VariantAttrs {
        consonant: LitStr,
        _comma: Token![,],
        vowel: LitStr,
    }

    fn find_attr(attrs: &[Attribute], span: Span) -> syn::Result<&Attribute> {
        static ATTR_NAME: &str = "mora_mappings";

        attrs
            .iter()
            .find(|a| a.path().is_ident(ATTR_NAME))
            .ok_or_else(|| syn::Error::new(span, format!("missing `#[{ATTR_NAME}(…)]`")))
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

    fn unit_enum_variants(data: &Data) -> syn::Result<Vec<(&[Attribute], &syn::Ident, Span)>> {
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
            .map(|v| (&*v.attrs, &v.ident, v.span()))
            .collect())
    }
}
