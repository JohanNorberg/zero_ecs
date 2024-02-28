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

use quote::quote;
use quote::ToTokens;
use std::fs::File;
use std::io::Write;
use syn::{Fields, Item, ItemFn, Meta, PatType, PathArguments, Type};
use zero_ecs_build::*;
fn main() {
    let mut include_files = vec![];
    let out_dir = std::env::var("OUT_DIR").unwrap();
    include_files.push(generate_queries(&out_dir));

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("could not get manifest dir");

    let main_rs_path = Path::new(&manifest_dir).join("src/main.rs");
    let collected = collect_data(main_rs_path.to_str().unwrap());

    debug!("{:?}", collected);

    let main_file = Path::new(&out_dir).join("zero_ecs.rs");

    let mut include_rs = vec![];
    for file in include_files {
        include_rs.push(quote! {
            include!(concat!(env!("OUT_DIR"), #file));
        });
    }

    let zero_ecs_rs = quote! {
        #(#include_rs)*
    };

    let mut f = fs::File::create(main_file).expect("Unable to create file");

    write!(f, "{}", zero_ecs_rs).expect("Unable to write data to file");
}
