use intehan_util_dump::dump;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, FnArg, GenericArgument, Ident, ItemFn, Pat, PatIdent, PatType,
    PathArguments, Type,
};

pub fn system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_name = &fn_sig.ident;
    let fn_block = &input_fn.block;

    let system_args: Vec<SystemArg> = collect_fn_sig(&fn_sig);
    assert_eq!(
        system_args
            .iter()
            .filter(|a| matches!(a, SystemArg::World(_)))
            .count(),
        1,
        "#[system] must have exactly ONE &World or &mut World argument"
    );

    let query_args: Vec<_> = system_args
        .iter()
        .filter_map(|arg| {
            if let SystemArg::Query(query) = arg {
                Some(query)
            } else {
                None
            }
        })
        .collect();

    let query_codes: Vec<_> = query_args
        .iter()
        .map(|arg| {
            let struct_name = format_ident!("Query__{}", arg.name);
            let arg_name = &arg.name_ident;

            let components: Vec<_> = arg
                .fields
                .iter()
                .map(|field| {
                    let ty_ident = format_ident!("{}", field.ty);

                    if field.mutable {
                        quote! {
                            &'a mut #ty_ident
                        }
                    } else {
                        quote! {
                            &'a #ty_ident
                        }
                    }
                })
                .collect();

            let code = quote! {
                #[query(World)]
                struct #struct_name<'a>(#(#components),*);

                let #arg_name = Query::<#struct_name>::new();
            };

            code
        })
        .collect();

    let out_fn_args: Vec<_> = system_args
        .iter()
        .filter_map(|arg| arg.get_arg_code())
        .collect();
    let resource_fn_args: Vec<_> = system_args
        .iter()
        .filter(|arg| matches!(arg, SystemArg::Resource(_)))
        .filter_map(|arg| arg.get_arg_code())
        .collect();
    let call_fn_code: Vec<_> = system_args
        .iter()
        .filter_map(|arg| arg.get_call_code())
        .collect();

    let ext_name = format_ident!("__ext_{}", fn_name);
    let expanded = quote! {
        #fn_vis fn #fn_name(#(#out_fn_args),*) {
            #(#query_codes)*

            #fn_block
        }


        #[ext(name = #ext_name)]
        pub impl World {
            fn #fn_name(&mut self, #(#resource_fn_args),*) {
                #fn_name(#(#call_fn_code),*);
            }
        }
    };

    expanded.into()
}

#[derive(Debug)]
struct ArgQueryField {
    mutable: bool,
    ty: String,
}

#[derive(Debug)]
struct ArgQuery {
    name: String,
    name_ident: Ident,
    fields: Vec<ArgQueryField>,
}

#[derive(Debug)]
struct ArgResource {
    name_ident: Ident,
    mutable: bool,
    ty: String,
    by_ref: bool,
}

#[derive(Debug)]
struct ArgWorld {
    name_ident: Ident,
    mutable: bool,
}

#[derive(Debug)]
enum SystemArg {
    World(ArgWorld),
    Query(ArgQuery),
    Resource(ArgResource),
}

