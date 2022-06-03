use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::*;
use parse::Parser;
use indexmap::IndexMap;

#[proc_macro_attribute]
pub fn model(args: TokenStream, input: TokenStream) -> TokenStream {
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
            let col_vars = fields.iter().skip(1).map(|f| quote! { #f }.to_string()).collect::<Vec<String>>().join(",");
            let mut val_vars = vec![];
            for i in 1..size {
                val_vars.push(format!("${}", i));
            }
            // Struct fields to bind as variables in INSERT statement
            let val_vars = val_vars.join(",");
            let mut bind_values = quote! {};
            for field in fields.iter().skip(1) {
                bind_values = quote! {
                    #bind_values.bind(&self.#field)
                };
            }
            let insert_sql = format!("INSERT INTO {} ({}) VALUES ({}) RETURNING *", table, col_vars, val_vars);
            // SQL variables ($1, $2, etc) in UPDATE statement
            let mut set_vars = vec![];
            for (i, field) in fields.iter().skip(1).enumerate() {
                set_vars.push(format!("{} = ${}", quote! { #field }.to_string(), i + 1));
            }
            let set_vars = set_vars.join(",");
            // Struct fields to bind as variables in UPDATE statement
            let mut set_binds = quote! {};
            for field in fields.iter().skip(1) {
                set_binds = quote! {
                    #set_binds.bind(&self.#field)
                };
            }
            let update_sql = format!("UPDATE {} SET {} RETURNING *", table, set_vars);

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

            let find_sql = format!("SELECT * FROM {} WHERE id = $1", table);
            let read_sql = format!("SELECT * FROM {} WHERE id = $1", table);
            let delete_sql = format!("DELETE FROM {} WHERE id = $1", table);

            return quote! {
                #[derive(Debug, Clone, Deserialize, Serialize)]
                #[serde(crate = "rocket::serde")]
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
                        (rocket_db_pools::sqlx::query(#find_sql)
                            .bind(id)
                            .fetch_one(&mut *db)
                            .map_ok(|r| <#name>::from(r))
                            .await.ok(), db)
                    }

                    pub async fn find_where(field: &str, value: &String, mut db: rocket_db_pools::Connection<crate::Db>) -> (Option<Self>, rocket_db_pools::Connection<crate::Db>) {
                        use rocket::futures::TryFutureExt;
                        (rocket_db_pools::sqlx::query(format!("SELECT * FROM {} WHERE {} = $1", <#name>::table(), field).as_str())
                            .bind(value)
                            .fetch_one(&mut *db)
                            .map_ok(|r| <#name>::from(r))
                            .await.ok(), db)
                    }

                    pub async fn save(&self, mut db: rocket_db_pools::Connection<crate::Db>) -> (Option<Self>, rocket_db_pools::Connection<crate::Db>) {
                        use rocket::futures::TryFutureExt;
                        match self.id {
                            None => (
                                rocket_db_pools::sqlx::query(#insert_sql)
                                    #bind_values
                                    .fetch_one(&mut *db)
                                    .map_ok(|r| <#name>::from(r))
                                    .await.ok(), db
                            ),
                            Some(_) => (
                                rocket_db_pools::sqlx::query(#update_sql)
                                    #set_binds
                                    .fetch_one(&mut *db)
                                    .map_ok(|r| <#name>::from(r))
                                    .await.ok(), db
                            )
                        }
                    }

                    pub async fn read(id: i32, mut db: rocket_db_pools::Connection<crate::Db>) -> (Option<Self>, rocket_db_pools::Connection<crate::Db>) {
                        use rocket::futures::TryFutureExt;
                        (rocket_db_pools::sqlx::query(#read_sql)
                         .bind(id)
                         .fetch_one(&mut *db)
                         .map_ok(|r| Self::from(r))
                         .await.ok(), db)
                    }

                    pub async fn delete(id: i32, mut db: rocket_db_pools::Connection<crate::Db>) -> (std::result::Result<u64, sqlx::Error>, rocket_db_pools::Connection<crate::Db>) {
                        use rocket::futures::TryFutureExt;
                        (rocket_db_pools::sqlx::query(#delete_sql)
                            .bind(id)
                            .execute(&mut *db)
                            .map_ok(|r| r.rows_affected())
                            .await, db)
                    }

                    pub fn json(self) -> rocket::serde::json::Json<#name> {
                        rocket::serde::json::Json(self)
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
pub fn impl_related(input: TokenStream) -> TokenStream {
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
                        pub async fn #fname(&self, mut db: rocket_db_pools::Connection<crate::Db>) -> (Option<#obj>, rocket_db_pools::Connection<crate::Db>) {
                            use rocket::futures::TryFutureExt;
                            (rocket_db_pools::sqlx::query(format!("SELECT * FROM {} WHERE id = $1", <#obj>::table()).as_str())
                                .bind(&self.#field)
                                .fetch_one(&mut *db)
                                .map_ok(|r| <#obj>::from(r))
                                .await.ok(), db)
                        }
                    }

                    impl #obj {
                        pub async fn #f2name(&self, mut db: rocket_db_pools::Connection<crate::Db>) -> (Vec<#name>, rocket_db_pools::Connection<crate::Db>) {
                            use rocket::futures::TryStreamExt;
                            let result = rocket_db_pools::sqlx::query(format!("SELECT * FROM {} WHERE {} = $1", <#name>::table(), #field_string).as_str())
                                .bind(&self.id)
                                .fetch(&mut *db)
                                .map_ok(|r| <#name>::from(r))
                                .try_collect::<Vec<_>>()
                                .await.ok();
                            (result.unwrap_or(vec![]), db)
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
                            #new_params #ident_no_id: &#path,
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

#[proc_macro_derive(CreateAsOwner)]
pub fn impl_create_as_owner(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let fname = format_ident!("create_{}", &name.to_string().to_lowercase());

    quote! {
        #[post("/", data = "<model>")]
        pub async fn #fname(db: rocket_db_pools::Connection<crate::Db>, user: crate::auth::AuthenticatedUser, model: rocket::serde::json::Json<#name>) -> Option<rocket::response::status::Created<rocket::serde::json::Json<#name>>> {
            let mut model = model.into_inner();
            model.user_id = user.id();
            let (model, _) = model.save(db).await;

            match model {
                None => None,
                Some(m) => Some(rocket::response::status::Created::new("/").body(rocket::serde::json::Json(m)))
            }
        }
    }.into()
}

#[proc_macro_derive(Read)]
pub fn impl_read(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let fname = format_ident!("read_{}", &name.to_string().to_lowercase());

    quote! {
        #[get("/<id>")]
        pub async fn #fname(db: rocket_db_pools::Connection<crate::Db>, _user: crate::auth::User, id: i32) -> Option<rocket::serde::json::Json<#name>> {
            let (m, _db) = <#name>::read(id, db).await;
            m.json()
        }
    }.into()
}

#[proc_macro_derive(ReadIfVisible)]
pub fn impl_read_if_visible(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let fname = format_ident!("read_{}", &name.to_string().to_lowercase());

    quote! {
        #[get("/<id>")]
        pub async fn #fname(db: rocket_db_pools::Connection<crate::Db>, user: crate::auth::User, id: i32) -> Option<rocket::serde::json::Json<#name>> {
            let (m, _db) = <#name>::read(id, db).await;
            if m.is_none() {
                return None;
            }
            let m = m.unwrap();
            if m.visibility.is_some()  {
                match user {
                    crate::auth::User::Guest =>  {
                        return None;
                    },
                    crate::auth::User::Authenticated(user) => {
                        if m.visibility.unwrap() && m.user_id != user.id() {
                            return None;
                        }
                    }
                }
            }
            Some(m.json())
        }
    }.into()
}

#[proc_macro_derive(ReadIfOwner)]
pub fn impl_read_if_owner(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let fname = format_ident!("read_{}", &name.to_string().to_lowercase());

    quote! {
        #[get("/<id>")]
        pub async fn #fname(db: rocket_db_pools::Connection<crate::Db>, user: crate::auth::User, id: i32) -> Option<rocket::serde::json::Json<#name>> {
            if let crate::auth::User::Guest = user {
                return None;
            }

            let (m, _db) = <#name>::read(id, db).await;
            if m.is_none() {
                return None;
            }
            let m = m.unwrap();
            if m.user_id != user.id().unwrap() {
                return None;
            }
            Some(m.json())
        }
    }.into()
}

#[proc_macro_derive(UpdateIfOwner)]
pub fn impl_update_if_owner(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let fname = format_ident!("update_{}", &name.to_string().to_lowercase());

    quote! {
        #[put("/", data = "<model>")]
        pub async fn #fname(db: rocket_db_pools::Connection<crate::Db>, user: crate::auth::AuthenticatedUser, model: rocket::serde::json::Json<#name>) -> Option<rocket::serde::json::Json<#name>> {
            let model = model.into_inner();
            if model.id.is_none() {
                return None;
            }

            let (model, db) = <#name>::read(model.id.unwrap(), db).await;
            if model.is_none() {
                return None;
            }

            let model = model.unwrap();
            if model.user_id != user.id() {
                return None;
            }

            let (m, _) = model.save(db).await;
            m.json()
        }
    }.into()
}

#[proc_macro_derive(DeleteIfOwner)]
pub fn impl_delete_if_owner(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let fname = format_ident!("delete_{}", &name.to_string().to_lowercase());

    quote! {
        #[delete("/<id>")]
        pub async fn #fname(db: rocket_db_pools::Connection<crate::Db>, user: crate::auth::AuthenticatedUser, id: i32) -> rocket::serde::json::Json<bool> {
            let (m, db) = <#name>::find(id, db).await;
            if m.is_none() {
                return rocket::serde::json::Json(false);
            }
            if m.unwrap().user_id != user.id() {
                return rocket::serde::json::Json(false);
            }
            let (rows_affected, _) = <#name>::delete(user.id(), db).await;
            rocket::serde::json::Json(rows_affected.is_ok())
        }
    }.into()
}

#[proc_macro_attribute]
pub fn router(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);

    let mut models: Vec<Path> = vec![];
    for arg in args {
        if let NestedMeta::Meta(inner) = arg {
            if let Meta::Path(path) = inner {
                models.push(path);
            } else {
                return quote! {
                    compile_error!("router can only accept paths to model structs");
                }.into();
            }
        }
    }
    let path_prefix = models.remove(0);

    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let name_str = format!("{}", &name);
    let mut mount_routes = quote! { };

    for model in models {
        let mut segments = model.segments;
        let ident = &segments.pop().unwrap();
        let ident = &ident.value().ident;
        let path = &mut path_prefix.clone();
        path.segments.extend(segments);
        let lower_name = &ident.to_string().to_lowercase();
        let mount_point = format!("/{}", &lower_name);

        let methods: Vec<Path> = vec!["create_$", "read_$", "update_$", "delete_$"]
            .iter().map(|s| {
                let i = format_ident!("{}", s.replace("$", &lower_name));
                let mut p_clone = path.clone();
                p_clone.segments.push(PathSegment {
                    ident: i,
                    arguments: PathArguments::None,
                });
                p_clone
            }).collect();
        
        mount_routes = quote! {
            #mount_routes
            .mount(#mount_point, routes![#(#methods),*])
        };
    }

    quote! {
        #ast

        #[rocket::async_trait]
        impl rocket::fairing::Fairing for #name {
            fn info(&self) -> rocket::fairing::Info {
                rocket::fairing::Info {
                    name: #name_str,
                    kind: rocket::fairing::Kind::Ignite
                }
            }

            async fn on_ignite(&self, rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
                Ok(rocket
                    #mount_routes
                )
            }
        }
    }.into()
}