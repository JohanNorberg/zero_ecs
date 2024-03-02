mod code_collection;
mod code_generation;
mod file;
mod macros;

use quote::quote;
use regex::Regex;
use std::io::Write;
use std::{env, fs, path::Path};
use walkdir::WalkDir;

pub use code_collection::*;
pub use code_generation::*;
pub use file::*;

pub fn generate_ecs(source_regex: &str) {
    let pattern = match Regex::new(source_regex) {
        Ok(r) => r,
        Err(e) => {
            panic!("Invalid regex pattern: {}", e);
        }
    };
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("could not get manifest dir");

    let files_to_look_in: Vec<_> = WalkDir::new(&manifest_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| pattern.is_match(e.path().to_str().unwrap()))
        // map to string
        .map(|e| e.path().to_str().unwrap().to_string())
        .collect();

    let mut include_files = vec![];

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let mut collected_datas: Vec<CollectedData> = vec![];

    for file in files_to_look_in {
        let collected = collect_data(&file);
        collected_datas.push(collected);
    }

    let main_rs_path = Path::new(&manifest_dir).join("src/main.rs");
    let mut collected = collect_data(main_rs_path.to_str().unwrap());

    collected.entities.iter_mut().for_each(|entity| {
        entity.fields.push(Field {
            name: "entity".into(),
            data_type: "Entity".into(),
        })
    });

    collected.retain_unique_queries();

    debug!("{:?}", collected);

    include_files.push(generate_default_queries(&out_dir));
    generate_world_rs(&out_dir, &mut include_files, &collected);
    generate_queries(&out_dir, &mut include_files, &collected);

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
