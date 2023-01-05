use proc_macro::TokenStream;

use quote::{quote, format_ident, quote_spanned};
use syn::{Data, Fields, Ident, FieldsNamed, parse_macro_input, DeriveInput};
use syn::spanned::Spanned;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let fields = parse_fields(&input.data);
    
    println!("fields:");
    for field in fields.named.iter() {
        if let Some(ref name) = field.ident {
            println!("\t{}", name);
        }
    }

    let builder_struct_name = format_ident!("{}Builder", name);
    let builder_struct = get_builder_struct(&fields, &builder_struct_name);
    let init_builder = init_builder_struct(&fields, &builder_struct_name);

    let expanded = quote!{
        #builder_struct
        impl #name {
            pub fn builder() -> #builder_struct_name {
                #init_builder 
            }
        }
    };
    eprintln!("TOKENS: {}", expanded);
    expanded.into()
}

fn parse_fields(data: &Data) -> FieldsNamed {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    fields.clone()
                },
                _ => unimplemented!()
            }
        },
        _ => unimplemented!()
    }
}

fn get_builder_struct(fields: &FieldsNamed, name: &Ident) -> proc_macro2::TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote_spanned! { f.span()=>
            #name: Option<#ty>,
        }
    });
    quote! {
        pub struct #name {
            #(#recurse)*
        }
    }.into()
}

fn init_builder_struct(fields: &FieldsNamed, name: &Ident) -> proc_macro2::TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! { f.span()=>
            #name: None,
        }
    });
    quote! {
        #name {
            #(#recurse)*
        }
    }.into()
}

