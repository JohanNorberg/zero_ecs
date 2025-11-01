use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Token,
};

/// Represents a component in the query, which can be mutable or immutable
struct ComponentSpec {
    is_mut: bool,
    ident: Ident,
}

impl Parse for ComponentSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let is_mut = input.peek(Token![mut]);
        if is_mut {
            input.parse::<Token![mut]>()?;
        }
        let ident = input.parse::<Ident>()?;
        Ok(ComponentSpec { is_mut, ident })
    }
}

/// Input for the make_query macro
/// Format: QueryName, [mut] Component1, [mut] Component2, ...
struct MakeQueryInput {
    query_name: Ident,
    components: Vec<ComponentSpec>,
}

impl Parse for MakeQueryInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let query_name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;

        let mut components = Vec::new();
        loop {
            components.push(input.parse::<ComponentSpec>()?);
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
        }

        Ok(MakeQueryInput {
            query_name,
            components,
        })
    }
}

pub fn make_query(input: TokenStream) -> TokenStream {
    let MakeQueryInput {
        query_name,
        components,
    } = parse_macro_input!(input as MakeQueryInput);

    // Generate the tuple fields for the struct
    let fields = components.iter().map(|comp| {
        let ident = &comp.ident;
        if comp.is_mut {
            quote! { &'a mut #ident }
        } else {
            quote! { &'a #ident }
        }
    });

    let expanded = quote! {
        #[query(World)]
        struct #query_name<'a>(#(#fields),*);
    };

    expanded.into()
}
