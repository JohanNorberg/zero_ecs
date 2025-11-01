use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Error, Fields, ItemStruct};

use crate::ecs_world_impl::StructList;

pub fn expand_world(attr: TokenStream, item: TokenStream) -> TokenStream {
    let types = syn::parse_macro_input!(attr as StructList);

    let mut types: Vec<_> = types.0.iter().collect();
    let Some(next_type) = types.pop() else {
        return item;
    };

    let local_struct = syn::parse_macro_input!(item as ItemStruct);
    let Fields::Named(local_fields) = local_struct.fields else {
        return Error::new(
            local_struct.fields.span(),
            "unnamed fields are not supported",
        )
        .to_compile_error()
        .into();
    };
    let local_fields = local_fields.named.iter();
    let attrs = local_struct.attrs;
    let generics = local_struct.generics;
    let ident = local_struct.ident;
    let vis = local_struct.vis;

    quote! {
        #[tag_world(#next_type)]
        #[expand_world(#(#types),*)]
        #(#attrs)
        *
        #vis struct #ident<#generics> {
            #(#local_fields),
            *
        }
    }
    .into()
}

pub fn tag_world(attr: TokenStream, item: TokenStream) -> TokenStream {
    let foreign_struct = syn::parse_macro_input!(attr as ItemStruct);

    let local_struct = syn::parse_macro_input!(item as ItemStruct);
    let Fields::Named(local_fields) = local_struct.fields else {
        return Error::new(
            local_struct.fields.span(),
            "unnamed fields are not supported",
        )
        .to_compile_error()
        .into();
    };
    let local_fields = local_fields.named.iter();
    let attrs = local_struct.attrs;
    let generics = local_struct.generics;
    let ident = local_struct.ident;
    let vis = local_struct.vis;

    let Fields::Named(foreign_fields) = foreign_struct.fields else {
        return Error::new(
            foreign_struct.fields.span(),
            "unnamed fields are not supported",
        )
        .to_compile_error()
        .into();
    };

    let foreign_struct_ident = foreign_struct.ident;

    let foreign_fields = foreign_fields.named.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let field_type_str = quote::ToTokens::to_token_stream(field_type).to_string();
        let field_type_str = field_type_str.replace(' ', ""); // Remove spaces if necessary
        let field_name = format_ident!(
            "__twcf__{}__{}__{}",
            foreign_struct_ident,
            field_name.clone().expect("foreign fields field clone"),
            field_type_str
        );
        quote! {
            #field_name: std::marker::PhantomData<()>
        }
    });

    quote! {
        #(#attrs)
        *
        #vis struct #ident<#generics> {
            #(#foreign_fields,)*
            #(#local_fields),
            *
        }
    }
    .into()
}
