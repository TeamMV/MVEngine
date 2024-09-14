use proc_macro::TokenStream;

mod graphics_item;

#[proc_macro_attribute]
pub fn graphics_item(attrib: TokenStream, input: TokenStream) -> TokenStream {
    graphics_item::graphics_item(attrib, input)
}
