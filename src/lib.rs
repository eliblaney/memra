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
        } else {
            return quote! {
                compile_error!("struct must have named fields");
            }.into();
        }
    } else {
        return quote! {
            compile_error!("macro can only be used on structs");
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