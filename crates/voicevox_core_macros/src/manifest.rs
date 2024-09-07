use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, DeriveInput, Expr, Meta, Type};

pub(crate) fn derive_index(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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
            Meta::List(list) if list.path.is_ident("index") => Some(list),
            _ => None,
        })
        .ok_or_else(|| syn::Error::new(Span::call_site(), "missing `#[index(…)]`"))?
        .parse_args::<Type>()?;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let targets = crate::extract::struct_fields(data)?
        .into_iter()
        .flat_map(|(attrs, name, ty)| {
            let list = attrs.iter().find_map(|Attribute { meta, .. }| match meta {
                Meta::List(list) if list.path.is_ident("index") => Some(list),
                _ => None,
            })?;
            Some((list, name, ty))
        })
        .map(|(list, name, ty)| {
            let key = list.parse_args::<Expr>()?;
            Ok((key, name, ty))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let (_, _, output) = targets
        .first()
        .ok_or_else(|| syn::Error::new(Span::call_site(), "no fields with `#[index(…)]`"))?;

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
