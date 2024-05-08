use quote::{quote, ToTokens};
use syn::{parse_quote, ItemFn};

pub fn derive_tx_variant(mut func: ItemFn) -> proc_macro2::TokenStream {
    let ident = &func.sig.ident;
    let mut code = quote! {
        #func
    };
    let new_ident = syn::Ident::new(&format!("{}_tx", ident), ident.span());

    // Change the argument type
    for arg in func.sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Type::Reference(type_ref) = &*pat_type.ty {
                if let syn::Type::Path(type_path) = &*type_ref.elem {
                    // Check both "Client" and "edgedb_tokio::Client"
                    let is_client = type_path.path.is_ident("Client")
                        || type_path.path.segments.last().map_or(false, |segment| {
                            segment.ident == "Client" && segment.arguments.is_empty()
                        });

                    let is_full_path_client = type_path
                        .path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.to_string())
                        .collect::<Vec<_>>()
                        == vec!["edgedb_tokio", "Client"];

                    if is_client || is_full_path_client {
                        // Change to &mut Transaction
                        pat_type.ty = parse_quote!(&mut edgedb_tokio::Transaction);
                    }
                }
            }
        }
    }

    // Rename the function
    func.sig.ident = new_ident;
    quote! {
        #func
    }
    .to_tokens(&mut code);
    code
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_macro_output() {
        let input = parse_quote! {
            pub async fn my_db_func(client: &edgedb_tokio::Client) -> anyhow::Result<Deez> {
                todo!()
            }
        };
        let output = derive_tx_variant(input).to_string();
        assert!(output.contains("pub async fn my_db_func_tx"));
        assert!(output.contains("Transaction"));
    }
}
