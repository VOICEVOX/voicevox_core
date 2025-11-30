use syn::{spanned::Spanned as _, Generics};

pub(crate) fn deny_generics(generics: &Generics) -> syn::Result<()> {
    if !generics.params.is_empty() {
        return Err(syn::Error::new(generics.params.span(), "must be empty"));
    }
    if let Some(where_clause) = &generics.where_clause {
        return Err(syn::Error::new(where_clause.span(), "must be empty"));
    }
    Ok(())
}
