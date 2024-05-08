use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn, LitStr};

mod shape;
mod tx_variant;

#[proc_macro_derive(Shape, attributes(shape))]
pub fn derive_shape_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    shape::derive_shape(input).into()
}

#[proc_macro]
pub fn shaped_query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    shape::derive_shaped_query(input).into()
}

#[proc_macro_attribute]
pub fn tx_variant(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    tx_variant::derive_tx_variant(input).into()
}
