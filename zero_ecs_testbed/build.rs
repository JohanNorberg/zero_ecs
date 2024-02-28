#![allow(
    unused_attributes,
    dead_code,
    unused_imports,
    unused_variables,
    unused_macro_rules,
    unused_macros,
    unused_mut
)]
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::{env, fs, path::Path};
use syn::{Fields, Item, ItemFn, Meta, PatType, PathArguments, Type};
use zero_ecs_build::*;
fn main() {
    let mut include_files = vec![];

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("could not get manifest dir");

    let main_rs_path = Path::new(&manifest_dir).join("src/main.rs");
    let collected = collect_data(main_rs_path.to_str().unwrap());

    debug!("{:?}", collected);

    generate_ecs(&out_dir, &mut include_files, &collected);

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

fn generate_ecs(out_dir: &str, include_files: &mut Vec<String>, collected: &CollectedData) {
    include_files.push(generate_queries(out_dir));

    let mut world_rs = vec![];

    let mut world_fields = vec![];

    for entity in collected.entities.iter() {
        let entity_name = &entity.name;
        let field_name = fident!(singular_to_plural(&pascal_case_to_snake_case(entity_name)));
        let archetype_type = fident!(singular_to_plural(entity_name));

        world_fields.push(quote! {
            #field_name: #archetype_type
        });
    }

    world_rs.push(quote! {
        #[derive(Default, Debug)]
        struct World {
            #(#world_fields)*
        }
    });

    let world_rs = quote! {
        #(#world_rs)*
    };

    include_files.push(write_token_stream_to_file(
        out_dir,
        "world.rs",
        &world_rs.to_string(),
    ));
}

fn singular_to_plural(name: &str) -> String {
    let last_char = name.chars().last().unwrap();
    if last_char == 'y' {
        return format!("{}ies", &name[0..name.len() - 1]);
    } else {
        return format!("{}s", name);
    }
}
fn pascal_case_to_snake_case(name: &str) -> String {
    // This function formats SomeString to some_string

    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
