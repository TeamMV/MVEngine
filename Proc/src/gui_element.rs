use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use std::str::FromStr;
use syn::token::Comma;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Generics};

const PRIMITIVES: [&str; 15] = [
    "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
    "f32", "f64",
];

const ARGS: [(&str, &str, bool); 16] = [
    ("id", "String", true),
    ("x", "i32", true),
    ("y", "i32", true),
    ("border_x", "i32", true),
    ("border_y", "i32", true),
    ("content_x", "i32", true),
    ("content_y", "i32", true),
    ("style", "Style", true),
    ("parent", "Option<Arc<dyn GuiElement>>", true),
    ("resolve_context", "ResCon", true),
    ("content_width", "i32", true),
    ("content_height", "i32", true),
    ("bounding_width", "i32", true),
    ("bounding_height", "i32", true),
    ("width", "i32", true),
    ("height", "i32", true),
];

pub fn gui_element(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => {
                let vis = input.vis;
                let name = input.ident;
                let generics = input.generics;
                let attrs = input.attrs;
                let mut fields = fields.named;

                if !fields.trailing_punct() && !fields.is_empty() {
                    fields.push_punct(Comma::default());
                }

                let extras = ARGS.iter().map(|(name, ty, _)| {
                    let a = format!("{}: {},", name, ty);
                    proc_macro2::TokenStream::from_str(&a).unwrap()
                });

                let implementation = gen_impl(&name, &generics);

                return quote! {
                    #( #attrs )*
                    #vis struct #name #generics {
                        #fields
                        #( #extras )*
                    }

                    #implementation
                }
                .into();
            }
            _ => panic!("#[gui_element] can only be used on a named struct."),
        },
        _ => panic!("#[gui_element] can only be used on a named struct."),
    }
}

fn gen_impl(name: &Ident, generics: &Generics) -> proc_macro2::TokenStream {
    let (ig, tg, wc) = generics.split_for_impl();
    let functions = ARGS.iter().map(|(name, ty, mutable)| {
        let a = if ty == &"String" {
            format!("fn {name}(&self) -> &str {{ &self.{name}}} {}", if *mutable {
                    format!("fn set_{name}(&mut self, {name}: String) {{ self.{name} = {name} }}")
                } else {
                    String::new()
                }
            )
        } else if PRIMITIVES.contains(ty) {
            format!("fn {name}(&self) -> {ty} {{ self.{name} }} {}",
                if *mutable {
                    format!("fn set_{name}(&mut self, {name}: {ty}) {{ self.{name} = {name} }}")
                } else {
                    String::new()
                }
            )
        } else if ty.starts_with("Option<Arc<") || ty.starts_with("Arc<") {
            format!("fn {name}(&self) -> {ty} {{ self.{name}.clone() }} {}",
                if *mutable {
                    format!("fn set_{name}(&mut self, {name}: {ty}) {{ self.{name} = {name} }}")
                } else {
                    String::new()
                }
            )
        } else if ty.starts_with("Option<") {
            let ty1 = "Option<";
            let ty2 = ty.split_once("<").unwrap().1;
            if PRIMITIVES.contains(&ty2.split_once(">").unwrap().0) {
                format!("fn {name}(&self) -> {ty} {{ self.{name}.clone() }} {}",
                    if *mutable {
                        format!("fn set_{name}(&mut self, {name}: {ty}) {{ self.{name} = {name} }}")
                    } else {
                        String::new()
                    }
                )
            } else {
                format!(
                    "fn {name}(&self) -> {ty1}&{ty2} {{ self.{name}.as_ref() }} {}",
                    if *mutable {
                        format!("fn {name}_mut(&mut self) -> {ty1}&mut {ty2} {{ self.{name}.as_mut() }}")
                    } else {
                        String::new()
                    }
                )
            }
        } else {
            format!(
                "fn {name}(&self) -> &{ty} {{ &self.{name} }} {}",
                if *mutable {
                    format!("fn {name}_mut(&mut self) -> &mut {ty} {{ &mut self.{name} }}")
                } else {
                    String::new()
                }
            )
        };
        proc_macro2::TokenStream::from_str(&a).unwrap()
    });
    quote! {
        impl #ig GuiElement for #name #tg #wc {
            #( #functions )*
        }
    }
}

pub fn gui_element_trait() -> TokenStream {
    let functions = ARGS.iter().map(|(name, ty, mutable)| {
        let a = if ty == &"String" {
            format!(
                "fn {name}(&self) -> &str; {}",
                if *mutable {
                    format!("fn set_{name}(&mut self, {name}: String);")
                } else {
                    String::new()
                }
            )
        } else if PRIMITIVES.contains(ty) {
            format!(
                "fn {name}(&self) -> {ty}; {}",
                if *mutable {
                    format!("fn set_{name}(&mut self, {name}: {ty});")
                } else {
                    String::new()
                }
            )
        } else if ty.starts_with("Option<Arc<") || ty.starts_with("Arc<") {
            format!(
                "fn {name}(&self) -> {ty}; {}",
                if *mutable {
                    format!("fn set_{name}(&mut self, {name}: {ty});")
                } else {
                    String::new()
                }
            )
        } else if ty.starts_with("Option<") {
            let ty1 = "Option<";
            let ty2 = ty.split_once("<").unwrap().1;
            if PRIMITIVES.contains(&ty2.split_once(">").unwrap().0) {
                format!(
                    "fn {name}(&self) -> {ty}; {}",
                    if *mutable {
                        format!("fn set_{name}(&mut self, {name}: {ty});")
                    } else {
                        String::new()
                    }
                )
            } else {
                format!(
                    "fn {name}(&self) -> {ty1}&{ty2}; {}",
                    if *mutable {
                        format!("fn {name}_mut(&mut self) -> {ty1}&mut {ty2};")
                    } else {
                        String::new()
                    }
                )
            }
        } else {
            format!(
                "fn {name}(&self) -> &{ty}; {}",
                if *mutable {
                    format!("fn {name}_mut(&mut self) -> &mut {ty};")
                } else {
                    String::new()
                }
            )
        };
        proc_macro2::TokenStream::from_str(&a).unwrap()
    });
    quote! {
        pub trait GuiElement: GuiElementCallbacks {
            #( #functions )*
        }
    }
    .into()
}
