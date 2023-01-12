use proc_macro::{TokenStream};

use quote::{quote, quote_spanned};
use syn::{DeriveInput, parse_macro_input, Ident, Type, Data, Fields};

#[proc_macro_derive(CustomDebug)]
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

struct FieldInfo<'a> {
    name: &'a Option<Ident>,
    #[allow(dead_code)]
    ty: &'a Type,
}

fn parse_fields(data: &Data) -> Vec<FieldInfo> {
    match *data {
        Data::Struct(ref data) => match data.fields {
            // Fields::Named(ref fields) => fields.clone(),
            Fields::Named(ref fields) => fields
                .named
                .iter()
                .map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;

                    FieldInfo {
                        name,
                        ty,
                    }
                })
                .collect(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn impl_debug(fields: Vec<FieldInfo>, name: &Ident) -> proc_macro2::TokenStream {
    let struct_name = format!("{}", name);
    
    let recurse = fields.iter().map(|f| {
        if let Some(name) = f.name {
            let name_string = format!("{}", name);
            quote_spanned!{name.span()=>
                .field(#name_string, &self.#name)
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
