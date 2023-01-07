use syn::{Data, Fields, GenericArgument, Ident, PathArguments, PathSegment, Type};

pub struct FieldInfo<'a> {
    pub name: &'a Option<Ident>,
    pub ty: &'a Type,
    pub is_optional: bool,
    pub inner: Option<Ident>,
}

pub fn first_path_segment(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(tp) => tp.path.segments.first(),
        _ => None,
    }
}

pub fn first_generic_arg(args: &PathArguments) -> Option<&PathSegment> {
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
