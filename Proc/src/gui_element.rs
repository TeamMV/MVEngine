use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use std::str::FromStr;
use syn::parse_quote;
use syn::token::Comma;
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed, Generics};

const PRIMITIVES: [&str; 15] = [
    "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
    "f32", "f64",
];

const ARGS: [(&str, &str); 7] = [
    ("x", "String"),
    ("y", "Option<u32>"),
    ("z", "i32"),
    ("v", "Arc<String>"),
    ("w", "Option<Arc<String>>"),
    ("a", "Option<String>"),
    ("b", "Style"),
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

                let extras = ARGS.iter().map(|(name, ty)| {
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
    let functions = ARGS.iter().map(|(name, ty)| {
        let a = if ty == &"String" {
            format!(
                r#"
                fn {name}(&self) -> &str {{ &self.{name} }}
                fn set_{name}(&mut self, {name}: String) {{ self.{name} = {name} }}
            "#
            )
        } else if PRIMITIVES.contains(ty) {
            format!(
                r#"
                fn {name}(&self) -> {ty} {{ self.{name} }}
                fn set_{name}(&mut self, {name}: {ty}) {{ self.{name} = {name} }}
            "#
            )
        } else if ty.starts_with("Option<Arc<") || ty.starts_with("Arc<") {
            format!(
                r#"
                fn {name}(&self) -> {ty} {{ self.{name}.clone() }}
            "#
            )
        } else if ty.starts_with("Option<") {
            let ty1 = "Option<";
            let ty2 = ty.split_once("<").unwrap().1;
            if PRIMITIVES.contains(&ty2.split_once(">").unwrap().0) {
                format!(
                    r#"
                    fn {name}(&self) -> {ty} {{ self.{name}.clone() }}
                    fn set_{name}(&mut self, {name}: {ty}) {{ self.{name} = {name} }}
                "#
                )
            } else {
                format!(
                    r#"
                    fn {name}(&self) -> {ty1}&{ty2} {{ self.{name}.as_ref() }}
                    fn {name}_mut(&mut self) -> {ty1}&mut {ty2} {{ self.{name}.as_mut() }}
                "#
                )
            }
        } else {
            format!(
                r#"
                fn {name}(&self) -> &{ty} {{ &self.{name} }}
                fn {name}_mut(&mut self) -> &mut {ty} {{ &mut self.{name} }}
            "#
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
    let functions = ARGS.iter().map(|(name, ty)| {
        let a = if ty == &"String" {
            format!(
                r#"
                fn {name}(&self) -> &str;
                fn set_{name}(&mut self, {name}: String);
            "#
            )
        } else if PRIMITIVES.contains(ty) {
            format!(
                r#"
                fn {name}(&self) -> {ty};
                fn set_{name}(&mut self, {name}: {ty});
            "#
            )
        } else if ty.starts_with("Option<Arc<") || ty.starts_with("Arc<") {
            format!(
                r#"
                fn {name}(&self) -> {ty};
            "#
            )
        } else if ty.starts_with("Option<") {
            let ty1 = "Option<";
            let ty2 = ty.split_once("<").unwrap().1;
            if PRIMITIVES.contains(&ty2.split_once(">").unwrap().0) {
                format!(
                    r#"
                    fn {name}(&self) -> {ty} {{ self.{name}.clone() }}
                    fn set_{name}(&mut self, {name}: {ty}) {{ self.{name} = {name} }}
                "#
                )
            } else {
                format!(
                    r#"
                    fn {name}(&self) -> {ty1}&{ty2} {{ self.{name}.as_ref() }}
                    fn {name}_mut(&mut self) -> {ty1}&mut {ty2} {{ self.{name}.as_mut() }}
                "#
                )
            }
        } else {
            format!(
                r#"
                fn {name}(&self) -> &{ty};
                fn {name}_mut(&mut self) -> &mut {ty};
            "#
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
