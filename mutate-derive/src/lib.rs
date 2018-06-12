#![recursion_limit="256"]
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

#[proc_macro_derive(EnumMutate)]
pub fn enum_mutate(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let name = &input.ident;
    let vis = &input.vis;
    let database_name = syn::Ident::new(&format!("{}MutateDatabase", name), proc_macro2::Span::call_site());
    let data = match input.data {
        syn::Data::Enum(ref data_enum) => data_enum,
        _ => panic!("expect an enum"),
    };
    let variants = data.variants.iter()
        .inspect(|v| match v.fields {
            syn::Fields::Unit => (),
            _ => panic!("expect enum with no fields"),
        })
        .map(|v| &v.ident)
        .collect::<Vec<_>>();
    let variants2 = variants.clone();
    let variants3 = variants.clone();

    let variants_id = variants.iter()
        .enumerate()
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    let variants_id2 = variants_id.clone();
    let variants_id3 = variants_id.clone();

    let variants_count = variants.len();
    let variants_count2 = variants_count;

    let expanded = quote! {
        #vis struct #database_name ([usize; #variants_count]);
        impl ::mutate::Mutate for #name {
            type Database = #database_name;
            fn new_database() -> Self::Database {
                #database_name ([0; #variants_count2])
            }
            fn insert_in_database(&self, database: &mut Self::Database) {
                use #name as _MutateSelfShortCut;
                let id = match self {
                    #(_MutateSelfShortCut::#variants => #variants_id,)*
                };
                database.0[id] += 1;
            }
            fn mutate_in_database(&self, database: &Self::Database, distribution: &impl ::rand::distributions::Distribution<f64>) -> Self {
                // TODO: check if this is OK
                use ::rand::Rng;
                use ::rand::distributions::Distribution;
                use #name as _MutateSelfShortCut;

                let mut rng = ::rand::thread_rng();
                let mut order = (0..#variants_count).collect::<Vec<_>>();
                rng.shuffle(&mut order);

                let id = match self {
                    #(_MutateSelfShortCut::#variants2 => #variants_id2,)*
                };

                let start_range = (0..#variants_count)
                    .map(|i| {
                        order.iter().take(i).map(|&id| database.0[id]).sum::<usize>()
                    })
                    .collect::<Vec<_>>();

                let inner_indice = ::rand::distributions::Range::new(0, database.0[id]).sample(&mut rng);
                let range_indice = order.iter().enumerate().find(|(_, &i)| i == id).unwrap().0;
                let mut indice = inner_indice + start_range[range_indice];

                indice = (indice as isize + distribution.sample(&mut rng).round() as isize).max(0) as usize;

                let final_range = start_range.iter()
                    .take_while(|&&start| indice >= start)
                    .count() - 1;

                let final_value = order[final_range];
                match final_value {
                    #(#variants_id3 => _MutateSelfShortCut::#variants3,)*
                    _ => unreachable!(),
                }
            }
        }
    };

    expanded.into()
}
