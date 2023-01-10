use quote::{quote, quote_spanned};
use syn::{
    spanned::Spanned, GenericArgument, Ident, PathArguments, Type, Error
};

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
        if f.each.is_some() {
            quote_spanned! { name.span()=>
                #name: Some(vec![]),
            }
        } else {
            quote_spanned! { name.span()=>
                #name: None,
            }
        }
    });
    quote! {
        #builder_name {
            #(#recurse)*
        }
    }
    .into()
}

fn get_nested_type(ty: &Type) -> Type {
    match ty {
        Type::Path(tp) => {
            let outer = tp.path.segments.first().unwrap();
            match outer.arguments.clone() {
                PathArguments::AngleBracketed(aba) => match aba.args.first().unwrap() {
                    GenericArgument::Type(t) => t.to_owned(),
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    }
}

fn gen_setters(fields: &Vec<FieldInfo>) -> proc_macro2::TokenStream {
    let recurse = fields.iter().map(|f| {
        let name = f.name;
        let ty = f.ty;
        let is_optional = f.is_optional;
        let inner = &f.inner;
        if is_optional {
            quote! {
                pub fn #name(&mut self, #name: #inner) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        } else {
            match f.each.clone() {
                Some((ident, str_lit)) => {
                    if ident != "each" {
                        return Error::new(ident.span(), r#"expected `builder(each = "...")`"#).into_compile_error();
                    }
                    let each_name = str_lit.value();
                    let each_id = Ident::new(&each_name, name.span());
                    let nested_type = get_nested_type(ty);
                    let outer_fn = if each_name != name.clone().unwrap().to_string() {
                        quote! {
                            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                                self.#name = Some(#name);
                                self
                            }
                        }
                    } else {
                        proc_macro2::TokenStream::new()
                    };
                    quote! {
                        pub fn #each_id(&mut self, #each_id: #nested_type) -> &mut Self {
                            if let Some(ref mut v) = self.#name {
                                v.push(#each_id);
                            }
                            self
                        }
                        #outer_fn
                    }
                }
                None => {
                    quote! {
                        pub fn #name(&mut self, #name: #ty) -> &mut Self {
                            self.#name = Some(#name);
                            self
                        }
                    }
                }
            }
        }
    });
    quote! {
        #(#recurse)*
    }
}

fn gen_build(fields: &Vec<FieldInfo>) -> proc_macro2::TokenStream {
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
        #(#unwrap_build)*
    }
}

pub fn impl_builder(
    fields: &Vec<FieldInfo>,
    builder_name: &Ident,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    let setters = gen_setters(fields);
    let unwrap_build = gen_build(fields);

    quote! {
        impl #builder_name {
            #setters

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                let s = #struct_name {
                    #unwrap_build
                };
                Ok(s)
            }
        }
    }
    .into()
}
