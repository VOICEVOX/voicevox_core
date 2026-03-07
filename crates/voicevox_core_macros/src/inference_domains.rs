use derive_syn_parse::Parse;
use quote::ToTokens as _;
use syn::{
    Path, PathArguments, PathSegment, Token, Type, TypePath, parse_quote,
    visit_mut::{self, VisitMut},
};

pub(crate) fn substitute_type(input: Substitution) -> syn::Result<proc_macro2::TokenStream> {
    let Substitution {
        mut body,
        arg,
        replacement,
        replacement_as,
        ..
    } = input;

    Substitute {
        arg,
        replacement,
        replacement_as,
    }
    .visit_type_mut(&mut body);

    return Ok(body.to_token_stream());

    struct Substitute {
        arg: syn::Ident,
        replacement: Path,
        replacement_as: Path,
    }

    impl VisitMut for Substitute {
        fn visit_type_mut(&mut self, i: &mut Type) {
            visit_mut::visit_type_mut(self, i);

            let Type::Path(TypePath {
                qself: None,
                path:
                    Path {
                        leading_colon: None,
                        segments,
                    },
            }) = i
            else {
                return;
            };

            match &mut *segments.iter_mut().collect::<Vec<_>>() {
                [
                    PathSegment {
                        ident,
                        arguments: PathArguments::None,
                    },
                ] if *ident == self.arg => {
                    let replacement = self.replacement.clone();
                    *i = parse_quote!(#replacement);
                }
                [
                    PathSegment {
                        ident: ident1,
                        arguments: PathArguments::None,
                    },
                    seg,
                ] if *ident1 == self.arg => {
                    let replacement = self.replacement.clone();
                    let replacement_as = self.replacement_as.clone();
                    *i = parse_quote!(<#replacement as #replacement_as>::#seg);
                }
                _ => {}
            }
        }
    }
}

/// `$body:ty where $arg:ident = $replacement:path as $replacement_as:path`
#[derive(Parse)]
pub(crate) struct Substitution {
    body: Type,
    _where_token: Token![where],
    arg: syn::Ident,
    _eq_token: Token![=],
    replacement: Path,
    _as_token: Token![as],
    replacement_as: Path,
}
