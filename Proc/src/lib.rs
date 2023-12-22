use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn style_interpolator(_args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as ItemStruct);
    match &parsed.fields {
        Fields::Named(fields) => {
            let field_items: Vec<_> = fields.named.iter().map(|f| {
                let ident = &f.ident;
                let ty = &f.ty;
                let vis = &f.vis;
                quote! {
                    #vis #ident: #ty,
                }
            }).collect();

            let field_idents: Vec<_> = fields.named.iter().filter_map(|f| f.ident.clone()).collect();

            let ts = quote! {
                pub struct GuiStyle {
                    #(#field_items)*,
                }

                impl Interpolator<GuiStyle> for GuiStyle {
                    fn interpolate(&self, t: GuiStyle, start: GuiStyle, end: GuiStyle, progress: f32, easing: Easing) -> GuiStyle {
                        GuiStyle {
                            #(#field_idents: self.#field_idents.interpolate(start.#field_idents, end.#field_idents, progress, easing),)*
                        }
                    }
                }
            };
            TokenStream::from(ts)
        },
        _ => { panic!("Macro was not called on GuiStyle") }
    }
}