extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, ItemStruct, Pat, PatType};

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
    let input_fn = parse_macro_input!(input as ItemFn);
    quote! {
        #[make_mut]
        #input_fn
    }
    .into()
}

#[proc_macro_attribute]
pub fn make_mut(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_fn = parse_macro_input!(item as ItemFn);

    // Clone the function signature for modification
    let modified_fn = input_fn.clone();

    // This vector will hold the statements to prepend to the function body
    let mut prepends = vec![];

    // Iterate over function inputs to find parameters that are not references
    for input in modified_fn.sig.inputs.iter() {
        if let FnArg::Typed(PatType { pat, ty, .. }) = input {
            if !matches!(**ty, syn::Type::Reference(_)) {
                if let Pat::Ident(ident) = pat.as_ref() {
                    // Prepare the let mut statement for non-reference parameters
                    let ident_name = &ident.ident;
                    prepends.push(quote! {
                        let mut #ident_name = #ident_name;
                    });
                }
            }
        }
    }

    // Reconstruct the function, prepending the new statements to the original body
    let fn_body = &modified_fn.block;
    let stmts = quote! {
        #(#prepends)*
        #fn_body
    };

    // Reconstruct the function with the modified body
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let output = quote! {
        #vis #sig {
            #stmts
        }
    };

    TokenStream::from(output)
}
