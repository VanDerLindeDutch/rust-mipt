#![forbid(unsafe_code)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type, TypePath};
#[proc_macro_derive(Object, attributes(table_name, column_name))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    struct StructField<'a>{
        name: &'a Option<Ident>,
        ty: &'a TypePath
    }

    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
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
                       data_types.push(StructField{ name: &x.ident, ty: p });
                   },
                   _ => panic!("incorrect type")
               }
            });
        }
        _ => panic!("struct fields must be named")
    }
    let fields: Vec<_> = data_types.iter().map(|x|{
        let (data_name, data_type) = (x.name, x.ty);
        quote! {
            fields.append(SchemaField{
                    name: stringify!(#data_name),
                    column_name: stringify!(#data_name),
                    field_type: FieldType::from::<#data_type>(),
                });
        }
    }).collect();
    let gen = quote! {
        impl Object for #name {
            fn get_schema(&self)->Schema {
                let fields = vec![];
                #(#fields)*
                Schema{
                    type_name: stringify!(#name),
                    table_name: stringify!(#name),
                    fields: fields,
                }
            }
        }
    };
    proc_macro::TokenStream::from(gen)



}

// TODO: your code goes here.
