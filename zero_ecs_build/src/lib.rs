use std::fs::{self};
use std::path::Path;
use std::process::Command;

use quote::{quote, ToTokens};
use syn::{Fields, Item, ItemFn, Meta, PathArguments, Type};

// export macro
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        println!("cargo:warning={}", format_args!($($arg)*));
    };
}
#[macro_export]
macro_rules! fident {
    ($name:expr) => {
        format_ident!("{}", $name)
    };
}
pub fn generate_queries(out_dir: &str) -> String {
    let file_name = "queries.rs";

    let code_rs = quote! {

        use std::marker::PhantomData;

        pub trait QueryFrom<'a, T> {
            fn query_from(&'a mut self) -> impl Iterator<Item = T>;
        }
        #[derive(Default, Debug)]
        struct AQuery<T> {
            phantom: PhantomData<T>,
        }

        impl<'a, T: 'a> AQuery<T> {
            fn iter_mut(&self, world: &'a mut World) -> impl Iterator<Item = T> + 'a
            where
                World: QueryFrom<'a, T>,
            {
                world.query_from()
            }
        }

        pub struct Query<'a, T> {
            a_query: AQuery<T>,
            world: &'a mut World,
        }

        impl<'a, T> Query<'a, T>
            where World: QueryFrom<'a, T>,
        {
            pub fn iter_mut(&'a mut self) -> impl Iterator<Item = T> + 'a {
                self.a_query.iter_mut(self.world)
            }
        }

        pub struct World {}
    };
    write_token_stream_to_file(out_dir, file_name, &code_rs.to_string())
}

fn write_token_stream_to_file(out_dir: &str, file_name: &str, code: &str) -> String {
    let dest_path = Path::new(&out_dir).join(file_name);
    fs::write(&dest_path, code.to_string())
        .expect(format!("failed to write to file: {}", file_name).as_str());
    format_file(&dest_path.to_str().unwrap());
    format!("/{}", file_name)
}
fn format_file(file_name: &str) {
    Command::new("rustfmt")
        .arg(file_name)
        .output()
        .expect("failed to execute rustfmt");
}

#[derive(Debug)]
pub struct EntityDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug)]
pub struct CollectedData {
    pub entities: Vec<EntityDef>,
    pub queries: Vec<Query>,
}
#[derive(Debug)]
pub struct Query {
    pub mutable_fields: Vec<String>,
    pub const_fields: Vec<String>,
}
pub fn collect_data(path: &str) -> CollectedData {
    let mut entities = vec![];
    let mut queries = vec![];

    let content = fs::read_to_string(path).expect(format!("Unable to read file {}", path).as_str());

    let parsed_file =
        syn::parse_file(&content).expect(format!("Unable to parse file {}", path).as_str());

    for item in parsed_file.items {
        match item {
            Item::Struct(item_struct) => {
                item_struct.attrs.iter().for_each(|attr| match &attr.meta {
                    Meta::Path(path) => {
                        if path.is_ident("entity") {
                            let mut fields = vec![];
                            if let Fields::Named(named_fields) = &item_struct.fields {
                                for field in &named_fields.named {
                                    let field = field.to_token_stream().to_string();
                                    let field = field.split(":").collect::<Vec<&str>>();
                                    fields.push(Field {
                                        name: field[0].trim().to_string(),
                                        data_type: field[1].trim().to_string(),
                                    });
                                }
                            }
                            entities.push(EntityDef {
                                name: item_struct.ident.to_string(),
                                fields,
                            });
                        }
                    }
                    _ => {}
                });
            }
            Item::Fn(item_fn) => {
                let is_system = is_system(&item_fn);
                if is_system {
                    // get all function parameters
                    for input in &item_fn.sig.inputs {
                        match input {
                            syn::FnArg::Receiver(_) => {}
                            syn::FnArg::Typed(pat_type) => {
                                let ty = pat_type.ty.clone();
                                match *ty {
                                    Type::Path(typed_path) => {
                                        for segment in typed_path.path.segments.iter() {
                                            let name = segment.ident.to_string();

                                            if name == "Query" {
                                                //debug!("{}", name);
                                                match &segment.arguments {
                                                    PathArguments::AngleBracketed(arguments) => {
                                                        if let Some(arg) =
                                                            &arguments.args.iter().next()
                                                        {
                                                            match arg {
                                                                syn::GenericArgument::Type(ty) => {
                                                                    // debug!(
                                                                    //     "{}",
                                                                    //     ty.to_token_stream()
                                                                    // );
                                                                    //print_type(ty);
                                                                    if let Some(query) =
                                                                        collect_query(ty)
                                                                    {
                                                                        queries.push(query);
                                                                    }
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    CollectedData { entities, queries }
}

fn collect_query(ty: &Type) -> Option<Query> {
    // handle reference, they have one value
    // "& mut Position"
    // "& Velocity"
    // also handle tuples, they have multiple, examples
    // (& mut Position , & Velocity)

    let query = match ty {
        Type::Reference(type_reference) => {
            let mut mutable_fields = vec![];
            let mut const_fields = vec![];
            let ty = type_reference.elem.clone();
            match *ty {
                Type::Path(type_path) => {
                    if type_reference.mutability.is_some() {
                        mutable_fields.push(type_path.to_token_stream().to_string());
                    } else {
                        const_fields.push(type_path.to_token_stream().to_string());
                    }
                }
                _ => {}
            }
            Some(Query {
                mutable_fields,
                const_fields,
            })
        }
        Type::Tuple(type_tuple) => {
            let mut mutable_fields = vec![];
            let mut const_fields = vec![];
            for elem in &type_tuple.elems {
                match elem {
                    Type::Reference(type_reference) => {
                        if type_reference.mutability.is_some() {
                            mutable_fields.push(type_reference.elem.to_token_stream().to_string());
                        } else {
                            const_fields.push(type_reference.elem.to_token_stream().to_string());
                        }
                    }
                    _ => {}
                }
            }
            Some(Query {
                mutable_fields,
                const_fields,
            })
        }
        _ => None,
    };

    if let Some(query) = query {
        if query.mutable_fields.len() == 0 && query.const_fields.len() == 0 {
            None
        } else {
            Some(query)
        }
    } else {
        None
    }
}

//only used for debugging
//pub fn _print_type(ty: &Type) {
//    match ty {
//        // Double dereference to get the `Type` from `&Box<Type>`
//        Type::Array(_) => debug!("Array"),
//        Type::BareFn(_) => debug!("BareFn"),
//        Type::Group(_) => debug!("Group"),
//        Type::ImplTrait(_) => debug!("ImplTrait"),
//        Type::Infer(_) => debug!("Infer"),
//        Type::Macro(_) => debug!("Macro"),
//        Type::Never(_) => debug!("Never"),
//        Type::Paren(_) => debug!("Paren"),
//        Type::Path(_) => debug!("Path"),
//        Type::Ptr(_) => debug!("Ptr"),
//        Type::Reference(_) => debug!("Reference"),
//        Type::Slice(_) => debug!("Slice"),
//        Type::TraitObject(_) => debug!("TraitObject"),
//        Type::Tuple(_) => debug!("Tuple"),
//        Type::Verbatim(_) => debug!("Verbatim"),
//        _ => {} // Add new variants here as needed
//    }
//}

fn is_system(item_fn: &ItemFn) -> bool {
    for attr in &item_fn.attrs {
        if let Meta::Path(path) = &attr.meta {
            if path.is_ident("system") {
                //debug!("{:?}", item_fn.sig.ident.to_string());
                return true;
            }
        }
    }
    false
}
