extern crate proc_macro;

use lazy_static::lazy_static;
use quote::{quote, ToTokens};
use regex::Regex;
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::{Data, DeriveInput, GenericArgument, LitStr, PathArguments, Type, TypePath};

#[derive(Debug)]
struct Projection<'a> {
    field_name: &'a syn::Ident,
    projection_type: ProjectionType,
    nested_projection_type: Option<&'a syn::Ident>,
}

#[derive(Debug)]
enum ProjectionType {
    FieldName,
    Alias(String),
    Expression(String),
}

impl<'a> ToTokens for Projection<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.projection_type {
            ProjectionType::FieldName => {
                if let Some(t) = self.nested_projection_type {
                    let left = format!("{} := .{}", self.field_name, self.field_name);
                    quote! {
                        edgedb_tokio_ext::const_format::concatcp!(#left, " { ", #t::shape(), " }, ")
                    }
                    .to_tokens(tokens);
                } else {
                    let projection = format!("{}, ", self.field_name);
                    quote! { #projection }.to_tokens(tokens)
                }
            }
            ProjectionType::Expression(exp) => {
                let projection = format!("{} := {}, ", self.field_name, exp);
                quote! { #projection }.to_tokens(tokens)
            }
            ProjectionType::Alias(alias) => {
                if let Some(t) = self.nested_projection_type {
                    let left = format!("{} := .{}", self.field_name, alias);
                    quote! {
                        edgedb_tokio_ext::const_format::concatcp!(#left, " { ", #t::shape(), " }, ")
                    }
                    .to_tokens(tokens);
                } else {
                    let projection = format!("{} := .{}, ", self.field_name, alias);
                    quote! { #projection }.to_tokens(tokens)
                }
            }
        }
    }
}

pub fn derive_shape(input: DeriveInput) -> proc_macro2::TokenStream {
    let Data::Struct(data_struct) = input.data else {
        panic!("Project macro only applicable to structs");
    };
    let name = &input.ident;
    let mut projections = Vec::with_capacity(data_struct.fields.len());
    for field in &data_struct.fields {
        let field_name = field.ident.as_ref().unwrap();
        let mut projection_type = None::<ProjectionType>;
        let mut nested = false;
        for attr in &field.attrs {
            if !attr.path().is_ident("shape") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("alias") {
                    if projection_type.is_some() {
                        return Err(meta.error("Only one of `alias` or `exp` attributes allowed"));
                    }
                    let value = meta.value()?;
                    let alias = value.parse::<LitStr>()?.value();
                    if alias.is_empty() {
                        return Err(meta.error("Expected non-empty alias"));
                    }
                    projection_type = Some(ProjectionType::Alias(alias));
                } else if meta.path.is_ident("exp") {
                    if projection_type.is_some() {
                        return Err(meta.error("Only one of `alias` or `exp` attributes allowed"));
                    }
                    let value = meta.value()?;
                    let edgedb_expression = value.parse::<LitStr>()?.value();
                    if edgedb_expression.is_empty() {
                        return Err(meta.error("Expected non-empty expression"));
                    }
                    projection_type = Some(ProjectionType::Expression(edgedb_expression));
                } else if meta.path.is_ident("nested") {
                    nested = true;
                } else {
                    return Err(meta.error("Unknown attribute"));
                }
                Ok(())
            })
            .unwrap();
        }
        let mut nested_projection = None::<&syn::Ident>;
        if nested {
            if let syn::Type::Path(ref type_path) = field.ty {
                let segment = &type_path.path.segments.last().unwrap();
                if segment.ident == "Vec" || segment.ident == "Option" {
                    let PathArguments::AngleBracketed(args) = &segment.arguments else {
                        panic!(
                            "Expected an inner type for nested field with Vec or Option type: {}",
                            field_name
                        );
                    };
                    if let Some(GenericArgument::Type(Type::Path(TypePath { path, .. }))) =
                        args.args.first()
                    {
                        nested_projection = Some(&path.segments.last().unwrap().ident);
                    }
                } else {
                    nested_projection = Some(&segment.ident);
                }
            } else {
                panic!("Expected syn::Type::Path for nested field {}", field_name);
            }
        }
        projections.push(Projection {
            field_name,
            projection_type: projection_type.unwrap_or(ProjectionType::FieldName),
            nested_projection_type: nested_projection,
        });
    }
    let code = quote! {
        impl #name {
            const fn shape() -> &'static str {
                edgedb_tokio_ext::const_format::concatcp!(#(#projections),*)
            }
        }
    };
    code
}

pub fn derive_shaped_query(input: LitStr) -> proc_macro2::TokenStream {
    lazy_static! {
        static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
        static ref PROJECT_REGEX: Regex = Regex::new(r"shape::(\S+)").unwrap();
    }
    let mut replacements = vec![];
    for projection in PROJECT_REGEX.captures_iter(&input.value()) {
        let string_match = projection.get(0).unwrap().as_str();
        let projection_entity_name = projection.get(1).unwrap().as_str().to_owned();
        let projection_entity_type =
            syn::Ident::new(&projection_entity_name, proc_macro2::Span::call_site());
        replacements.push(quote! {
            .replace(#string_match, #projection_entity_type::shape())
        });
    }
    let query_num = COUNTER.fetch_add(1, Ordering::Relaxed);
    let query_const_name = format!("__QUERY_{}", query_num);
    let query_const_ident = syn::Ident::new(&query_const_name, proc_macro2::Span::call_site());
    let code = quote! {
        {
            static #query_const_ident: std::sync::OnceLock<String> = std::sync::OnceLock::new();
            #query_const_ident.get_or_init(|| {
                #input #(#replacements)*
            })
        }
    };
    code
}

#[cfg(test)]
mod test {
    use syn::DeriveInput;

    use super::*;

    #[test]
    fn test_macro_output() {
        let input = quote! {
            struct User {
                id: Uuid,
                #[shape(exp = ".org.name")]
                org_name: String,
                #[shape(nested)]
                manager: User,
                #[shape(alias = "org", nested)]
                organizations: Vec<Organization>,
            }
        };

        let derive_input: DeriveInput = syn::parse2(input).unwrap();
        let output = derive_shape(derive_input).to_string();
        assert!(output.contains("id, "));
    }
}
