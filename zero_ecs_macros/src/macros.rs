extern crate proc_macro;

use macro_magic::import_tokens_attr;
use proc_macro::TokenStream;

mod default_queries;
mod ecs_world_impl;
mod entity_impl;
mod helpers;
mod make_query_impl;
mod query_impl;
mod system_for_each_impl;
mod system_impl;
mod world_impl;

#[proc_macro_attribute]
pub fn entity(attr: TokenStream, item: TokenStream) -> TokenStream {
    entity_impl::entity(attr, item)
}

#[proc_macro]
pub fn ecs_world(input: TokenStream) -> TokenStream {
    ecs_world_impl::ecs_world(input)
}

#[proc_macro_derive(World, attributes(tag_collection_field))]
pub fn world_tag_collection_field(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn expand_world(attr: TokenStream, item: TokenStream) -> TokenStream {
    world_impl::expand_world(attr, item)
}

#[import_tokens_attr(zero_ecs::macro_magic)]
#[proc_macro_attribute]
pub fn tag_world(attr: TokenStream, item: TokenStream) -> TokenStream {
    world_impl::tag_world(attr, item)
}

#[import_tokens_attr(zero_ecs::macro_magic)]
#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    query_impl::query(attr, item)
}

#[import_tokens_attr(zero_ecs::macro_magic)]
#[proc_macro_attribute]
pub fn system_for_each(attr: TokenStream, item: TokenStream) -> TokenStream {
    system_for_each_impl::system_for_each(attr, item)
}

#[import_tokens_attr(zero_ecs::macro_magic)]
#[proc_macro_attribute]
pub fn system(attr: TokenStream, item: TokenStream) -> TokenStream {
    system_impl::system(attr, item)
}

#[proc_macro]
pub fn make_query(input: TokenStream) -> TokenStream {
    make_query_impl::make_query(input)
}
