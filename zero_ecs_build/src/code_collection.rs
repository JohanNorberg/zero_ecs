use std::{
    collections::HashSet,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
};

use quote::ToTokens;
use syn::{Fields, Item, ItemFn, Meta, PathArguments, Type};

use crate::debug;

#[derive(Debug)]
pub struct EntityDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Default, Clone)]
pub struct SystemDefParamReference {
    pub name: String,
    pub ty: String,
    pub mutable: bool,
}

#[derive(Debug)]
pub enum SystemDefParam {
    Query(String),
    Reference(SystemDefParamReference),
}

#[derive(Debug, Default)]
pub struct SystemDef {
    pub name: String,
    pub group: String,
    pub params: Vec<SystemDefParam>,
}

#[derive(Debug, Default)]
pub struct CollectedData {
    pub entities: Vec<EntityDef>,
    pub queries: Vec<Query>,
    pub systems: Vec<SystemDef>,
}
#[derive(Debug, Clone)]
pub struct Query {
    pub mutable_fields: Vec<String>,
    pub const_fields: Vec<String>,
}

impl PartialEq for Query {
    fn eq(&self, other: &Self) -> bool {
        self.mutable_fields == other.mutable_fields && self.const_fields == other.const_fields
    }
}

impl Eq for Query {}

impl Hash for Query {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.mutable_fields.hash(state);
        self.const_fields.hash(state);
    }
}

impl CollectedData {
    pub fn retain_unique_queries(&mut self) {
        let mut seen = HashSet::new();
        self.queries.retain(|query| {
            let mut hasher = DefaultHasher::new();
            query.hash(&mut hasher);
            let hash = hasher.finish();
            seen.insert(hash)
        });
    }
}

