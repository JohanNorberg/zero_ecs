#![allow(
    unused_attributes,
    dead_code,
    unused_imports,
    unused_variables,
    unused_macro_rules,
    unused_macros,
    unused_mut
)]
use std::process::Command;
use std::{env, fs, path::Path};

use quote::ToTokens;
use quote::{format_ident, quote};
use syn::{Fields, Item, Meta};
macro_rules! debug {
    ($($arg:tt)*) => {
        println!("cargo:warning={}", format_args!($($arg)*));
    };
}
macro_rules! fident {
    ($name:expr) => {
        format_ident!("{}", $name)
    };
}
fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("could not get manifest dir");

    let main_rs_path = Path::new(&manifest_dir).join("src/main.rs");
    let collected = collect_data(main_rs_path.to_str().unwrap());

    debug!("{:?}", collected);
}

#[derive(Debug)]
struct EntityDef {
    name: String,
    fields: Vec<Field>,
}

#[derive(Debug)]
struct Field {
    name: String,
    data_type: String,
}

#[derive(Debug)]
struct CollectedData {
    entities: Vec<EntityDef>,
    queries: Vec<Query>,
}
#[derive(Debug)]
struct Query {
    mutable_fields: Vec<String>,
    const_fields: Vec<String>,
}
fn collect_data(path: &str) -> CollectedData {
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
                item_fn.attrs.iter().for_each(|attr| match &attr.meta {
                    Meta::Path(path) => {
                        if path.is_ident("system") {
                            // todo
                            debug!("{:?}", item_fn.sig.ident.to_string());
                        }
                    }
                    _ => {}
                });
            }
            _ => {}
        }
    }

    CollectedData { entities, queries }
}
