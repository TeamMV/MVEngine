use proc_macro::TokenStream;

mod resolve;
mod gui_element;

#[proc_macro_attribute]
pub fn gui_element(_: TokenStream, input: TokenStream) -> TokenStream {
    gui_element::gui_element(input)
}