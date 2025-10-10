use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, LitStr};

pub fn resolve_resource(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let s = lit.value();
    let s = s
        .strip_prefix('@')
        .expect("resource strings have to start with an @!");
    let (path, name) = s
        .split_once('/')
        .expect("resource strings have to be in the format '@R.type/name'");
    let (r, t) = path
        .split_once('.')
        .expect("resource strings have to be in the format '@R.type/name'");
    let r_ident = Ident::new(r, Span::call_site());
    let fn_ident = Ident::new(&("resolve_".to_owned() + t), Span::call_site());
    let t_ident = Ident::new(t, Span::call_site());
    let name_ident = Ident::new(name, Span::call_site());

    let ts = quote! {#r_ident.#fn_ident(#r_ident.#t_ident.#name_ident)};
    ts.into()
}
