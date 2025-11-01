use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::{FnArg, ItemFn, ItemStruct, Pat, PatIdent, PatType, Type};

use crate::query_impl::get_collection_component_fields;

pub fn system_for_each(attr: TokenStream, item: TokenStream) -> TokenStream {
    let foreign_struct = syn::parse_macro_input!(attr as ItemStruct);
    let collection_component_fields = get_collection_component_fields(foreign_struct);
    let component_types: HashSet<_> = collection_component_fields
        .iter()
        .map(|ccf| &ccf.field_type)
        .collect();
    let input_fn = syn::parse_macro_input!(item as ItemFn);

    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_name = &fn_sig.ident;
    let fn_block = &input_fn.block;

    let mut query_fields = Vec::new();
    let mut call_args = Vec::new();
    let mut any_mutable_arguments = false;
    let mut resource_args = Vec::new();
    let mut resource_params = Vec::new();
    let mut all_args = Vec::new();

    for arg in &fn_sig.inputs {
        match arg {
            FnArg::Receiver(_) => {
                panic!("#[system_for_each] functions cannot take self")
            }
            FnArg::Typed(PatType { pat, ty, .. }) => {
                // get the name (pat)
                let arg_ident = if let Pat::Ident(PatIdent { ident, .. }) = &**pat {
                    ident.clone()
                } else {
                    panic!("Unsupported argument pattern in #[system_for_each]");
                };

                // &** i don't understand but do what the compiler tells me.
                if let Type::Reference(ty) = &**ty {
                    let is_mutable = ty.mutability.is_some();

                    let Type::Path(type_path) = &*ty.elem else {
                        panic!("failed to get elem path #[system_for_each]");
                    };

                    let Some(type_path) = type_path.path.get_ident() else {
                        panic!("failed to get type path #[system_for_each]")
                    };

                    let type_path_str = type_path.to_string();
                    let is_component = component_types.iter().any(|ty| **ty == type_path_str);

                    if is_component {
                        call_args.push(arg_ident.clone());
                        if is_mutable {
                            any_mutable_arguments = true;
                            query_fields.push(quote! {&'a mut #type_path});
                        } else {
                            query_fields.push(quote! {&'a #type_path});
                        }
                    } else {
                        resource_args.push(arg_ident.clone());
                        resource_params.push(arg);
                    }
                    all_args.push(arg_ident.clone());
                } else {
                    panic!("Only references in #[system_for_each]");
                }
            }
        }
    }

    let fn_call = quote! {
        #fn_name(#(#all_args),*);
    };

    let ext_name = format_ident!("__ext_{}", fn_name);
    let mut_code = quote! {
        self.with_query_mut(Query::<QueryObject>::new())
            .iter_mut()
            .for_each(|QueryObject(#(#call_args),*)| {
                #fn_call
            });
    };

    let ref_code = quote! {
        self.with_query(Query::<QueryObject>::new())
            .iter()
            .for_each(|QueryObject(#(#call_args),*)| {
                #fn_call
            });
    };

    let code = if any_mutable_arguments {
        mut_code
    } else {
        ref_code
    };

    let resource_args_params = if resource_args.is_empty() {
        quote! {}
    } else {
        quote! { #(#resource_params),* }
    };

    // Build the output
    let expanded = quote! {
        #fn_vis #fn_sig {
            #fn_block
        }

        #[ext(name = #ext_name)]
        pub impl World {
            fn #fn_name(&mut self, #resource_args_params) {
                #[query(World)]
                struct QueryObject<'a>(#(#query_fields),*);

                #code
            }
        }
    };

    expanded.into()
}
