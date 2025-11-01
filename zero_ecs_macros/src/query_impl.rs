use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{spanned::Spanned, Error, Fields, ItemStruct, Type};

use crate::helpers::{format_collection_name, format_field_name};

#[derive(Debug)]
pub struct CollectionComponentField {
    pub collection_name: String,
    pub field_name: String,
    pub field_type: String,
}

pub fn get_collection_component_fields(
    foreign_struct: ItemStruct,
) -> Vec<CollectionComponentField> {
    let Fields::Named(foreign_fields) = foreign_struct.fields else {
        panic!(
            "Unnamed fields are not supported: {:?}",
            foreign_struct.fields
        );
    };

    let collection_component_fields: Vec<CollectionComponentField> = foreign_fields
        .named
        .iter()
        .filter_map(|field| {
            if let Some(field_name) = &field.ident {
                if field_name.to_string().starts_with("__twcf__") {
                    let field_name_str = field_name.to_string();
                    let field_name_str = field_name_str.replace("__twcf__", "");
                    let parts: Vec<_> = field_name_str.split("__").collect();
                    if parts.len() == 3 {
                        let collection_name = parts[0];
                        let field_name = parts[1];
                        let field_type = parts[2];
                        Some((
                            collection_name.to_string(),
                            field_name.to_string(),
                            field_type.to_string(),
                        ))
                    } else {
                        // print error to build
                        eprintln!("Error: invalid field name: {}", field_name_str);
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map(
            |(collection_name, field_name, field_type)| CollectionComponentField {
                collection_name,
                field_name,
                field_type,
            },
        )
        .collect();

    collection_component_fields
}

pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    let foreign_struct = syn::parse_macro_input!(attr as ItemStruct);
    let collection_component_fields = get_collection_component_fields(foreign_struct);
    let local_struct = syn::parse_macro_input!(item as ItemStruct);
    let Fields::Unnamed(local_fields) = local_struct.fields else {
        return Error::new(local_struct.fields.span(), "named fields are not supported")
            .to_compile_error()
            .into();
    };

    let local_struct_name = local_struct.ident;

    let local_fields: Vec<_> = local_fields.unnamed.iter().collect();
    let mut any_mutable_local_fields = false;
    let types_to_query: Vec<String> = local_fields
        .iter()
        .filter_map(|field| {
            // only the type name, ignore all ' and < and stuff
            // start by casting to a reference
            if let Type::Reference(ty) = &field.ty {
                if ty.mutability.is_some() {
                    any_mutable_local_fields = true;
                }

                // then get the type
                if let Type::Path(path) = &*ty.elem {
                    // then get the first segment
                    if let Some(segment) = path.path.segments.first() {
                        // then get the ident
                        let ident = &segment.ident;
                        Some(ident.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let all_collections: HashSet<_> = collection_component_fields
        .iter()
        .map(|field| &field.collection_name)
        .collect();

    // get all collections that have all of the local_field types
    let matching_collections: Vec<_> = all_collections
        .iter()
        .filter_map(|collection_name| {
            let collection_types = collection_component_fields
                .iter()
                .filter(|field| field.collection_name == **collection_name)
                .map(|field| &field.field_type)
                .collect::<HashSet<_>>();

            if types_to_query
                .iter()
                // entity is special case
                .filter(|field_name| *field_name != "Entity")
                .all(|field_name| collection_types.contains(field_name))
            {
                Some(*collection_name)
            } else {
                None
            }
        })
        .collect();

    let query_codes: Vec<_> = matching_collections
        .iter()
        .map(|&collection_name| {
            let collection_type_name = format_collection_name(collection_name);

            let collection_field_names: Vec<_> = types_to_query.iter()
                .map(|field_type| {
                    // find from collection_component_fields the one that matches field_type and the collection name. Should only be one
                    let field_name = if field_type != "Entity" { collection_component_fields
                        .iter()
                        .find(|field| &field.collection_name == collection_name && field.field_type == *field_type)
                        .unwrap_or_else(|| panic!("expect_collection_component_fields field_type: {}", field_type))
                        .field_name
                        .clone()
                    } else {
                        "entity".into()
                    };
                    quote::format_ident!("{}", field_name)
                }).collect();

            let query_code = quote! {
                impl<'a> QueryFrom<'a, #local_struct_name<'a>> for #collection_type_name {
                    fn query_from(&'a self) -> impl Iterator<Item = #local_struct_name<'a>> {
                        izip!(#(self.#collection_field_names.iter()),*)
                            .map(|(#(#collection_field_names),*)| #local_struct_name(#(#collection_field_names),*))
                    }

                    fn par_query_from(&'a self) -> impl ParallelIterator<Item = #local_struct_name<'a>> {
                        izip_par!(#(self.#collection_field_names.par_iter()),*)
                            .map(|(#(#collection_field_names),*)| #local_struct_name(#(#collection_field_names),*))
                    }

                    fn get_from(&'a self, entity: Entity) -> Option<#local_struct_name<'a>> {
                        if let Some(&Some(index)) = self.index_lookup.get(entity.id) {
                            Some(#local_struct_name(
                                #(self.#collection_field_names.get(index)?),*
                            ))
                        } else {
                            None
                        }
                    }

                    fn at(&'a self, index: usize) -> Option<#local_struct_name<'a>> {
                        Some(#local_struct_name(
                            #(self.#collection_field_names.get(index)?),*
                        ))
                    }
                }
            };

            let query_mut_code = quote! {
                impl<'a> QueryMutFrom<'a, #local_struct_name<'a>> for #collection_type_name {
                    fn query_mut_from(&'a mut self) -> impl Iterator<Item = #local_struct_name<'a>> {
                        izip!(#(self.#collection_field_names.iter_mut()),*)
                            .map(|(#(#collection_field_names),*)| #local_struct_name(#(#collection_field_names),*))
                    }

                    fn par_query_mut_from(&'a mut self) -> impl ParallelIterator<Item = #local_struct_name<'a>> {
                        izip_par!(#(self.#collection_field_names.par_iter_mut()),*)
                            .map(|(#(#collection_field_names),*)| #local_struct_name(#(#collection_field_names),*))
                    }

                    fn get_mut_from(&'a mut self, entity: Entity) -> Option<#local_struct_name<'a>> {
                        if let Some(&Some(index)) = self.index_lookup.get(entity.id) {
                            Some(#local_struct_name(
                                #(self.#collection_field_names.get_mut(index)?),*
                            ))
                        } else {
                            None
                        }
                    }

                    fn at_mut(&'a mut self, index: usize) -> Option<#local_struct_name<'a>> {
                        Some(#local_struct_name(
                            #(self.#collection_field_names.get_mut(index)?),*
                        ))
                    }
                }
            };

            let len_from_code = quote! {
                impl<'a> LenFrom<'a, #local_struct_name<'a>> for #collection_type_name {
                    fn len(&'a self) -> usize {
                        self.entity.len()
                    }
                }
            };

            if any_mutable_local_fields {
                quote! {
                    #query_mut_code
                    #len_from_code
                }
            } else {
                quote! {
                    #query_code
                    #query_mut_code
                    #len_from_code
                }
            }
        })
        .collect();

    let world_query_code = {
        let world_fields: Vec<_> = matching_collections.iter().map(format_field_name).collect();

        let query_from_body_parts: Vec<_> = matching_collections
            .iter()
            .map(|name| {

                let field_name = format_field_name(name);
                let collection_name = format_collection_name(name);

                quote! {
                    <#collection_name as QueryFrom<'a, #local_struct_name<'a>>>::query_from(& self.#field_name)
                }
            })
            .collect();

        let query_mut_from_body_parts: Vec<_> = matching_collections
            .iter()
            .map(|name| {
                let field_name = format_field_name(name);
                let collection_name = format_collection_name(name);
                quote! {
                    <#collection_name as QueryMutFrom<'a, #local_struct_name<'a>>>::query_mut_from(&mut self.#field_name)
                }
            })
            .collect();

        let par_query_from_body_parts: Vec<_> = matching_collections
            .iter()
            .map(|name| {
                let field_name = format_field_name(name);
                let collection_name = format_collection_name(name);
                quote! {
                    <#collection_name as QueryFrom<'a, #local_struct_name<'a>>>::par_query_from(&self.#field_name)
                }
            })
            .collect();
        let par_query_mut_from_body_parts: Vec<_> = matching_collections
            .iter()
            .map(|name| {
                let field_name = format_field_name(name);
                let collection_name = format_collection_name(name);
                quote! {
                    <#collection_name as QueryMutFrom<'a, #local_struct_name<'a>>>::par_query_mut_from(&mut self.#field_name)
                }
            })
            .collect();

        let get_from_body_parts: Vec<_> = matching_collections
            .iter()
            .map(|name| {
                let field_name = format_field_name(name);
                let enum_name = quote::format_ident!("{}", name);

                quote! {
                    EntityType::#enum_name => self.#field_name.get(entity)
                }
            })
            .collect();

        let get_mut_from_body_parts: Vec<_> = matching_collections
            .iter()
            .map(|name| {
                let field_name = format_field_name(name);
                let enum_name = quote::format_ident!("{}", name);

                quote! {
                    EntityType::#enum_name => self.#field_name.get_mut(entity)
                }
            })
            .collect();

        let at_parts: Vec<_> = world_fields
            .iter()
            .map(|name| {
                quote! {
                    {
                        let len = self.#name.len();
                        if index < len {
                            return self.#name.at(index);
                        }
                        index -= len;
                    }
                }
            })
            .collect();

        let at_mut_parts: Vec<_> = world_fields
            .iter()
            .map(|name| {
                quote! {
                    {
                        let len = self.#name.len();
                        if index < len {
                            return self.#name.at_mut(index);
                        }
                        index -= len;
                    }
                }
            })
            .collect();

        let len_parts: Vec<_> = world_fields
            .iter()
            .map(|name| {
                quote! {
                    self.#name.len()
                }
            })
            .collect();

        let query_code = quote! {

            impl<'a> QueryFrom<'a, #local_struct_name<'a>> for World {
                fn query_from(&'a self) -> impl Iterator<Item = #local_struct_name<'a>> {
                    chain!(
                        #(#query_from_body_parts),*
                    )
                }

                fn par_query_from(&'a self) -> impl ParallelIterator<Item = #local_struct_name<'a>> {
                    chain_par!(
                        #(#par_query_from_body_parts),*
                    )
                }

                fn get_from(&'a self, entity: Entity) -> Option<#local_struct_name<'a>> {
                    match entity.entity_type {
                        #(#get_from_body_parts,)*
                        _ => None,
                    }
                }

                fn at(&'a self, index: usize) -> Option<#local_struct_name<'a>> {
                    let mut index = index;
                    #(#at_parts)*
                    None
                }
            }
        };

        let query_mut_code = quote! {
            impl<'a> QueryMutFrom<'a, #local_struct_name<'a>> for World {
                fn query_mut_from(&'a mut self) -> impl Iterator<Item = #local_struct_name<'a>> {
                    chain!(
                        #(#query_mut_from_body_parts),*
                    )
                }

                fn par_query_mut_from(&'a mut self) -> impl ParallelIterator<Item = #local_struct_name<'a>> {
                    chain_par!(
                        #(#par_query_mut_from_body_parts),*
                    )
                }

                fn get_mut_from(&'a mut self, entity: Entity) -> Option<#local_struct_name<'a>> {
                    match entity.entity_type {
                        #(#get_mut_from_body_parts,)*
                        _ => None,
                    }
                }

                fn at_mut(&'a mut self, index: usize) -> Option<#local_struct_name<'a>> {
                    let mut index = index;
                    #(#at_mut_parts)*
                    None
                }
            }
        };

        let sum = if len_parts.is_empty() {
            quote! {
                0
            }
        } else {
            quote! {
                sum!(
                    #(#len_parts),*
                )
            }
        };

        let len_from_code = quote! {
            impl<'a> LenFrom<'a, #local_struct_name<'a>> for World {
                fn len(&'a self) -> usize {
                    #sum
                }
            }
        };

        if any_mutable_local_fields {
            quote! {
                #query_mut_code
                #len_from_code
            }
        } else {
            quote! {
                #query_code
                #query_mut_code
                #len_from_code
            }
        }
    };

    quote! {

        #[derive(From, Into)]
        struct #local_struct_name<'a> (#(#local_fields),*);

        #(#query_codes)*

        #world_query_code
    }
    .into()
}
