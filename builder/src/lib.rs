mod builder_struct;
mod fields;

use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};
use builder_struct::*;
use fields::parse_fields;

#[allow(dead_code)]
type Option<T> = std::option::Option<T>;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let fields = parse_fields(&input.data);

    let builder_struct_name = format_ident!("{}Builder", name);
    let builder_struct = get_builder_struct(&fields, &builder_struct_name);
    let impl_builder = impl_builder(&fields, &builder_struct_name, &name);
    let init_builder = init_builder_struct(&fields, &builder_struct_name);

    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder_struct_name {
                #init_builder
            }
        }
        #builder_struct
        #impl_builder
    };
    // For debugging:
    // eprintln!("TOKENS: {}", expanded);
    expanded.into()
}
