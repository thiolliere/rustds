#![recursion_limit="128"]
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
extern crate mutate;

use proc_macro::TokenStream;
use syn::DeriveInput;

enum MutateAttribute {
    Skip
}

impl syn::synom::Synom for MutateAttribute {
    named!(parse -> Self, map!(
        parens!(custom_keyword!(skip)),
        |(_, _)| MutateAttribute::Skip
    ));
}
#[proc_macro_derive(CompositeMutate, attributes(mutate))]
pub fn composite_mutate(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let name = &input.ident;
    let vis = &input.vis;
    let database_name = syn::Ident::new(&format!("{}MutateDatabase", name), proc_macro2::Span::call_site());
    let data = match input.data {
        syn::Data::Struct(ref data_struct) => data_struct,
        _ => panic!("expect a struct"),
    };
    let expanded = match data.fields {
        syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: ref fields,
            ..
        }) => {
            // TODO: allow skip attributes for unnamed
            let fields_ty = fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>();
            let fields_ty2 = fields_ty.clone();
            let fields_id = fields.iter().enumerate().map(|(id, _)| id).collect::<Vec<_>>();
            let fields_id2 = fields_id.clone();
            let fields_id3 = fields_id.clone();
            let fields_id4 = fields_id.clone();
            quote! {
                #vis struct #database_name (#(
                        <#fields_ty as ::mutate::Mutate>::Database,
                )*);
                impl ::mutate::Mutate for #name {
                    type Database = #database_name;
                    fn new_database() -> Self::Database {
                        #database_name (#(
                            #fields_ty2::new_database(),
                        )*)
                    }
                    fn insert_in_database(&self, database: &mut Self::Database) {
                        #(
                            self.#fields_id.insert_in_database(&mut database.#fields_id2);
                        )*
                    }
                    fn mutate_in_database(&self, database: &Self::Database, distribution: &impl ::rand::distributions::Distribution<f64>) -> Self {
                        #name (#(
                            self.#fields_id3.mutate_in_database(&database.#fields_id4, distribution),
                        )*)
                    }
                }
            }
        },
        syn::Fields::Named(syn::FieldsNamed {
            named: ref fields,
            ..
        }) => {
            let (fields, skip_fields): (Vec<_>, Vec<_>) = fields.iter()
                .partition(|f| {
                    f.attrs.iter()
                        .find(|attr| attr.path.segments[0].ident == "mutate")
                        .map(|attr| {
                            let _parse = syn::parse2::<MutateAttribute>(attr.tts.clone())
                                .unwrap();
                            false
                        })
                        .unwrap_or(true)
                });

            let fields_ty = fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>();
            let fields_ty2 = fields_ty.clone();
            let fields_id = fields.iter().enumerate().map(|(id, _)| id).collect::<Vec<_>>();
            let fields_id2 = fields_id.clone();
            let fields_name = fields.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>();
            let fields_name2 = fields_name.clone();
            let fields_name3 = fields_name.clone();

            let skip_fields_name = skip_fields.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>();
            let skip_fields_name2 = skip_fields_name.clone();

            quote! {
                #vis struct #database_name (#(
                        <#fields_ty as ::mutate::Mutate>::Database,
                )*);
                impl ::mutate::Mutate for #name {
                    type Database = #database_name;
                    fn new_database() -> Self::Database {
                        #database_name (#(
                            #fields_ty2::new_database(),
                        )*)
                    }
                    fn insert_in_database(&self, database: &mut Self::Database) {
                        #(
                            self.#fields_name.insert_in_database(&mut database.#fields_id);
                        )*
                    }
                    fn mutate_in_database(&self, database: &Self::Database, distribution: &impl ::rand::distributions::Distribution<f64>) -> Self {
                        #name {
                            #(
                                #fields_name2: self.#fields_name3.mutate_in_database(&database.#fields_id2, distribution),
                            )*
                            #(
                                #skip_fields_name: self.#skip_fields_name2.clone(),
                            )*
                        }
                    }
                }
            }
        },
        _ => panic!("unexpected unit fields"),
    };

    expanded.into()
}
