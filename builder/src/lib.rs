use proc_macro::TokenStream;

use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed, Ident};

#[proc_macro_derive(Builder)]
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
    eprintln!("TOKENS: {}", expanded);
    expanded.into()
}

fn parse_fields(data: &Data) -> FieldsNamed {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields.clone(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
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
    }
    .into()
}

fn init_builder_struct(fields: &FieldsNamed, builder_name: &Ident) -> proc_macro2::TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! { f.span()=>
            #name: None,
        }
    });
    quote! {
        #builder_name {
            #(#recurse)*
        }
    }
    .into()
}

fn impl_builder(fields: &FieldsNamed, builder_name: &Ident, struct_name: &Ident) -> proc_macro2::TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote_spanned! { f.span()=>
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let unwrap_build = fields.named.iter().map(|f| {
        let name = &f.ident;
        let name_string = name.clone().unwrap().to_string();
        let error_message = format!("{} not present", name_string);
        quote_spanned! { f.span()=>
            #name: match &self.#name {
                Some(v) => v.to_owned(),
                None => return Err(#error_message.into())
            },
        }
    });

    quote! {
        impl #builder_name {
            #(#recurse)*

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                let s = #struct_name {
                    #(#unwrap_build)*
                };
                Ok(s)
            }
        }
    }
    .into()
}
