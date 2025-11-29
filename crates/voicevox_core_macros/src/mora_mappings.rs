use derive_syn_parse::Parse;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
    Attribute, DeriveInput, Expr, ItemStatic, LitStr, Token,
};

pub(crate) fn derive_mora_mappings(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {
        attrs,
        ident: mora_kana_ty,
        generics,
        data,
        ..
    } = input;

    crate::check::deny_generics(generics)?;

    let OutputStaticItems {
        mut mora_phonemes_to_mora_kana,
        mut mora_kana_to_mora_phonemes,
    } = find_attr(attrs)?.parse_args()?;

    let variants = crate::extract::unit_enum_variants(data)?
        .into_iter()
        .map(|(attrs, mora_kana_variant)| {
            let attr = find_attr(attrs)?;
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

    fn find_attr(attrs: &[Attribute]) -> syn::Result<&Attribute> {
        static NAME: &str = "mora_mappings";
        crate::extract::find_attr(attrs, NAME)
    }
}