#[allow(clippy::single_match)]
pub fn collect_data(path: &str) -> CollectedData {
    let mut entities = vec![];
    let mut queries = vec![];
    let mut systems = vec![];

    let content =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Unable to read file {}", path));

    let parsed_file =
        syn::parse_file(&content).unwrap_or_else(|_| panic!("Unable to parse file {}", path));

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
                                    let field = field.split(':').collect::<Vec<&str>>();

                                    let field_name = field[0].trim().to_string();

                                    // split on space and take last element
                                    let field_name =
                                        field_name.split(' ').last().unwrap().to_string();

                                    fields.push(Field {
                                        name: field_name,
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
                let function_name = item_fn.sig.ident.to_string();
                let system_group = is_system(&item_fn);
                if let Some(system_group) = system_group {
                    let mut system_def = SystemDef {
                        group: system_group,
                        name: function_name.to_string(),
                        params: vec![],
                    };

                    //debug!("system: {}, group: {}", function_name, &system_def.group);
                    // get all function parameters
                    for input in &item_fn.sig.inputs {
                        match input {
                            syn::FnArg::Receiver(_) => {}
                            syn::FnArg::Typed(pat_type) => {
                                let param_name = pat_type.pat.to_token_stream().to_string();
                                let ty = pat_type.ty.clone();
                                match *ty {
                                    Type::Path(typed_path) => {
                                        for segment in typed_path.path.segments.iter() {
                                            let name = segment.ident.to_string();

                                            //debug!("param: {}: {}", param_name, name);

                                            if name == "Query" {
                                                match &segment.arguments {
                                                    PathArguments::AngleBracketed(arguments) => {
                                                        if let Some(arg) =
                                                            &arguments.args.iter().next()
                                                        {
                                                            match arg {
                                                                syn::GenericArgument::Type(ty) => {
                                                                    if let Some(query) =
                                                                        collect_query(ty)
                                                                    {
                                                                        queries.push(query);
                                                                        system_def.params.push(
                                                                            SystemDefParam::Query(
                                                                                param_name
                                                                                    .to_string(),
                                                                            ),
                                                                        );
                                                                    }
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            } else {
                                                panic!(
                                                    "unsupported type: {} for param: {}",
                                                    name, param_name
                                                );
                                            }
                                        }
                                    }
                                    Type::Reference(type_reference) => {
                                        let mutable = type_reference.mutability.is_some();
                                        let elem_str =
                                            type_reference.elem.to_token_stream().to_string();
                                        // debug!(
                                        //     "reference: {}, of type: {}, mutable: {}",
                                        //     param_name, elem_str, mutable
                                        // );

                                        system_def.params.push(SystemDefParam::Reference(
                                            SystemDefParamReference {
                                                name: param_name.to_string(),
                                                ty: elem_str,
                                                mutable,
                                            },
                                        ));
                                    }
                                    _ => {
                                        debug!("not typed: ");
                                    }
                                }
                            }
                        }
                    }

                    systems.push(system_def);
                }
            }
            _ => {}
        }
    }

    CollectedData {
        entities,
        queries,
        systems,
    }
}

fn collect_query(ty: &Type) -> Option<Query> {
    // handle reference, they have one value
    // "& mut Position"
    // "& Velocity"
    // also handle tuples, they have multiple, examples
    // (& mut Position , & Velocity)

    let query = match ty {
        Type::Reference(type_reference) => {
            let mut mutable_fields = vec![];
            let mut const_fields = vec![];
            let ty = type_reference.elem.clone();
            match *ty {
                Type::Path(type_path) => {
                    if type_reference.mutability.is_some() {
                        mutable_fields.push(type_path.to_token_stream().to_string());
                    } else {
                        const_fields.push(type_path.to_token_stream().to_string());
                    }
                }
                _ => {}
            }
            Some(Query {
                mutable_fields,
                const_fields,
            })
        }
        Type::Tuple(type_tuple) => {
            let mut mutable_fields = vec![];
            let mut const_fields = vec![];
            for elem in &type_tuple.elems {
                match elem {
                    Type::Reference(type_reference) => {
                        if type_reference.mutability.is_some() {
                            mutable_fields.push(type_reference.elem.to_token_stream().to_string());
                        } else {
                            const_fields.push(type_reference.elem.to_token_stream().to_string());
                        }
                    }
                    _ => {}
                }
            }
            Some(Query {
                mutable_fields,
                const_fields,
            })
        }
        _ => None,
    };

    if let Some(query) = query {
        if query.mutable_fields.is_empty() && query.const_fields.is_empty() {
            None
        } else {
            Some(query)
        }
    } else {
        None
    }
}

//only used for debugging
//pub fn _print_type(ty: &Type) {
//    match ty {
//        // Double dereference to get the `Type` from `&Box<Type>`
//        Type::Array(_) => debug!("Array"),
//        Type::BareFn(_) => debug!("BareFn"),
//        Type::Group(_) => debug!("Group"),
//        Type::ImplTrait(_) => debug!("ImplTrait"),
//        Type::Infer(_) => debug!("Infer"),
//        Type::Macro(_) => debug!("Macro"),
//        Type::Never(_) => debug!("Never"),
//        Type::Paren(_) => debug!("Paren"),
//        Type::Path(_) => debug!("Path"),
//        Type::Ptr(_) => debug!("Ptr"),
//        Type::Reference(_) => debug!("Reference"),
//        Type::Slice(_) => debug!("Slice"),
//        Type::TraitObject(_) => debug!("TraitObject"),
//        Type::Tuple(_) => debug!("Tuple"),
//        Type::Verbatim(_) => debug!("Verbatim"),
//        _ => {} // Add new variants here as needed
//    }
//}

fn is_system(item_fn: &ItemFn) -> Option<String> {
    for attr in &item_fn.attrs {
        match &attr.meta {
            Meta::Path(path) => {
                if path.is_ident("system") {
                    return Some("main".into());
                }
            }
            Meta::List(list) => {
                if list.path.is_ident("system") {
                    let tokens_str = list.tokens.to_string();

                    let mut kvp = tokens_str.split('=');

                    if let Some(key) = kvp.next() {
                        if let Some(value) = kvp.next() {
                            let key = key.trim();
                            if key == "group" {
                                //                                debug!("group: {}", value);

                                return Some(value.trim().into());
                            }
                        }
                    }

                    return Some("main".into());
                }
            }
            Meta::NameValue(name_value) => {
                if name_value.path.is_ident("system") {
                    debug!("name value: {}", name_value.to_token_stream());
                    return Some("main".into());
                }
            }
        }
    }
    None
}
