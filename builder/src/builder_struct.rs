use quote::{quote, quote_spanned};
use syn::{Ident, spanned::Spanned};


use super::fields::FieldInfo;

pub fn get_builder_struct(fields: &Vec<FieldInfo>, name: &Ident) -> proc_macro2::TokenStream {
    let recurse = fields.iter().map(|f| {
        let name = f.name;
        let ty = f.ty;
        let is_optional = f.is_optional;
        if is_optional {
            quote_spanned! { name.span()=>
                #name: #ty,
            }
        } else {
            quote_spanned! { name.span()=>
                #name: Option<#ty>,
            }
        }
    });
    quote! {
        pub struct #name {
            #(#recurse)*
        }
    }
    .into()
}

pub fn init_builder_struct(
    fields: &Vec<FieldInfo>,
    builder_name: &Ident,
) -> proc_macro2::TokenStream {
    let recurse = fields.iter().map(|f| {
        let name = f.name;
        quote_spanned! { name.span()=>
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

pub fn impl_builder(
    fields: &Vec<FieldInfo>,
    builder_name: &Ident,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    let recurse = fields.iter().map(|f| {
        let name = f.name;
        let ty = f.ty;
        let is_optional = f.is_optional;
        let inner = &f.inner;
        if is_optional {
            quote_spanned! { name.span()=>
                pub fn #name(&mut self, #name: #inner) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        } else {
            quote_spanned! { name.span()=>
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        }
    });

    let unwrap_build = fields.iter().map(|f| {
        let name = f.name;
        let name_string = name.clone().unwrap().to_string();
        let error_message = format!("{} not present", name_string);
        let is_optional = f.is_optional;
        if is_optional {
            quote_spanned! { name.span()=>
                #name: self.#name.as_ref().cloned(),
            }
        } else {
            quote_spanned! { name.span()=>
                #name: match &self.#name {
                    Some(v) => v.to_owned(),
                    None => return Err(#error_message.into())
                },
            }
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
