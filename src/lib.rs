use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::*;

#[proc_macro_attribute]
pub fn model(_args: TokenStream, input: TokenStream) -> TokenStream  {
    let mut ast = parse_macro_input!(input as DeriveInput);
    if let Data::Struct(ref mut struct_data) = &mut ast.data {
        if let Fields::Named(fields) = &mut struct_data.fields {
            let named = &mut fields.named;
            let mut p = punctuated::Punctuated::<PathSegment, token::Colon2>::new();
            p.push(PathSegment { ident: format_ident!("serde"), arguments: PathArguments::None });
            let id_field = Field {
                attrs: vec![
                    // #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
                    Attribute {
                        pound_token: token::Pound::default(),
                        style: AttrStyle::Outer,
                        bracket_token: token::Bracket::default(),
                        path: Path { 
                            leading_colon: None,
                            segments: p
                        },
                        tokens: quote! { (skip_deserializing, skip_serializing_if = "Option::is_none") }
                    }
                ],
                // pub id: Option<i32>
                vis: Visibility::Public(VisPublic { pub_token: token::Pub::default() }),
                ident: Some(format_ident!("id")),
                colon_token: Some(token::Colon::default()),
                ty: Type::Verbatim(quote! { Option<i32> })
            };
            named.insert(0, id_field);
        }
    } else {
        return quote! {
            compile_error!("macro can only be used on structs with named fields");
        }.into();
    }

    let name = &ast.ident;
    if let Data::Struct(s) = &ast.data {
        if let Fields::Named(f) = &s.fields {
            let fields = &f.named;
            let iter = fields.into_iter().map(|f| &f.ident).into_iter();
            return quote! {
                #ast

                impl From<rocket_db_pools::sqlx::postgres::PgRow> for #name {
                    fn from(r: rocket_db_pools::sqlx::postgres::PgRow) -> Self {
                        use rocket_db_pools::sqlx::Row;
                        Self {
                            #(#iter: r.get("#iter")),*
                        }
                    }
                }
            }.into();
        }
    }

    quote! {
        compile_error!("can't parse struct");
    }.into()
}

fn builder(i: &mut std::vec::IntoIter<proc_macro2::TokenStream>, a: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let q = i.next();
    if q.is_none() { return a; }
    let q = q.unwrap();
    return builder(i, quote! {
        #a
        #q
    });
}

#[proc_macro_derive(Related, attributes(foreign))]
pub fn impl_related(input: TokenStream) -> TokenStream  {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    if let Data::Struct(s) = &ast.data {
        if let Fields::Named(f) = &s.fields {
            let fields = &f.named;
            let quotes: Vec<_> = fields.into_iter().filter_map(|p| {
                let attrs = &p.attrs;
                let attrs: Vec<&Attribute> = attrs.into_iter().filter(|a| {
                    let segs = &a.path.segments;
                    segs.len() == 1
                        && segs.first().unwrap().ident == format_ident!("foreign") 
                }).collect();

                if attrs.len() != 1 {
                    return None;
                }

                let meta = &attrs.first().unwrap().parse_meta().unwrap();
                let mut obj: Option<String> = None;
                let mut table: Option<String> = None;
                let lower_name = &name.to_string().to_lowercase();
                let mut lower_name_ident = format_ident!("{}", &lower_name);

                if let Meta::List(ml) = &meta {
                    ml.nested.iter().for_each(|m| {
                        if let NestedMeta::Meta(inner) = m {
                            if let Meta::NameValue(nv) = inner {
                                let ident = &nv.path.segments.first().unwrap().ident;
                                if ident == &format_ident!("type") {
                                    if let Lit::Str(s) = &nv.lit {
                                        obj = Some(s.value());
                                    }
                                }
                                if ident == &format_ident!("table") {
                                    if let Lit::Str(s) = &nv.lit {
                                        table = Some(s.value())
                                    }
                                }
                                if ident == &format_ident!("collect") {
                                    if let Lit::Str(s) = &nv.lit {
                                        lower_name_ident = format_ident!("{}", s.value())
                                    }
                                }
                            }
                        }
                    });
                }

                if obj.is_none() || table.is_none() {
                    return quote! {
                            compile_error!("foreign attribute must include type and table");
                        }.into();
                }

                let obj = obj.unwrap();
                let table = table.unwrap();
                let obj = format_ident!("{}", &obj);
                let field = &p.ident.as_ref().unwrap();
                let field_string: &str = &field.to_string();
                let shortened_field = &field_string.split("_").next().unwrap();
                let fname = format_ident!("get_{}", &shortened_field);
                let q = quote! {
                    impl #name {
                        pub async fn #fname(&self, mut db: rocket_db_pools::Connection<super::Db>) -> #obj {
                            use rocket::futures::TryFutureExt;
                            rocket_db_pools::sqlx::query("SELECT * FROM $1 WHERE id = $2")
                                .bind(#table).bind(&self.#field)
                                .fetch_one(&mut *db)
                                .map_ok(|r| <#obj>::from(r))
                                .await.ok().unwrap()
                        }
                    }

                    impl #obj {
                        pub async fn #lower_name_ident(&self, mut db: rocket_db_pools::Connection<super::Db>) -> Vec<#name> {
                            use rocket::futures::TryStreamExt;
                            rocket_db_pools::sqlx::query("SELECT * FROM $1 WHERE $1.$2 = $3")
                                .bind(#lower_name).bind(#field_string).bind(&self.id)
                                .fetch(&mut *db)
                                .map_ok(|r| <#name>::from(r))
                                .try_collect::<Vec<_>>()
                                .await.ok().unwrap()
                        }
                    }

                };

                return Some(q);
            }).collect();

            return builder(&mut quotes.into_iter(), quote! {}).into();
        }
    }

    quote! {
        compile_error!("macro can only be used on structs with named fields");
    }.into()
}