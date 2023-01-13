mod fields;

use proc_macro::TokenStream;

use fields::*;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let fields = parse_fields(&input.data);

    let debug_impl = impl_debug(fields, &name);
    let tokens = quote! {
        #debug_impl
    };
    // eprintln!("Tokens: {}", tokens);
    tokens.into()
}

fn impl_debug(fields: Vec<FieldInfo>, name: &Ident) -> proc_macro2::TokenStream {
    let struct_name = format!("{}", name);

    let recurse = fields.iter().map(|f| {
        if let Some(name) = f.name {
            let name_string = format!("{}", name);
            match &f.debug_attr {
                Some((_i, str_lit)) => {
                    let s = str_lit.value();
                    quote_spanned! {name.span()=>
                        .field(#name_string, &format_args!(#s, &self.#name))
                    }
                },
                None => {
                        quote_spanned! {name.span()=>
                            .field(#name_string, &self.#name)
                        }
                },
            }
        } else {
            quote!()
        }
    });
    quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#struct_name)
                #(#recurse)*
                 .finish()
            }
        }
    }
}
