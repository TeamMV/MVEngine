use proc_macro::TokenStream;

mod ecs;
mod r;
mod ui;
mod uix;

#[proc_macro]
pub fn generate_get_components(input: TokenStream) -> TokenStream {
    ecs::generate_get_components(input)
}

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    ui::ui(input)
}

#[proc_macro_attribute]
pub fn uix(attrib: TokenStream, input: TokenStream) -> TokenStream {
    uix::uix(attrib, input)
}

#[proc_macro]
pub fn r(input: TokenStream) -> TokenStream {
    r::r(input)
}