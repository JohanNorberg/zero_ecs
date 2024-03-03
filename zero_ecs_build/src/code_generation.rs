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
use std::collections::HashMap;
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


        #[derive(Default, Debug, Copy, Clone)]
        pub struct Query<T> {
            phantom: PhantomData<T>,
        }

        impl<T> Query<T> {
            fn new() -> Query<T> {
                Query {
                    phantom: PhantomData,
                }
            }
        }

        impl<'a, T: 'a + Send> Query<T>
        {
            pub fn iter(&self, world: &'a World) -> impl Iterator<Item = T> + 'a
            where
                World: QueryFrom<'a, T>,
            {
                world.query_from()
            }
        }
        impl<'a, T: 'a + Send> Query<T>
        {
            pub fn par_iter(&self, world: &'a World) -> impl ParallelIterator<Item = T> + 'a
            where
                World: QueryFrom<'a, T>,
            {
                world.par_query_from()
            }
        }
        impl<'a, T: 'a + Send> Query<T> {
            pub fn iter_mut(&self, world: &'a mut World) -> impl Iterator<Item = T> + 'a
            where
                World: QueryMutFrom<'a, T>,
            {
                world.query_mut_from()
            }
        }
        impl<'a, T: 'a + Send> Query<T>
        {
            pub fn par_iter_mut(&self, world: &'a mut World) -> impl ParallelIterator<Item = T> + 'a
            where
                World: QueryMutFrom<'a, T>,
            {
                world.par_query_mut_from()
            }
        }
        impl<'a, T: 'a + Send> Query<T> {
            pub fn get(&self, world: &'a World, entity: Entity) -> Option<T>
            where
                World: QueryFrom<'a, T>,
            {
                world.get_from(entity)
            }
        }
        impl<'a, T: 'a + Send> Query<T> {
            pub fn get_mut(&self, world: &'a mut World, entity: Entity) -> Option<T>
            where
                World: QueryMutFrom<'a, T>,
            {
                world.get_mut_from(entity)
            }
        }

        pub struct WithQueryMut<'a, T> {
            query: Query<T>,
            world: &'a mut World,
        }
        pub struct WithQuery<'a, T> {
            query: Query<T>,
            world: &'a World,
        }

        impl<'a, T> WithQueryMut<'a, T>
            where World: QueryMutFrom<'a, T>,
                T: 'a + Send,
        {
            pub fn iter_mut(&'a mut self) -> impl Iterator<Item = T> + 'a {
                self.query.iter_mut(self.world)
            }
            pub fn par_iter_mut(&'a mut self) -> impl ParallelIterator<Item = T> + 'a {
                self.query.par_iter_mut(self.world)
            }
            pub fn get_mut(&'a mut self, entity: Entity) -> Option<T> {
                self.query.get_mut(self.world, entity)
            }
        }
        impl<'a, T> WithQuery<'a, T>
            where World: QueryFrom<'a, T>,
                T: 'a + Send,
        {
            pub fn iter(&'a self) -> impl Iterator<Item = T> + 'a {
                self.query.iter(self.world)
            }
            pub fn par_iter(&'a self) -> impl ParallelIterator<Item = T> + 'a {
                self.query.par_iter(self.world)
            }
            pub fn get(&'a self, entity: Entity) -> Option<T> {
                self.query.get(self.world, entity)
            }
        }

        #[allow(dead_code)]
        impl World {
            pub fn with_query_mut<'a, T: 'a + Send>(&'a mut self, query: Query<T>) -> WithQueryMut<'a, T>
            where
                World: QueryMutFrom<'a, T>,
            {
                WithQueryMut {
                    query: query,
                    world: self,
                }
            }
        }
        #[allow(dead_code)]
        impl World {
            pub fn with_query<'a, T: 'a + Send>(&'a self, query: Query<T>) -> WithQuery<'a, T>
            where
                World: QueryFrom<'a, T>,
            {
                WithQuery {
                    query: query,
                    world: self,
                }
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
        use zero_ecs::*;
        #[derive(Debug, Clone, Copy)]
        enum EntityType {
            #(#entity_types),*
        }

        #[derive(Debug, Clone, Copy)]
        pub struct Entity {
            entity_type: EntityType,
            id: usize
        }
        #[allow(dead_code)]
        impl World {
            pub fn query_mut<'a, T: 'a + Send>(&'a mut self) -> impl Iterator<Item = T> + 'a
            where
                World: QueryMutFrom<'a, T>,
            {
                QueryMutFrom::<T>::query_mut_from(self)
            }
            pub fn par_query_mut<'a, T: 'a + Send>(&'a mut self) -> impl ParallelIterator<Item = T> + 'a
            where
                World: QueryMutFrom<'a, T>,
            {
                QueryMutFrom::<T>::par_query_mut_from(self)
            }

            pub fn get_mut<'a, T: 'a + Send>(&'a mut self, entity: Entity) -> Option<T>
            where
                World: QueryMutFrom<'a, T>,
            {
                QueryMutFrom::<T>::get_mut_from(self, entity)
            }
        }
        #[allow(dead_code)]
        impl World {
            pub fn query<'a, T: 'a + Send>(&'a self) -> impl Iterator<Item = T> + 'a
            where
                World: QueryFrom<'a, T>,
            {
                QueryFrom::<T>::query_from(self)
            }
            pub fn par_query<'a, T: 'a + Send>(&'a self) -> impl ParallelIterator<Item = T> + 'a
            where
                World: QueryFrom<'a, T>,
            {
                QueryFrom::<T>::par_query_from(self)
            }
            pub fn get<'a, T: 'a + Send>(&'a self, entity: Entity) -> Option<T>
            where
                World: QueryFrom<'a, T>,
            {
                QueryFrom::<T>::get_from(self, entity)
            }
        }
        pub trait WorldCreate<T> {
            fn create(&mut self, e: T) -> Entity;
        }
    });

    let mut match_destroy_rs = vec![];

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
                next_id: usize,
                index_lookup: Vec<Option<usize>>,
            }
        });

        world_rs.push(quote! {
            #[allow(dead_code)]
            impl #archetype_type {
                fn query_mut<'a, T: 'a>(&'a mut self) -> impl Iterator<Item = T> + 'a
                where
                    #archetype_type: QueryMutFrom<'a, T>,
                    T: 'a + Send,
                {
                    QueryMutFrom::<T>::query_mut_from(self)
                }
                fn par_query_mut<'a, T: 'a>(&'a mut self) -> impl ParallelIterator<Item = T> + 'a
                where
                    #archetype_type: QueryMutFrom<'a, T>,
                    T: 'a + Send,
                {
                    QueryMutFrom::<T>::par_query_mut_from(self)
                }
                fn get_mut<'a, T: 'a>(&'a mut self, entity: Entity) -> Option<T>
                where
                    #archetype_type: QueryMutFrom<'a, T>,
                    T: 'a + Send,
                {
                    QueryMutFrom::<T>::get_mut_from(self, entity)
                }
            }
        });
        world_rs.push(quote! {
            #[allow(dead_code)]
            impl #archetype_type {
                fn query<'a, T: 'a>(&'a self) -> impl Iterator<Item = T> + 'a
                where
                    #archetype_type: QueryFrom<'a, T>,
                    T: 'a + Send,
                {
                    QueryFrom::<T>::query_from(self)
                }
                fn par_query<'a, T: 'a>(&'a self) -> impl ParallelIterator<Item = T> + 'a
                where
                    #archetype_type: QueryFrom<'a, T>,
                    T: 'a + Send,
                {
                    QueryFrom::<T>::par_query_from(self)
                }
                fn get<'a, T: 'a>(&'a self, entity: Entity) -> Option<T>
                where
                    #archetype_type: QueryFrom<'a, T>,
                    T: 'a + Send,
                {
                    QueryFrom::<T>::get_from(self, entity)
                }
            }
        });

        let push_lines = entity
            .fields
            .iter()
            .filter(|e| e.data_type != "Entity")
            .map(|field| {
                let component_field_name = format_ident!("{}", singular_to_plural(&field.name));
                let component_name = fident!(&field.name);

                quote! {
                    self.#field_name.#component_field_name.push(e.#component_name);
                }
            });

        let entity_name = fident!(entity_name);

        world_rs.push(quote! {
            impl WorldCreate<#entity_name> for World {
                fn create(&mut self, e: #entity_name) -> Entity {
                    self.#field_name.index_lookup.push(Some(self.#field_name.entities.len()));
                    let entity = Entity {
                        entity_type: EntityType::#entity_name,
                        id: self.#field_name.next_id,
                    };
                    self.#field_name.entities.push(entity);
                    #(#push_lines)*
                    self.#field_name.next_id += 1;
                    entity
                }
            }
        });

        let pop_and_drop_code = entity.fields.iter().map(|field| {
            let field_name = format_ident!("{}", singular_to_plural(&field.name));
            let component_field_name = format_ident!("{}", singular_to_plural(&field.name));
            quote! {
                self.#component_field_name.swap(old_index, last_index);
                self.#component_field_name.pop();
            }
        });

        let pop_and_drop_code_copy = pop_and_drop_code.clone();

        world_rs.push(quote! {
            #[allow(dead_code)]
            impl #archetype_type {
                fn destroy(&mut self, entity: Entity) {
                    if let Some(&Some(old_index)) = self.index_lookup.get(entity.id) {
                        self.index_lookup[entity.id] = None;

                        let last_index = self.entities.len() - 1;

                        if old_index != last_index {
                            let last_entity = self.entities[last_index];

                            #(#pop_and_drop_code)*

                            self.index_lookup[last_entity.id] = Some(old_index);
                        } else {
                            #(#pop_and_drop_code_copy)*
                        }
                    }
                }
            }
        });

        match_destroy_rs.push(quote! {
            EntityType::#entity_name => self.#field_name.destroy(entity),
        });
    }

    world_rs.push(quote! {
        #[allow(dead_code)]
        impl World {
            fn destroy(&mut self, entity: Entity) {
                match entity.entity_type {
                    #(#match_destroy_rs)*
                }
            }
        }
    });

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
        //use zero_ecs::ParallelIterator;
        use zero_ecs::izip;
        use zero_ecs::chain;

        pub trait QueryFrom<'a, T>
        where
            T: 'a + Send
        {
            fn query_from(&'a self) -> impl Iterator<Item = T>;
            fn par_query_from(&'a self) -> impl ParallelIterator<Item = T>;
            fn get_from(&'a self, entity: Entity) -> Option<T>;
        }
        pub trait QueryMutFrom<'a, T>
        where
            T: 'a + Send
        {
            fn query_mut_from(&'a mut self) -> impl Iterator<Item = T>;
            fn par_query_mut_from(&'a mut self) -> impl ParallelIterator<Item = T>;
            fn get_mut_from(&'a mut self, entity: Entity) -> Option<T>;
        }
    });

    for query in collected.queries.iter() {
        let mutable = query.mutable_fields.iter().count() > 0;
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

        let mut match_get_rs = vec![];

        for entity in matching_entities.iter() {
            let entity_name = fident!(entity.name);

            let mut field_quotes = vec![];
            let mut par_field_quotes = vec![];
            let mut get_quotes = vec![];

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
                par_field_quotes.push(quote! {
                    self.#field_name.par_iter_mut()
                });
                get_quotes.push(quote! {
                    self.#field_name.get_mut(index)?
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
                par_field_quotes.push(quote! {
                    self.#field_name.par_iter()
                });
                get_quotes.push(quote! {
                    self.#field_name.get(index)?
                });
            }

            let archetype_type = fident!(singular_to_plural(&entity.name));
            let archetype_field_name =
                fident!(singular_to_plural(&pascal_case_to_snake_case(&entity.name)));

            if mutable {
                code_rs.push(quote! {
                    #[allow(unused_parens)]
                    impl<'a> QueryMutFrom<'a, (#(#data_types),*)> for #archetype_type {
                        fn query_mut_from(&'a mut self) -> impl Iterator<Item = (#(#data_types),*)> {
                            izip!(#(#field_quotes),*)
                        }
                        fn par_query_mut_from(&'a mut self) -> impl ParallelIterator<Item = (#(#data_types),*)> {
                            izip_par!(#(#par_field_quotes),*)
                        }
                        fn get_mut_from(&'a mut self, entity: Entity) -> Option<(#(#data_types),*)> {
                            if let Some(&Some(index)) = self.index_lookup.get(entity.id) {
                                Some((#(#get_quotes),*))
                            } else {
                                None
                            }
                        }
                    }
                });
                match_get_rs.push(quote! {
                    EntityType::#entity_name => self.#archetype_field_name.get_mut_from(entity),
                });
            } else {
                code_rs.push(quote! {
                    #[allow(unused_parens)]
                    impl<'a> QueryFrom<'a, (#(#data_types),*)> for #archetype_type {
                        fn query_from(&'a self) -> impl Iterator<Item = (#(#data_types),*)> {
                            izip!(#(#field_quotes),*)
                        }
                        fn par_query_from(&'a self) -> impl ParallelIterator<Item = (#(#data_types),*)> {
                            izip_par!(#(#par_field_quotes),*)
                        }
                        fn get_from(&'a self, entity: Entity) -> Option<(#(#data_types),*)> {
                            if let Some(&Some(index)) = self.index_lookup.get(entity.id) {
                                Some((#(#get_quotes),*))
                            } else {
                                None
                            }
                        }
                    }
                });
                match_get_rs.push(quote! {
                    EntityType::#entity_name => self.#archetype_field_name.get_from(entity),
                });
            }
        }

        if mutable {
            let chain_args: Vec<_> = matching_entities
                .iter()
                .map(|entity| {
                    let property_name = format_ident!(
                        "{}",
                        singular_to_plural(&pascal_case_to_snake_case(&entity.name))
                    );
                    quote! { self.#property_name.query_mut() }
                })
                .collect();
            let par_chain_args: Vec<_> = matching_entities
                .iter()
                .map(|entity| {
                    let property_name = format_ident!(
                        "{}",
                        singular_to_plural(&pascal_case_to_snake_case(&entity.name))
                    );
                    quote! { self.#property_name.par_query_mut() }
                })
                .collect();
            code_rs.push(quote! {
                #[allow(unused_parens)]
                impl<'a> QueryMutFrom<'a, (#(#data_types),*)> for World {
                    fn query_mut_from(&'a mut self) -> impl Iterator<Item = (#(#data_types),*)> {
                        chain!(#(#chain_args),*)
                    }
                    fn par_query_mut_from(&'a mut self) -> impl ParallelIterator<Item = (#(#data_types),*)> {
                        chain_par!(#(#par_chain_args),*)
                    }
                    #[allow(unreachable_patterns)]
                    fn get_mut_from(&'a mut self, entity: Entity) -> Option<(#(#data_types),*)> {
                        match entity.entity_type {
                            #(#match_get_rs)*
                            _ => None
                        }
                    }
                }
            })
        } else {
            let chain_args: Vec<_> = matching_entities
                .iter()
                .map(|entity| {
                    let property_name = format_ident!(
                        "{}",
                        singular_to_plural(&pascal_case_to_snake_case(&entity.name))
                    );
                    quote! { self.#property_name.query() }
                })
                .collect();
            let par_chain_args: Vec<_> = matching_entities
                .iter()
                .map(|entity| {
                    let property_name = format_ident!(
                        "{}",
                        singular_to_plural(&pascal_case_to_snake_case(&entity.name))
                    );
                    quote! { self.#property_name.par_query() }
                })
                .collect();
            code_rs.push(quote! {
                #[allow(unused_parens)]
                impl<'a> QueryFrom<'a, (#(#data_types),*)> for World {
                    fn query_from(&'a self) -> impl Iterator<Item = (#(#data_types),*)> {
                        chain!(#(#chain_args),*)
                    }
                    fn par_query_from(&'a self) -> impl ParallelIterator<Item = (#(#data_types),*)> {
                        chain_par!(#(#par_chain_args),*)
                    }
                    #[allow(unreachable_patterns)]
                    fn get_from(&'a self, entity: Entity) -> Option<(#(#data_types),*)> {
                        match entity.entity_type {
                            #(#match_get_rs)*
                            _ => None
                        }
                    }
                }
            })
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
pub fn generate_systems(out_dir: &str, include_files: &mut Vec<String>, collected: &CollectedData) {
    let mut code_rs = vec![];

    // distinct groups
    let groups: Vec<_> = collected
        .systems
        .iter()
        .map(|s| &s.group)
        .unique()
        .collect();

    //debug!("{:?}", groups);

    for group in groups.iter() {
        let mut calls = vec![];

        let mut call_params: HashMap<(String, String), SystemDefParamReference> = HashMap::new();

        for system in collected
            .systems
            .iter()
            .filter(|s| &s.group == *group)
            .sorted_by(|a, b| a.name.cmp(&b.name))
        {
            let mut params_rs = vec![];

            for param in system.params.iter() {
                match param {
                    SystemDefParam::Query(name) => {
                        params_rs.push(quote! {
                            Query::new(),
                        });
                    }
                    SystemDefParam::Reference(reference) => {
                        let name = fident!(reference.name);

                        params_rs.push(quote! {
                            #name,
                        });

                        let key = (reference.name.clone(), reference.ty.clone());
                        let item = reference.clone();
                        call_params
                            .entry(key)
                            .and_modify(|e| {
                                if reference.mutable {
                                    e.mutable = true;
                                }
                            })
                            .or_insert(item);
                    }
                }
            }

            let system_function_name = fident!(&system.name);

            calls.push(quote! {
                #system_function_name(#(#params_rs)*);
            })
        }

        let function_name = format_ident!("systems_{}", group);

        // get values of call_params, ignoring the key
        let call_params: Vec<_> = call_params.iter().map(|(_, v)| v).collect();

        // order call_params by name
        let call_params = call_params
            .iter()
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .collect::<Vec<_>>();

        let call_params_rs = call_params.iter().map(|r| {
            let name = fident!(r.name);
            let ty = fident!(r.ty);

            if r.mutable {
                quote! {
                   #name: &mut #ty
                }
            } else {
                quote! {

                    #name: &#ty
                }
            }
        });

        code_rs.push(quote! {
            #[allow(private_interfaces)]
            pub fn #function_name(#(#call_params_rs),*) {
                #(#calls)*
            }
        })
    }

    let code_rs = quote! {
        #(#code_rs)*
    };

    include_files.push(write_token_stream_to_file(
        out_dir,
        "systems.rs",
        &code_rs.to_string(),
    ));
}
