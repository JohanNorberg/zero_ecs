use crate::{
    default_queries::get_default_queries,
    helpers::{format_collection_name, format_field_name},
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token,
};

pub struct StructList(pub Punctuated<Ident, Token![,]>);

impl Parse for StructList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse zero or more identifiers separated by commas.
        let list = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(StructList(list))
    }
}

pub fn ecs_world(input: TokenStream) -> TokenStream {
    // Parse the input tokens as a list of identifiers separated by commas.
    let types = syn::parse_macro_input!(input as StructList);

    let fields = types.0.iter().map(|ty| {
        let field_name = format_field_name(ty);
        // type name should be collection
        let type_name = format_collection_name(ty);
        quote! {
            #field_name: #type_name
        }
    });

    let enum_names: Vec<_> = types
        .0
        .iter()
        .map(|ty| {
            quote! {
                #ty
            }
        })
        .collect();

    let create_implementations = types.0.iter().map(|ty| {
        let collection_field_name = format_field_name(ty);
        quote! {
            impl WorldCreate<#ty> for World {
                fn create(&mut self, e: #ty) -> Entity {
                    self.#collection_field_name.create(e)
                }
            }
        }
    });

    let destroy_match_calls = types.0.iter().map(|ty| {
        let collection_field_name = format_field_name(ty);
        quote! {
            EntityType::#ty => self.#collection_field_name.destroy(e),
        }
    });

    let destroy_implementation = quote! {
        impl WorldDestroy for World {
            fn destroy(&mut self, e: Entity) {
                match e.entity_type {
                    #(#destroy_match_calls)*
                }
            }
        }
    };

    let default_queries = get_default_queries();
    // Generate the struct World with the computed fields.
    let expanded = quote! {
        #[expand_world(#(#enum_names),*)]
        #[export_tokens]
        #[derive(Default)]
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        pub struct World {
            #(#fields,)*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum EntityType {
            #(#enum_names,)*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct Entity {
            entity_type: EntityType,
            id: usize
        }

        #default_queries

        #destroy_implementation

        // creates
        #(#create_implementations)*

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
        pub trait WorldDestroy {
            fn destroy(&mut self, e: Entity);
        }
    };

    expanded.into()
}
