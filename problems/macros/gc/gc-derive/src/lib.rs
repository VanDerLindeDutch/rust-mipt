#![forbid(unsafe_code)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, PathArguments, Type, TypePath};

#[proc_macro_derive(Scan)]
pub fn derive_scan(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let struct_ = match ast.data {
        Data::Struct(s) => {s}
        _ => panic!()
    };
    let mut to_scan = vec![];
    match struct_.fields {
        Fields::Named(ref fields) => {
            fields.named.iter().for_each(|x|{
                match &x.ty {
                    Type::Path(p) => {
                        if contains_gc(p) {
                            to_scan.push(&x.ident)
                        }

                    }
                    _ => {}
                }
            })
        }
        _ => {}
    }
    let gen = quote! {
        impl Scan for #name {
            fn scan(&self, prev: &mut HashSet<usize>)->Vec<Gc<dyn Scan>>{
                let mut out = vec![];
                #( out.append(&mut self.#to_scan.scan(prev));)*
                out
            }

            fn refers_to(&self)->Vec<Gc<dyn Scan>>{
                let mut out = vec![];
                #( out.append(&mut self.#to_scan.refers_to());)*
                out
            }
        }
        };
    proc_macro::TokenStream::from(gen)
}
fn contains_gc(path: &TypePath)->bool {
    let gc_type = String::from("Gc");
    let mut segments = path.path.segments.iter();
    segments.any(|x| {
        let args = &x.arguments;
        let rest = match args {
            PathArguments::None => {
                false
            }
            PathArguments::AngleBracketed(t) => {
                if let syn::GenericArgument::Type(Type::Path(p)) = &t.args[0]  {
                    contains_gc(p)
                }else {
                    false
                }
            }
            PathArguments::Parenthesized(t) => {
                false
            }
        };
        x.ident.to_string().eq(&gc_type) || rest
    })
}
