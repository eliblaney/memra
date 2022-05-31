use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::*;
use parse::Parser;
use indexmap::IndexMap;

#[proc_macro_attribute]
pub fn model(args: TokenStream, input: TokenStream) -> TokenStream  {
    let args = parse_macro_input!(args as AttributeArgs);
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
    let mut table: String = format!("{}s", &name.to_string().to_lowercase()).into();

    for arg in args {
        if let NestedMeta::Meta(inner) = arg {
            if let Meta::NameValue(nv) = inner {
                let ident = &nv.path.segments.first().unwrap().ident;
                if ident == &format_ident!("table") {
                    if let Lit::Str(s) = &nv.lit {
                        table = s.value()
                    }
                }
            }
        }
    }

    if let Data::Struct(s) = &ast.data {
        if let Fields::Named(f) = &s.fields {
            let fields = &f.named;
            let types: Vec<_> = fields.iter().map(|x| &x.ty).collect();
            let fields: Vec<_> = fields.iter().map(|x| &x.ident).collect();
            // row.get(field) repetitions for ::from(PgRow)
            let mut getters = quote! {};
            for field in &fields {
                let field_str = quote! { #field }.to_string();
                getters = quote! {
                    #getters
                    #field: r.get(#field_str),
                };
            }
            // SQL variables ($1, $2, etc) in INSERT statement
            let size = fields.len();
            let mut col_vars = vec![];
            for i in 0..size {
                col_vars.push(format!("${}", i + 2));
            }
            let col_vars = col_vars.join(",");
            let mut val_vars = vec![];
            for i in size..2*size {
                val_vars.push(format!("${}", i + 2));
            }
            // Struct fields to bind as variables in INSERT statement
            let val_vars = val_vars.join(",");
            let mut bind_columns = quote! {};
            for field in &fields {
                let field_str = quote! { #field }.to_string();
                bind_columns = quote! {
                    #bind_columns
                    .bind(#field_str)
                };
            }
            let mut bind_values = quote! {};
            for field in &fields {
                bind_values = quote! {
                    #bind_values
                    .bind(&self.#field)
                };
            }
            let insert_sql = format!("INSERT INTO $1 ({}) VALUES ({}) RETURNING *", col_vars, val_vars);
            // SQL variables ($1, $2, etc) in UPDATE statement
            let mut set_vars = vec![];
            for i in (0..2*size).step_by(2) {
                set_vars.push(format!("${} = ${}", i, i + 1));
            }
            let set_vars = set_vars.join(",");
            // Struct fields to bind as variables in UPDATE statement
            let mut set_binds = quote! {};
            for field in &fields {
                let field_str = quote! { #field }.to_string();
                set_binds = quote! {
                    #set_binds
                    .bind(#field_str)
                    .bind(&self.#field)
                };
            }
            let update_sql = format!("UPDATE $1 SET {} RETURNING *", set_vars);

            // Fields and types to accept in ::new() (skipping the first field/type, id)
            let mut new_params = quote! {};
            let mut new_constructor = quote! {};
            for (field, ty) in std::iter::zip(fields, types).skip(1) {
                new_params = quote! {
                    #new_params #field: #ty,
                };
                new_constructor = quote! {
                    #new_constructor #field,
                };
            }
            
            return quote! {
                #ast

                impl From<rocket_db_pools::sqlx::postgres::PgRow> for #name {
                    fn from(r: rocket_db_pools::sqlx::postgres::PgRow) -> Self {
                        use rocket_db_pools::sqlx::Row;
                        Self {
                            #getters
                        }
                    }
                }

                impl #name {
                    pub fn table() -> &'static str {
                        #table
                    }

                    pub async fn find(id: i32, mut db: rocket_db_pools::Connection<crate::Db>) -> (Option<Self>, rocket_db_pools::Connection<crate::Db>) {
                        use rocket::futures::TryFutureExt;
                        (sqlx::query("SELECT * FROM $1 WHERE id = $2")
                            .bind(#table)
                            .bind(id)
                            .fetch_one(&mut *db)
                            .map_ok(|r| <#name>::from(r))
                            .await.ok(), db)
                    }

                    pub async fn save(&self, mut db: rocket_db_pools::Connection<crate::Db>) -> (Option<Self>, rocket_db_pools::Connection<crate::Db>) {
                        use rocket::futures::TryFutureExt;
                        match self.id {
                            None => {
                                (sqlx::query(#insert_sql)
                                    .bind(#table)
                                    #bind_columns
                                    #bind_values
                                    .fetch_one(&mut *db)
                                    .map_ok(|r| <#name>::from(r))
                                    .await.ok(), db)
                            },
                            Some(_) => {
                                (sqlx::query(#update_sql)
                                    .bind(#table)
                                    #set_binds
                                    .fetch_one(&mut *db)
                                    .map_ok(|r| <#name>::from(r))
                                    .await.ok(), db)
                            }
                        }
                    }

                    pub async fn read(id: i32, mut db: rocket_db_pools::Connection<crate::Db>) -> (Option<Self>, rocket_db_pools::Connection<crate::Db>) {
                        use rocket::futures::TryFutureExt;
                        (sqlx::query("SELECT * FROM $1 WHERE id = $2")
                         .bind(#table)
                         .bind(id)
                         .fetch_one(&mut *db)
                         .map_ok(|r| Self::from(r))
                         .await.ok(), db)
                    }

                    pub async fn delete(id: i32, mut db: rocket_db_pools::Connection<crate::Db>) -> (std::result::Result<u64, sqlx::Error>, rocket_db_pools::Connection<crate::Db>) {
                        use rocket::futures::TryFutureExt;
                        (sqlx::query("DELETE FROM $1 WHERE id = $2")
                            .bind(#table)
                            .bind(id)
                            .execute(&mut *db)
                            .map_ok(|r| r.rows_affected())
                            .await, db)
                    }

                    pub fn new(#new_params) -> Self {
                        Self {
                            id: None, #new_constructor
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
            let mut linked_fields: IndexMap<Field, Option<Path>> = IndexMap::new();
            let quotes: Vec<_> = fields.into_iter().filter_map(|p| {
                let attrs = &p.attrs;
                let attrs: Vec<&Attribute> = attrs.into_iter().filter(|a| {
                    let segs = &a.path.segments;
                    segs.len() == 1
                        && segs.first().unwrap().ident == format_ident!("foreign") 
                }).collect();

                if attrs.len() != 1 {
                    let field_copy: Field = Field::parse_named.parse2(quote! { #p }).ok().unwrap();
                    linked_fields.insert(field_copy, None);
                    return None;
                }

                let meta = &attrs.first().unwrap().parse_meta().unwrap();
                let mut obj: Option<String> = None;
                let lower_name = &name.to_string().to_lowercase();
                let mut lower_name_ident = format_ident!("{}", &lower_name);

                if let Meta::List(ml) = meta {
                    ml.nested.iter().for_each(|m| {
                        if let NestedMeta::Meta(inner) = m {
                            if let Meta::NameValue(nv) = inner {
                                let ident = &nv.path.segments.first().unwrap().ident;
                                if ident == &format_ident!("type") {
                                    if let Lit::Str(s) = &nv.lit {
                                        obj = Some(s.value())
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

                if obj.is_none() {
                    return quote! {
                        compile_error!("foreign attribute must include type");
                    }.into();
                }

                let obj: Result<Path> = parse_str(obj.unwrap().as_str());

                if obj.is_err() {
                    return quote! {
                        compile_error!("type must be ident or path");
                    }.into();
                }

                let obj = obj.ok();
                let field = &p.ident.as_ref().unwrap();
                let field_string: &str = &field.to_string();
                let shortened_field = &field_string.split("_").next().unwrap();
                let fname = format_ident!("get_{}", &shortened_field);
                let f2name = format_ident!("find_{}", &lower_name_ident);

                let q = quote! {
                    impl #name {
                        pub async fn #fname(&self, mut db: rocket_db_pools::Connection<crate::Db>) -> #obj {
                            use rocket::futures::TryFutureExt;
                            rocket_db_pools::sqlx::query("SELECT * FROM $1 WHERE id = $2")
                                .bind(<#obj>::table()).bind(&self.#field)
                                .fetch_one(&mut *db)
                                .map_ok(|r| <#obj>::from(r))
                                .await.ok().unwrap()
                        }
                    }

                    impl #obj {
                        pub async fn #f2name(&self, mut db: rocket_db_pools::Connection<crate::Db>) -> Vec<#name> {
                            use rocket::futures::TryStreamExt;
                            rocket_db_pools::sqlx::query("SELECT * FROM $1 WHERE $1.$2 = $3")
                                .bind(<#name>::table()).bind(#field_string).bind(&self.id)
                                .fetch(&mut *db)
                                .map_ok(|r| <#name>::from(r))
                                .try_collect::<Vec<_>>()
                                .await.ok().unwrap()
                        }
                    }

                };

                let field_copy: Field = Field::parse_named.parse2(quote! { #p }).ok().unwrap();
                linked_fields.insert(field_copy, Some(obj.unwrap()));

                return Some(q);
            }).collect();

            // Fields and types to accept in ::new() (skipping the first field/type, id)
            let mut new_params = quote! {};
            let mut new_constructor = quote! {};
            let mut new_safeguards = quote! {};
            for (field, path) in linked_fields {
                let ident = &field.ident.unwrap();
                let ident_str = &ident.to_string();
                if ident_str == "id" { continue; }
                let ident_no_id = format_ident!("{}", str::replace(ident_str, "_id", ""));
                let ty = &field.ty;
                match path {
                    Some(path) =>  {
                        new_params = quote! {
                            #new_params #ident_no_id: #path,
                        };
                        new_constructor = quote! {
                            #new_constructor #ident: #ident_no_id.id.unwrap(),
                        };
                        new_safeguards = quote! {
                            #new_safeguards
                            if #ident_no_id.id.is_none() {
                                return None;
                            }
                        };
                    },
                    None =>  {
                        new_params = quote! {
                            #new_params #ident: #ty,
                        };
                        new_constructor = quote! {
                            #new_constructor #ident,
                        };
                    }
                };
            }

            return builder(&mut quotes.into_iter(), quote! {
                impl #name {
                    pub fn new_from(#new_params) -> Option<Self> {
                        #new_safeguards
                        Some(Self {
                            id: None, #new_constructor
                        })
                    }
                }
            }).into();
        }
    }

    quote! {
        compile_error!("macro can only be used on structs with named fields");
    }.into()
}