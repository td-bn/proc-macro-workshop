use syn::{
    Data, Fields, GenericArgument, Ident, Lit, LitStr, Meta, NestedMeta, PathArguments,
    PathSegment, Type,
};

type Option<T> = std::option::Option<T>;

pub struct FieldInfo<'a> {
    pub name: &'a Option<Ident>,
    pub ty: &'a Type,
    pub is_optional: bool,
    pub inner: Option<Ident>,
    pub each: Option<(Ident, LitStr)>,
}


pub fn first_path_segment(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(tp) => tp.path.segments.first(),
        _ => Option::None,
    }
}

pub fn first_generic_arg(args: &PathArguments) -> Option<&PathSegment> {
    match args {
        PathArguments::AngleBracketed(abg_args) => {
            let gen_arg = abg_args.args.first().unwrap();
            match gen_arg {
                GenericArgument::Type(t) => first_path_segment(t),
                _ => Option::None,
            }
        }
        _ => Option::None,
    }
}

fn match_meta(m: Meta) -> (Ident, LitStr) {
    match m {
        Meta::List(l) => match l.nested.first().unwrap() {
            NestedMeta::Meta(m) => match_meta(m.to_owned()),
            _ => unimplemented!(),
        },
        Meta::NameValue(nv) => {
            let path = nv.path;
            let lit = nv.lit;
            let ident = path.segments.first().unwrap().ident.clone();
            let each_name = match lit {
                Lit::Str(s) => (ident, s),
                _ => unimplemented!(),
            };
            each_name
        }
        _ => unimplemented!(),
    }
}

pub fn parse_fields(data: &Data) -> Vec<FieldInfo> {
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
                        Option::Some(ident.to_owned())
                    } else {
                        Option::None
                    };

                    let each_name = match f.attrs.is_empty() {
                        false => {
                            let n = match_meta(f.attrs.first().unwrap().parse_meta().unwrap());
                            Option::Some(n)
                        }
                        true => Option::None,
                    };

                    FieldInfo {
                        name,
                        ty,
                        is_optional,
                        inner,
                        each: each_name,
                    }
                })
                .collect(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
