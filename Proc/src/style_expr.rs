use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, parse_str, Expr, ExprField, LitStr, Member};
use ui_parsing::style::StyleParser;

pub(crate) fn style_expr(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as LitStr);
    let parsed = StyleParser::parse_expr(&inp.value());
    
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

        if value.starts_with('{') {
            //{} means the inner part should be kept as rust code to initialize the field
            let mut inner = String::new();
            let mut paren_depth = 0;
            for c in value.chars() {
                if c == '}' && paren_depth == 0 { 
                    break;
                } else {
                    if c == '{' { 
                        paren_depth += 1;
                    } else if c == '}' {
                        paren_depth -= 1;
                    }
                    inner.push(c);
                }
            }
            let value: Expr = parse_str(&inner).expect("Cannot parse inner rust code as expression! Note: values inside {} will be treated as rust code and does not call Parseable::parse.");

            mods.extend(quote! {
                modify_style!(#accessor_expr = mvengine::ui::styles::UiValue::Just(core::convert::Into::into(#value)));
            });
        } else if value.starts_with('@') {
            //@ means it's a resource
            mods.extend(quote! {
                {
                    let r = resolve_resource!(#value).expect("Cannot find resource!");
                    modify_style!(#accessor_expr = mvengine::ui::styles::UiValue::Just(core::convert::Into::into(r.clone())));
                }
            });
        } else {
            mods.extend(quote! {
                modify_style!(#accessor_expr = mvengine::ui::styles::Parseable::parse(#value).expect("Cannot parse style"));
            });
        }
    }
    
    let base_ts = quote! {
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