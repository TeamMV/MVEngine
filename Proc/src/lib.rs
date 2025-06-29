use proc_macro::TokenStream;

mod ecs;
mod r;
mod ui;
mod uix;
mod listener;
mod style_expr;
mod multiline_str_into;
mod resolve_resource;
mod msfx_function;

#[proc_macro]
pub fn generate_get_components(input: TokenStream) -> TokenStream {
    ecs::generate_get_components(input)
}

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    ui::ui(input)
}

#[proc_macro]
pub fn style_expr(input: TokenStream) -> TokenStream {
    style_expr::style_expr(input)
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
pub fn msfx_fn(_: TokenStream, body: TokenStream) -> TokenStream {
    msfx_function::msfx_fn(body)
}