impl SystemArg {
    fn get_arg_code(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            SystemArg::World(arg_world) => {
                let name = &arg_world.name_ident;
                Some(if arg_world.mutable {
                    quote! { #name: &mut World }
                } else {
                    quote! { #name: &World }
                })
            }
            SystemArg::Resource(arg_resource) => {
                let name = &arg_resource.name_ident;
                let ty = format_ident!("{}", arg_resource.ty);
                if arg_resource.by_ref {
                    Some(if arg_resource.mutable {
                        quote! { #name: &mut #ty }
                    } else {
                        quote! { #name: &#ty }
                    })
                } else {
                    Some(if arg_resource.mutable {
                        quote! { #name: mut #ty }
                    } else {
                        quote! { #name: #ty }
                    })
                }
            }
            _ => None,
        }
    }

    // for when calling fn (self, foo, bar)
    fn get_call_code(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            SystemArg::World(_) => Some(quote! { self }),
            SystemArg::Resource(arg_resource) => {
                let name = &arg_resource.name_ident;
                Some(quote! { #name })
            }
            _ => None,
        }
    }
}

fn collect_fn_sig(fn_sig: &&syn::Signature) -> Vec<SystemArg> {
    let mut system_args = vec![];

    for arg in &fn_sig.inputs {
        match arg {
            FnArg::Receiver(_) => {
                panic!("#[system] functions cannot take self")
            }
            FnArg::Typed(PatType { pat, ty, .. }) => {
                // get the name (pat)
                let arg_ident = if let Pat::Ident(PatIdent { ident, .. }) = &**pat {
                    ident.clone()
                } else {
                    panic!("Unsupported argument pattern in #[system_for_each]");
                };

                // &** i don't understand but do what the compiler tells me.
                match &**ty {
                    Type::Reference(ty) => {
                        let is_mutable = ty.mutability.is_some();
                        let Type::Path(type_path) = &*ty.elem else {
                            panic!("Reference: failed to get elem path #[system]");
                        };
                        let is_world = type_path
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == "World")
                            .expect("path should have one last segment");
                        let Some(type_path) = type_path.path.get_ident() else {
                            panic!("Reference: failed to get type path #[system]")
                        };

                        if is_world {
                            let arg = ArgWorld {
                                mutable: is_mutable,
                                name_ident: arg_ident,
                            };
                            system_args.push(SystemArg::World(arg));
                        } else {
                            let arg = ArgResource {
                                mutable: is_mutable,
                                ty: type_path.to_string(),
                                by_ref: true,
                                name_ident: arg_ident,
                            };
                            system_args.push(SystemArg::Resource(arg));
                        }
                    }
                    Type::Path(ty) => {
                        let segment = &ty
                            .path
                            .segments
                            .first()
                            .expect("#[system] no first segment");
                        let outer_ident = &segment.ident;

                        if outer_ident == "QueryDef" {
                            let PathArguments::AngleBracketed(args) = &segment.arguments else {
                                panic!("#[system] Expected angle bracketed arguments for QueryDef, but none were found");
                            };

                            let arg = &args
                                .args
                                .first()
                                .expect("#[system] args args should not be empty");

                            match arg {
                                GenericArgument::Type(Type::Tuple(tuple)) => {
                                    let mut arg_query_fields = vec![];

                                    for elem in &tuple.elems {
                                        let Type::Reference(elem) = elem else {
                                            panic!("#[system] Expected a reference type inside the tuple, but found something else");
                                        };
                                        let is_mutable = elem.mutability.is_some();
                                        let Type::Path(elem_path) = &*elem.elem else {
                                            panic!("#[system] Expected a path type inside the reference, but found something else");
                                        };
                                        let elem_ident =
                                            &elem_path.path.segments.first().unwrap().ident;

                                        arg_query_fields.push(ArgQueryField {
                                            mutable: is_mutable,
                                            ty: elem_ident.to_string(),
                                        });
                                    }

                                    let arg_query = ArgQuery {
                                        name: arg_ident.to_string(),
                                        name_ident: arg_ident,
                                        fields: arg_query_fields,
                                    };
                                    system_args.push(SystemArg::Query(arg_query));
                                }
                                GenericArgument::Type(Type::Reference(elem)) => {
                                    let is_mutable = elem.mutability.is_some();
                                    let Type::Path(elem_path) = &*elem.elem else {
                                        panic!("#[system] Expected a path type inside the reference, but found something else");
                                    };
                                    let elem_ident =
                                        &elem_path.path.segments.first().unwrap().ident;
                                    let arg_query = ArgQuery {
                                        name: arg_ident.to_string(),
                                        name_ident: arg_ident,
                                        fields: vec![ArgQueryField {
                                            mutable: is_mutable,
                                            ty: elem_ident.to_string(),
                                        }],
                                    };
                                    system_args.push(SystemArg::Query(arg_query))
                                }
                                GenericArgument::Type(Type::Paren(type_paren)) => {
                                    let Type::Reference(ref elem) = *type_paren.elem else {
                                        panic!("#[system] Expected a reference type inside the tuple, but found something else");
                                    };

                                    let is_mutable = elem.mutability.is_some();
                                    let Type::Path(elem_path) = &*elem.elem else {
                                        panic!("#[system] Expected a path type inside the reference, but found something else");
                                    };
                                    let elem_ident =
                                        &elem_path.path.segments.first().unwrap().ident;
                                    let arg_query = ArgQuery {
                                        name: arg_ident.to_string(),
                                        name_ident: arg_ident,
                                        fields: vec![ArgQueryField {
                                            mutable: is_mutable,
                                            ty: elem_ident.to_string(),
                                        }],
                                    };
                                    system_args.push(SystemArg::Query(arg_query))
                                }
                                GenericArgument::Type(ty) => {
                                    panic!("#[system] Unsupported type in QueryDef: {:?}", ty);
                                }
                                _ => {
                                    panic!(
                                        "#[system] Unsupported generic argument type: {:?}",
                                        arg
                                    );
                                }
                            }
                        } else {
                            let Some(ty_ident) = ty.path.get_ident() else {
                                panic!("#[system] failed tog get_ident ty.path");
                            };
                            let arg = ArgResource {
                                mutable: false,
                                ty: ty_ident.to_string(),
                                by_ref: false,
                                name_ident: arg_ident,
                            };
                            system_args.push(SystemArg::Resource(arg));
                        }
                    }
                    _ => {
                        dump!(ty);
                        panic!("#[system] unsuppoerd type");
                    }
                }
            }
        }
    }
    system_args
}
