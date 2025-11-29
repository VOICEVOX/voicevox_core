use syn::{
    spanned::Spanned as _, Attribute, Data, DataEnum, DataStruct, DataUnion, Field, Fields, Type,
    Variant,
};

pub(crate) fn find_attr<'a>(
    attrs: &'a [Attribute],
    name: &'static str,
) -> syn::Result<&'a Attribute> {
    attrs
        .iter()
        .find(|a| a.path().is_ident(name))
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("missing `#[{name}(…)]`"),
            )
        })
}

pub(crate) fn struct_fields(data: &Data) -> syn::Result<Vec<(&[Attribute], &syn::Ident, &Type)>> {
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
        .map(
            |Field {
                 attrs, ident, ty, ..
             }| (&**attrs, ident.as_ref().expect("should be named"), ty),
        )
        .collect())
}

pub(crate) fn unit_enum_variants(data: &Data) -> syn::Result<Vec<(&[Attribute], &syn::Ident)>> {
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
