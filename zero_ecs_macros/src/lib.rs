extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn just_for_test(_: TokenStream, input: TokenStream) -> TokenStream {
    let output = format!(
        "fn my_test() {{ println!(\"Macro just_for_test invoked\"); {} }}",
        input.to_string()
    );
    output.parse().unwrap()
}
