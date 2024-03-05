mod code_collection;
mod code_generation;
mod file;
mod macros;

use quote::quote;
use std::io::Write;
use std::{env, fs, path::Path};

pub use code_collection::*;
pub use code_generation::*;
pub use file::*;
use glob::glob;

pub fn generate_ecs(source_glob: &str) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("could not get manifest dir");
    let pattern = format!("{}/{}", manifest_dir, source_glob);
    let mut include_files = vec![];

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let mut collected_data = CollectedData::default();

    for file in glob(&pattern).expect("invalid glob") {
        match file {
            Ok(path) => {
                let path_str = path.display().to_string();

                let collected = collect_data(&path_str);

                collected_data.entities.extend(collected.entities);
                collected_data.queries.extend(collected.queries);
                collected_data.systems.extend(collected.systems);
            }
            Err(e) => eprintln!("Error processing path: {}", e),
        }
    }

    collected_data.entities.iter_mut().for_each(|entity| {
        entity.fields.push(Field {
            name: "entity".into(),
            data_type: "Entity".into(),
        })
    });

    collected_data.retain_unique_queries();

    //debug!("{:?}", collected_data);

    include_files.push(generate_default_queries(&out_dir));
    generate_world_rs(&out_dir, &mut include_files, &collected_data);
    generate_queries(&out_dir, &mut include_files, &collected_data);
    generate_systems(&out_dir, &mut include_files, &collected_data);
    generate_copy_traits(&out_dir, &mut include_files, &collected_data);

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
