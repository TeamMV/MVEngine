mod ui;
mod uix;
mod drawable;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};

#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    ui::ui(input)
}

#[proc_macro_attribute]
pub fn uix(attrib: TokenStream, input: TokenStream) -> TokenStream {
    uix::uix(attrib, input)
}

#[proc_macro]
pub fn drawable(input: TokenStream) -> TokenStream {
    drawable::drawable(input)
}
