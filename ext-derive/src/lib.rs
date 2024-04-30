extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, LitStr};

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

fn parse_projection(attr: &Attribute) -> syn::Result<ProjectionType> {
    let mut projection = None::<ProjectionType>;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("alias") {
            let value = meta.value()?;
            let alias = value.parse::<LitStr>()?.value();
            if alias.is_empty() {
                return Err(meta.error("Expected non-empty alias"));
            }
            projection = Some(ProjectionType::Alias(alias));
        } else if meta.path.is_ident("exp") {
            let value = meta.value()?;
            let edgedb_expression = value.parse::<LitStr>()?.value();
            if edgedb_expression.is_empty() {
                return Err(meta.error("Expected non-empty expression"));
            }
            projection = Some(ProjectionType::Expression(edgedb_expression));
        } else {
            return Err(meta.error("Expected alias or exp identifier"));
        }
        Ok(())
    })?;
    Ok(projection.unwrap())
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
            if attr.path().is_ident("project") {
                if projection_type.is_some() {
                    panic!("Multiple projections on field {}", field_name);
                }
                projection_type = Some(parse_projection(attr).unwrap());
            }
            if attr.path().is_ident("nested") {
                nested = true;
            }
        }
        let mut nested_projection = None::<&syn::Ident>;
        if nested {
            if let syn::Type::Path(ref type_path) = field.ty {
                nested_projection = Some(&type_path.path.segments.last().unwrap().ident);
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

#[proc_macro_derive(Project, attributes(project, nested))]
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
                #[nested]
                #[project(alias = "org")]
                organization: Organization,
            }
        };

        let derive_input: DeriveInput = syn::parse2(input).unwrap();
        let output = derive_projection(derive_input);
        println!("{}", output);
    }
}
