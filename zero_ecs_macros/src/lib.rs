extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn entity(_: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);

    quote! {
        #[derive(Default, Debug)]
        #input_struct
    }
    .into()
}

#[proc_macro_attribute]
pub fn component(_: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);

    quote! {
        #[derive(Default, Debug)]
        #input_struct
    }
    .into()
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn system(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}
