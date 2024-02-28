use quote::quote;

use crate::*;
pub fn generate_queries(out_dir: &str) -> String {
    let file_name = "queries.rs";

    let code_rs = quote! {

        use std::marker::PhantomData;

        pub trait QueryFrom<'a, T> {
            fn query_from(&'a mut self) -> impl Iterator<Item = T>;
        }
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

        pub struct World {}
    };
    write_token_stream_to_file(out_dir, file_name, &code_rs.to_string())
}
