use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};
use syn::token::Comma;

pub fn gui_element(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(s) => {
            match s.fields {
                Fields::Named(fields) => {
                    let vis = input.vis;
                    let name = input.ident;
                    let generics = input.generics;
                    let attrs = input.attrs;
                    let mut fields = fields.named;

                    if !fields.trailing_punct() {
                        fields.push_punct(Comma::default());
                    }

                    return quote! {
                        #( #attrs )*
                        #vis struct #name #generics {
                            #fields
                            x: u32,
                            y: Option<u32>
                        }
                    }.into();
                },
                _ => panic!("#[gui_element] can only be used on a named struct.")
            }
        }
        _ => panic!("#[gui_element] can only be used on a named struct.")
    }
}