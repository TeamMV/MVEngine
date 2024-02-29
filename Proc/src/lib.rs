use proc_macro::TokenStream;

mod gui_element;

#[proc_macro_attribute]
pub fn gui_element(_: TokenStream, input: TokenStream) -> TokenStream {
    gui_element::gui_element(input)
}

#[proc_macro]
pub fn gui_element_trait(_: TokenStream) -> TokenStream {
    gui_element::gui_element_trait()
}
