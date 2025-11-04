use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Error, Fields, ItemStruct};

use crate::helpers::format_collection_name;

pub fn entity(_: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = syn::parse_macro_input!(input as ItemStruct);

    let Fields::Named(fields) = input_struct.clone().fields else {
        return Error::new(
            input_struct.fields.span(),
            "unnamed fields are not supported",
        )
        .to_compile_error()
        .into();
    };

    let collection_fields: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let field_type = &field.ty;

            quote! {
                #field_name: Vec<#field_type>
            }
        })
        .collect();

    let ident = &input_struct.ident;
    let vis = &input_struct.vis;
    let collection_name = format_collection_name(ident);

    let create_push_calls: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            quote! {
                self.#field_name.push(e.#field_name);
            }
        })
        .collect();
    let destroy_swap_and_pops = fields.named.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            self.#field_name.swap(old_index, last_index);
            self.#field_name.pop();
        }
    });

    quote! {
        #[export_tokens]
        #input_struct

        #[export_tokens]
        #[derive(Default)]
        #vis struct #collection_name {
           #( pub #collection_fields, )*
           pub entity: Vec<Entity>,
           pub next_id: usize,
           pub index_lookup: Vec<Option<usize>>,
        }

        impl WorldCreate<#ident> for #collection_name {
            fn create(&mut self, e: #ident) -> Entity {
                self.index_lookup.push(Some(self.entity.len()));
                let entity = Entity {
                    entity_type: EntityType::#ident,
                    id: self.next_id,
                };
                self.entity.push(entity);

                #(#create_push_calls)*

                self.next_id += 1;
                entity
            }
        }


        impl WorldDestroy for #collection_name {
            fn destroy(&mut self, e: Entity) {
                if let Some(&Some(old_index)) = self.index_lookup.get(e.id) {
                    self.index_lookup[e.id] = None;
                    let last_index = self.entity.len() - 1;
                    let last_entity = self.entity[last_index];
                    let is_now_last = old_index == last_index;

                    #(#destroy_swap_and_pops)*

                    self.entity.swap(old_index, last_index);
                    self.entity.pop();

                    if !is_now_last {
                        self.index_lookup[last_entity.id] = Some(old_index);
                    }
                }
            }
        }

        impl #collection_name {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn len(&self) -> usize {
                self.entity.len()
            }

            pub fn query_mut<'a, T: 'a>(&'a mut self) -> impl Iterator<Item = T> + 'a
            where
                #collection_name: QueryMutFrom<'a, T>,
                T: 'a + Send,
            {
                QueryMutFrom::<T>::query_mut_from(self)
            }
            fn par_query_mut<'a, T: 'a>(&'a mut self) -> impl ParallelIterator<Item = T> + 'a
            where
                #collection_name: QueryMutFrom<'a, T>,
                T: 'a + Send,
            {
                QueryMutFrom::<T>::par_query_mut_from(self)
            }
            pub fn get_mut<'a, T: 'a>(&'a mut self, entity: Entity) -> Option<T>
            where
                #collection_name: QueryMutFrom<'a, T>,
                T: 'a + Send,
            {
                QueryMutFrom::<T>::get_mut_from(self, entity)
            }

            pub fn query<'a, T: 'a>(&'a self) -> impl Iterator<Item = T> + 'a
            where
                #collection_name: QueryFrom<'a, T>,
                T: 'a + Send,
            {
                QueryFrom::<T>::query_from(self)
            }
            pub fn par_query<'a, T: 'a>(&'a self) -> impl ParallelIterator<Item = T> + 'a
            where
                #collection_name: QueryFrom<'a, T>,
                T: 'a + Send,
            {
                QueryFrom::<T>::par_query_from(self)
            }
            pub fn get<'a, T: 'a>(&'a self, entity: Entity) -> Option<T>
            where
                #collection_name: QueryFrom<'a, T>,
                T: 'a + Send,
            {
                QueryFrom::<T>::get_from(self, entity)
            }
        }
    }
    .into()
}
