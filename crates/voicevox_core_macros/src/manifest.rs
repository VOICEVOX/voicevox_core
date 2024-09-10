use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, DeriveInput, Expr, Meta, Type};

pub(crate) fn derive_index_for_fields(
    input: &DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    const ATTR_NAME: &str = "index_for_fields";

    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = input;

    let idx = attrs
        .iter()
        .find_map(|Attribute { meta, .. }| match meta {
            Meta::List(list) if list.path.is_ident(ATTR_NAME) => Some(list),
            _ => None,
        })
        .ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                format!("missing `#[{ATTR_NAME}(…)]` in the struct itself"),
            )
        })?
        .parse_args::<Type>()?;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let targets = crate::extract::struct_fields(data)?
        .into_iter()
        .flat_map(|(attrs, name, output)| {
            let meta = attrs.iter().find_map(|Attribute { meta, .. }| match meta {
                Meta::List(meta) if meta.path.is_ident(ATTR_NAME) => Some(meta),
                _ => None,
            })?;
            Some((meta, name, output))
        })
        .map(|(meta, name, output)| {
            let key = meta.parse_args::<Expr>()?;
            Ok((key, name, output))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let (_, _, output) = targets.first().ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            format!("no fields have `#[{ATTR_NAME}(…)]`"),
        )
    })?;

    let arms = targets
        .iter()
        .map(|(key, name, _)| Ok(quote!(#key => &self.#name)))
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        impl #impl_generics ::std::ops::Index<#idx> for #ident #ty_generics #where_clause {
            type Output = #output;

            fn index(&self, index: #idx) -> &Self::Output {
                match index {
                    #(#arms),*
                }
            }
        }
    })
}
