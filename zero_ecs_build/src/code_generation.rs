#![allow(
    unused_attributes,
    dead_code,
    unused_imports,
    unused_variables,
    unused_macro_rules,
    unused_macros,
    unused_mut
)]
use core::arch;
use itertools::Itertools;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::{env, fs, path::Path};
use syn::{Fields, Item, ItemFn, Meta, PatType, PathArguments, Type};

use crate::*;
pub fn generate_default_queries(out_dir: &str) -> String {
    let file_name = "queries.rs";

    let code_rs = quote! {

        use std::marker::PhantomData;


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

    };
    write_token_stream_to_file(out_dir, file_name, &code_rs.to_string())
}

pub fn generate_world_rs(
    out_dir: &str,
    include_files: &mut Vec<String>,
    collected: &CollectedData,
) {
    let mut world_rs = vec![];

    let mut world_fields = vec![];

    let mut entity_types = collected.entities.iter().map(|entity| {
        let entity_name = &entity.name;
        fident!(entity_name)
    });

    world_rs.push(quote! {
        #[derive(Debug, Clone)]
        enum EntityType {
            #(#entity_types),*
        }

        #[derive(Debug, Clone)]
        pub struct Entity {
            entity_type: EntityType,
            id: usize
        }
    });

    for entity in collected.entities.iter() {
        let entity_name = &entity.name;
        let field_name = fident!(singular_to_plural(&pascal_case_to_snake_case(entity_name)));
        let archetype_type = fident!(singular_to_plural(entity_name));

        world_fields.push(quote! {
            #field_name: #archetype_type,
        });

        let archetype_fields = entity.fields.iter().map(|field| {
            let field_name = format_ident!("{}", singular_to_plural(&field.name));
            let field_type = format_ident!("{}", &field.data_type);
            quote! {
                #field_name: Vec<#field_type>,
            }
        });

        world_rs.push(quote! {

            #[derive(Default, Debug)]
            struct #archetype_type {
                #(#archetype_fields)*
            }
        });
    }

    world_rs.push(quote! {
        #[derive(Default, Debug)]
        pub struct World {
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

pub fn singular_to_plural(name: &str) -> String {
    let last_char = name.chars().last().unwrap();
    if last_char == 'y' {
        return format!("{}ies", &name[0..name.len() - 1]);
    } else {
        return format!("{}s", name);
    }
}
pub fn pascal_case_to_snake_case(name: &str) -> String {
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

pub fn generate_queries(out_dir: &str, include_files: &mut Vec<String>, collected: &CollectedData) {
    let mut code_rs = vec![];

    code_rs.push(quote! {
        use zero_ecs::izip;

        pub trait QueryFrom<'a, T> {
            fn query_from(&'a mut self) -> impl Iterator<Item = T>;
        }
    });

    for query in collected.queries.iter() {
        let matching_entities: Vec<&EntityDef> = collected
            .entities
            .iter()
            .filter(|entity| {
                let mut all_fields_present = true;

                for query_field in query.mutable_fields.iter() {
                    if !entity
                        .fields
                        .iter()
                        .any(|entity_field| entity_field.data_type == *query_field)
                    {
                        all_fields_present = false;
                        break;
                    }
                }
                for query_field in query.const_fields.iter() {
                    if !entity
                        .fields
                        .iter()
                        .any(|entity_field| entity_field.data_type == *query_field)
                    {
                        all_fields_present = false;
                        break;
                    }
                }
                all_fields_present
            })
            .collect();
        let mut data_types = vec![];

        for field in query.mutable_fields.iter() {
            let field_data_type = fident!(field);
            data_types.push(quote! {
                &'a mut #field_data_type
            });
        }
        for field in query.const_fields.iter() {
            let field_data_type = fident!(field);

            data_types.push(quote! {
                &'a #field_data_type
            });
        }
        for entity in matching_entities.iter() {
            let entity_name = fident!(entity.name);

            let mut field_quotes = vec![];

            for field in query.mutable_fields.iter() {
                let field_name = fident!(singular_to_plural(
                    entity
                        .fields
                        .iter()
                        .find(|f| f.data_type == *field)
                        .unwrap()
                        .name
                        .as_str()
                ));

                field_quotes.push(quote! {
                    self.#field_name.iter_mut()
                });
            }
            for field in query.const_fields.iter() {
                let field_name = fident!(singular_to_plural(
                    entity
                        .fields
                        .iter()
                        .find(|f| f.data_type == *field)
                        .unwrap()
                        .name
                        .as_str()
                ));

                field_quotes.push(quote! {
                    self.#field_name.iter()
                });
            }

            let archetype_type = fident!(singular_to_plural(&entity.name));

            code_rs.push(quote! {
                #[allow(unused_parens)]
                impl<'a> QueryFrom<'a, (#(#data_types),*)> for #archetype_type {
                    fn query_from(&'a mut self) -> impl Iterator<Item = (#(#data_types),*)> {
                        izip!(#(#field_quotes),*)
                    }
                }
            })

            //code_rs.push(quote! {
            //    #[allowunused_parens)]
            //    impl<'a> Entity<'a, (#(#data_types),*)> for #entity_name {
            //        fn get_query_model(&'a mut self) -> (#(#data_types),*) {
            //            (#(#field_quotes),*)
            //        }
            //    }
            //});
        }
    }

    let code_rs = quote! {
        #(#code_rs)*
    };

    include_files.push(write_token_stream_to_file(
        out_dir,
        "implementations.rs",
        &code_rs.to_string(),
    ));
}
