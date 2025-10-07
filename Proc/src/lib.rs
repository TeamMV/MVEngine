use proc_macro::TokenStream;
use quote::quote;

mod ecs;
mod r;
mod ui;
mod uix;
mod listener;
mod style_expr;
mod multiline_str_into;
mod resolve_resource;
mod msfx_function;
mod graphics_item;

#[proc_macro]
pub fn generate_queries(input: TokenStream) -> TokenStream {
    ecs::generate_queries(input)
}

#[proc_macro]
pub fn generate_system_impls(input: TokenStream) -> TokenStream {
    ecs::generate_system_impls(input)
}

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    ui::ui(input)
}

#[proc_macro]
pub fn style_expr(input: TokenStream) -> TokenStream {
    style_expr::style_expr(quote! { mvengine::ui::styles::UiStyle::default() }, input)
}

#[proc_macro]
pub fn style_expr_empty(input: TokenStream) -> TokenStream {
    style_expr::style_expr(quote! { mvengine::ui::styles::EMPTY_STYLE.clone() }, input)
}

#[proc_macro_attribute]
pub fn uix(attrib: TokenStream, input: TokenStream) -> TokenStream {
    uix::uix(attrib, input)
}

#[proc_macro]
pub fn r(input: TokenStream) -> TokenStream {
    r::r(input)
}

#[proc_macro_attribute]
pub fn listener(head: TokenStream, body: TokenStream) -> TokenStream {
    listener::listener(head, body)
}

#[proc_macro]
pub fn multiline_str_into(input: TokenStream) -> TokenStream {
    multiline_str_into::multiline_str_into(input)
}

#[proc_macro]
pub fn resolve_resource(input: TokenStream) -> TokenStream {
    resolve_resource::resolve_resource(input)
}

#[proc_macro_attribute]
pub fn msfx_fn(attr: TokenStream, body: TokenStream) -> TokenStream {
    msfx_function::msfx_fn(attr, body)
}

#[proc_macro_attribute]
pub fn graphics_item(attrib: TokenStream, input: TokenStream) -> TokenStream {
    graphics_item::graphics_item(attrib, input)
}