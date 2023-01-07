use proc_macro::TokenStream;

use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, GenericArgument, Ident, PathArguments,
    PathSegment, Type,
};

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

struct FieldInfo<'a> {
    name: &'a Option<Ident>,
    ty: &'a Type,
    is_optional: bool,
    inner: Option<Ident>,
}

fn first_path_segment(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(tp) => tp.path.segments.first(),
        _ => None,
    }
}

fn first_generic_arg(args: &PathArguments) -> Option<&PathSegment> {
    match args {
        PathArguments::AngleBracketed(abg_args) => {
            let gen_arg = abg_args.args.first().unwrap();
            match gen_arg {
                GenericArgument::Type(t) => first_path_segment(t),
                _ => None,
            }
        }
        _ => None,
    }
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
                    let first_path_segment = first_path_segment(ty).unwrap();
                    let is_optional = first_path_segment.ident.to_string() == "Option";
                    let inner = if is_optional {
                        let ident = &first_generic_arg(&first_path_segment.arguments)
                            .unwrap()
                            .ident;
                        Some(ident.to_owned())
                    } else {
                        None
                    };
                    FieldInfo {
                        name,
                        ty,
                        is_optional,
                        inner,
                    }
                })
                .collect(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn get_builder_struct(fields: &Vec<FieldInfo>, name: &Ident) -> proc_macro2::TokenStream {
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

fn init_builder_struct(fields: &Vec<FieldInfo>, builder_name: &Ident) -> proc_macro2::TokenStream {
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

fn impl_builder(
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
