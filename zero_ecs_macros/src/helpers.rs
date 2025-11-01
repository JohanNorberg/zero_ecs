use convert_case::{Case, Casing};
use quote::format_ident;
use syn::Ident;

pub fn format_collection_name(ident: &impl ToString) -> Ident {
    format_ident!("__{}Collection", ident.to_string())
}

pub fn format_field_name(ident: &impl ToString) -> Ident {
    let s = ident.to_string();
    let s = s.to_case(Case::Snake);
    format_ident!("__{}", s)
}
