use syn::{
    spanned::Spanned as _, Attribute, Data, DataEnum, DataStruct, DataUnion, Field, Fields, Type,
};

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
