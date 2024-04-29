extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, LitStr, Meta};

struct Projection<'a> {
    field_name: &'a syn::Ident,
}

enum ProjectionType {
    Alias(String),
    Expression(String),
    Nested(String),
}

fn derive_projection(input: DeriveInput) -> proc_macro2::TokenStream {
    let Data::Struct(data_struct) = input.data else {
        panic!("Project macro only applicable to structs");
    };
    let name = &input.ident;
    let mut projections = Vec::with_capacity(data_struct.fields.len());
    for field in &data_struct.fields {
        for attr in &field.attrs {
            let field_name = field.ident.as_ref().unwrap();
            if !attr.path().is_ident("project") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("alias") {
                    let value = meta.value()?;
                    let alias = value.parse::<LitStr>()?.value();
                    if alias.is_empty() {
                        return Err(meta.error("Expected non-empty alias"));
                    }
                    projections.push(format!("{} := .{}", field_name, alias));
                } else if meta.path.is_ident("exp") {
                    let value = meta.value()?;
                    let edgedb_expression = value.parse::<LitStr>()?.value();
                    if edgedb_expression.is_empty() {
                        return Err(meta.error("Expected non-empty expression"));
                    }
                    projections.push(format!("{} := {}", field_name, edgedb_expression));
                } else {
                    return Err(meta.error("Expected alias or exp identifier"));
                }
                Ok(())
            })
            .unwrap();
        }
    }
    println!("{:?}", projections);
    let code = quote! {
        impl edgedb_tokio_ext::QueryProjection for #name {
            fn project() -> &'static str {
                "SELECT 1"
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

    use super::derive_projection;

    #[test]
    fn test_macro_output() {
        let input = r#"
            struct User {
                #[project(alias = "id")]
                user_id_value_what_am_i_doing: Uuid,
                #[project(exp = ".org.name")]
                org_name: String,
            }
        "#;

        let input: proc_macro2::TokenStream = syn::parse_str(input).unwrap();
        let derive_input: DeriveInput = syn::parse2(input).unwrap();
        let output = derive_projection(derive_input);
    }
}
