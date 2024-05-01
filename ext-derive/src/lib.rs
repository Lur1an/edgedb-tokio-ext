extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DeriveInput, GenericArgument, LitStr, PathArguments, Type, TypePath,
};

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
                        const_format::concatcp!(#left, " { ", #t::project(), " }, ")
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
                        const_format::concatcp!(#left, " { ", #t::project(), " }, ")
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

fn derive_projection(input: DeriveInput) -> proc_macro2::TokenStream {
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
            if !attr.path().is_ident("project") {
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
                if segment.ident == "Vec" {
                    let PathArguments::AngleBracketed(args) = &segment.arguments else {
                        panic!(
                            "Expected an inner type for nested field with Vec type: {}",
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
            const fn project() -> &'static str {
                const_format::concatcp!(#(#projections),*)
            }
        }
    };
    code
}

#[proc_macro_derive(Project, attributes(project))]
pub fn derive_projection_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_projection(input).into()
}

#[proc_macro]
pub fn query_project(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let query = quote! {
        "Deez nuts in ya mouf"
    };
    query.into()
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
                #[project(alias = "id")]
                user_id_value_what_am_i_doing: Uuid,
                #[project(exp = ".org.name")]
                org_name: String,
                #[project(alias = "org", nested)]
                organizations: Vec<Organization>,
            }
        };

        let derive_input: DeriveInput = syn::parse2(input).unwrap();
        let output = derive_projection(derive_input);
        println!("{}", output);
    }
}
