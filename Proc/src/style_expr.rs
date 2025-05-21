use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_str, Expr, ExprField, Member};
use ui_parsing::style::StyleParser;

pub(crate) fn style_expr(input: TokenStream) -> TokenStream {
    let parsed = StyleParser::parse_expr(&input.to_string());
    
    let mut mods = quote! {};
    for entry in parsed.entries {
        let accessor = entry.accessor;
        
        let base: Expr = parse_str("style").unwrap();
        let accessor_expr = accessor
            .split('.')
            .map(|s| Ident::new(s, Span::call_site()))
            .fold(base, |acc, ident| {
                Expr::Field(ExprField {
                    attrs: vec![],
                    base: Box::new(acc),
                    dot_token: Default::default(),
                    member: Member::Named(ident),
                })
            });

        let value = &entry.value;
        mods.extend(quote! {
            modify_style!(#accessor_expr = Resolve::try_from(#value.to_string()));
        });
    }
    
    let mut base_ts = quote! {
        let mut style = mvengine::ui::styles::UiStyle::default();
        #mods
    };
    
    let ts = quote! {
        {
            #base_ts
            style
        }
    };
    ts.into()
}