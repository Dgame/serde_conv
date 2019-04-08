extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;

enum Convert {
    From(String),
    Into(String),
    Maybe(String),
    Extract(String),
    Deflate(String),
}

fn interpret_meta(attr: &syn::Attribute) -> Option<String> {
    use syn::Meta::{List, NameValue, Word};
    use syn::NestedMeta::Meta;

    match attr.interpret_meta() {
        Some(NameValue(mnv)) => {
            if let syn::Lit::Str(ls) = mnv.lit {
                Some(ls.value())
            } else {
                None
            }
        }
        Some(List(ref meta)) => meta
            .nested
            .iter()
            .filter_map(|nested| match nested {
                Meta(meta) => match meta {
                    Word(ident) => Some(ident.to_string()),
                    _ => None,
                },
                _ => None,
            })
            .map(Some)
            .take(1)
            .collect(),
        _ => {
            // TODO: produce an error
            None
        }
    }
}

fn get_meta_items(attr: &syn::Attribute) -> Option<Convert> {
    if attr.path.segments.len() == 1 {
        match attr.path.segments[0].ident.to_string().as_str() {
            "from" => {
                if let Some(ident) = interpret_meta(attr) {
                    Some(Convert::From(ident))
                } else {
                    // TODO: produce an error
                    None
                }
            }
            "into" => {
                if let Some(ident) = interpret_meta(attr) {
                    Some(Convert::Into(ident))
                } else {
                    // TODO: produce an error
                    None
                }
            }
            "maybe" => {
                if let Some(ident) = interpret_meta(attr) {
                    Some(Convert::Maybe(ident))
                } else {
                    // TODO: produce an error
                    None
                }
            }
            "extract" => {
                if let Some(ident) = interpret_meta(attr) {
                    Some(Convert::Extract(ident))
                } else {
                    // TODO: produce an error
                    None
                }
            }
            "deflate" => {
                if let Some(ident) = interpret_meta(attr) {
                    Some(Convert::Deflate(ident))
                } else {
                    // TODO: produce an error
                    None
                }
            }
            _ => None, // TODO: produce an error
        }
    } else {
        None
    }
}

#[proc_macro_attribute]
pub fn convert(_: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_convert(&ast)
}

fn collect_attributes(attributes: &[syn::Attribute]) -> Vec<proc_macro2::TokenStream> {
    let mut attrs = Vec::new();
    for attr in attributes.iter() {
        attrs.push(quote! { #attr });
    }

    attrs
}

fn impl_convert(ast: &syn::DeriveInput) -> TokenStream {
    use syn::Data;

    let name = &ast.ident;
    let attrs = collect_attributes(&ast.attrs);
    let mut fields = Vec::new();

    match &ast.data {
        Data::Struct(ds) => {
            for field in ds.fields.iter() {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                let mut field_attributes = Vec::new();
                for attr in field.attrs.iter() {
                    if let Some(convert) = get_meta_items(attr) {
                        let (conv_type, suffix) = match convert {
                            Convert::From(ty) => (ty, "from"),
                            Convert::Into(ty) => (ty, "into"),
                            Convert::Maybe(ty) => (ty, "maybe"),
                            Convert::Extract(attr) => (attr[1..].to_string(), "extract"),
                            Convert::Deflate(attr) => (attr[1..].to_string(), "deflate"),
                        };

                        let de_with = format!(
                            r#"#[serde(deserialize_with = "serde_conv::de::{}_{}")]"#,
                            suffix, conv_type
                        );
                        let de_with: proc_macro2::TokenStream = syn::parse_str(&de_with).unwrap();
                        field_attributes.push(quote! { #de_with });
                    } else {
                        field_attributes.push(quote! { #attr });
                    }
                }

                fields.push(quote! {
                    #(#field_attributes)*
                    #field_name: #field_type
                });
            }

            let result = quote! {
                #(#attrs)*
                struct #name {
                    #(#fields),*
                }
            };
            result.into()
        }
        _ => panic!("{} is not a struct", name),
    }
}
