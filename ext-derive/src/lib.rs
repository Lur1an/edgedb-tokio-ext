extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, LitStr};

fn derive_projection(input: DeriveInput) -> TokenStream {
    let Data::Struct(data_struct) = input.data else {
        panic!("Project macro only applicable to structs");
    };
    let name = &input.ident;
    // let mut projections = Vec::with_capacity(data_struct.fields.len());
    data_struct.fields.iter().for_each(|field| {
        for attr in &field.attrs {
            if !matches!(attr.style, syn::AttrStyle::Outer) {
                continue;
            }
            if attr.path().is_ident("alias") {}
        }
    });
    let code = quote! {
        impl edgedb_tokio_ext::QueryProjection for #name {
            fn project() -> &'static str {
                "SELECT 1"
            }
        }
    };
    code.into()
}

#[proc_macro_derive(Project, attributes(alias, exp))]
pub fn derive_projection_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_projection(input)
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
    use syn::parse::Parse;

    use super::*;

    #[test]
    fn test_macro_output() {
        let input = r#"
            struct User {
                id: i32,
                name: String,
            }
        "#;

        let input: proc_macro2::TokenStream = syn::parse_str(input).unwrap();
        let derive_input: DeriveInput = syn::parse2(input).unwrap();
        let output = derive_projection(derive_input).to_string();
        println!("{}", output);
    }
}
