use std::fs::{self};
use std::path::Path;
use std::process::Command;

use quote::quote;
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

fn write_token_stream_to_file(out_dir: &str, file_name: &str, code: &str) -> String {
    let dest_path = Path::new(&out_dir).join(file_name);
    fs::write(&dest_path, code.to_string())
        .expect(format!("failed to write to file: {}", file_name).as_str());
    format_file(&dest_path.to_str().unwrap());
    format!("/{}", file_name)
}
fn format_file(file_name: &str) {
    Command::new("rustfmt")
        .arg(file_name)
        .output()
        .expect("failed to execute rustfmt");
}
