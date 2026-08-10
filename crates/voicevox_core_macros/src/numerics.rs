use quote::quote;
use syn::LitInt;

pub(crate) fn int_type(lit: &LitInt) -> syn::Result<proc_macro2::TokenStream> {
    if lit.suffix().is_empty() {
        return Err(syn::Error::new(lit.span(), "suffix needed"));
    }
    let suffix = syn::parse_str::<syn::Ident>(lit.suffix())?;
    Ok(quote!(::core::primitive::#suffix))
}
