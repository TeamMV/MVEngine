use proc_macro::{TokenStream};
use proc_macro2::{Ident, Span};
use quote::quote;

pub fn multiline_str_into(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let ((mac, s)) = input.split_once(',').expect("Illegal setup");
    let s = s.trim();
    let s = s.strip_prefix('{').expect("Illegal string. use {} to encaspulate it");
    let s = s.strip_suffix('}').expect("Illegal string. use {} to encaspulate it");
    let ident = Ident::new(mac, Span::call_site());
    let ts = quote! {#ident!(#s)};
    ts.into()
}