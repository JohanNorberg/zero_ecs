use std::fs::File;
use std::io::Write;

use quote::quote;
pub fn generate_queries(path: &str) {
    let code_rs = quote! {

        trait QueryFrom<'a, T> {
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

        struct Query<'a, T> {
            a_query: AQuery<T>,
            world: &'a mut World,
        }

        struct World {}
    };

    let mut f = File::create(path).expect("Unable to create file");
    write!(f, "{}", code_rs).expect("Unable to write data to file");
}
