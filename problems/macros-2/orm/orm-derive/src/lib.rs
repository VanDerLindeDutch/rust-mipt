#![forbid(unsafe_code)]
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Type, TypePath};

#[proc_macro_derive(Object, attributes(table_name, column_name))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    struct StructField<'a>{
        name: &'a Option<Ident>,
        ty: &'a TypePath,
        attrs: &'a Vec<Attribute>
    }

    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let table_name = ast.attrs.iter().find(|x|{
        x.path().segments.first().map_or(false, |x|{
            x.ident == "table_name"
        })
    });

    let table_name = match table_name {
        None => {
            name.to_string()
        }
        Some(v) => {
            let lit: syn::LitStr = v.parse_args().unwrap();
            lit.value()
        }
    };
    let struct_ = match ast.data {
        Data::Struct(s) => {s}
        _ => panic!()
    };
    let mut data_types = vec![];
    match struct_.fields {
        Fields::Named(ref fields) => {
            fields.named.iter().for_each(|x| {
               match &x.ty {
                   Type::Path(p) => {

                       data_types.push(StructField{ name: &x.ident, ty: p, attrs: &x.attrs });
                   },
                   _ => panic!("incorrect type")
               }
            });
        }
        Fields::Unit=>{}
        _ => panic!("struct fields must be named")
    }
    let fields: Vec<_> = data_types.iter().map(|x|{
        let (data_name, data_type, attrs) = (x.name, x.ty, x.attrs);
        let column_name = attrs.iter().find(|x|{
            x.path().segments.first().map_or(false, |x|{
                x.ident == "column_name"
            })
        });

        let column_name = match column_name {
            None => {
                data_name.as_ref().unwrap().to_string()
            }
            Some(v) => {
                let lit: syn::LitStr = v.parse_args().unwrap();
                lit.value()
            }
        };
        quote! {
            fields.push(::orm::object::SchemaField{
                    name: stringify!(#data_name),
                    column_name: #column_name,
                    field_type: ::orm::data::DataType::from::<#data_type>(),
            });
        }
    }).collect();
    let values: Vec<_> = data_types.iter().map(|x|{
        let (data_name, data_type) = (x.name, x.ty);
        quote! {
            values.push(::orm::data::Value::from_data_type(&self.#data_name as &dyn Any, ::orm::data::DataType::from::<#data_type>()));
        }
    }).collect();

    let struct_from_schema: Vec<_> = data_types.iter().enumerate().map(|x|{
        let (data_name, data_type, i) = (x.1.name, x.1.ty, x.0);
        quote! {
            #data_name: <#data_type>::from(row.get(#i).unwrap()),
        }
    }).collect();
    let gen = quote! {
        impl Object for #name {
            fn get_schema(&self)->::orm::object::Schema {
                let mut fields = vec![];
                #(#fields)*
                ::orm::object::Schema{
                    type_name: stringify!(#name),
                    table_name: #table_name,
                    fields: fields,
                }
            }

            fn get_values(&self) -> ::orm::storage::Row {
                let mut values = vec![];
                #(#values)*
                values
            }
            fn get_s()->::orm::object::Schema {
                let mut fields = vec![];
                #(#fields)*
                ::orm::object::Schema{
                    type_name: stringify!(#name),
                    table_name: #table_name,
                    fields: fields,
                }
            }

            fn from_schema(schema: ::orm::object::Schema,row: ::orm::storage::Row) -> Self{
                Self{
                    #(#struct_from_schema)*
                }
            }
        }
    };
    proc_macro::TokenStream::from(gen)



}

// TODO: your code goes here.
