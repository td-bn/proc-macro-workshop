use syn::{Data, Fields, Ident, Lit, LitStr, Meta, NestedMeta, Type};

type Option<T> = std::option::Option<T>;

pub struct FieldInfo<'a> {
    pub name: &'a Option<Ident>,
    pub ty: &'a Type,
    pub debug_attr: Option<(Ident, LitStr)>,
}

pub fn parse_fields(data: &Data) -> Vec<FieldInfo> {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields
                .named
                .iter()
                .map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;
                    let debug_attr = match f.attrs.is_empty() {
                        false => {
                            let n = match_meta(f.attrs.first().unwrap().parse_meta().unwrap());
                            Option::Some(n)
                        }
                        true => Option::None,
                    };
                    FieldInfo {
                        name,
                        ty,
                        debug_attr,
                    }
                })
                .collect(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
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
