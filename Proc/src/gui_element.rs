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

const ARGS: [(&str, &str, bool); 20] = [
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
    ("origin_x", "i32", true),
    ("origin_y", "i32", true),
    ("paddings", "Sides", true),
    ("margins", "Sides", true),
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

                    impl DrawComponentBody for #name {}
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

        impl #ig PartialEq for #name #tg #wc {
            fn eq(&self, other: &Self) -> bool {
                self.id() == other.id()
            }
        }

        impl #ig Eq for #name #tg #wc {}
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
        pub trait GuiElement: GuiElementCallbacks + DrawComponentBody {
            #( #functions )*

            fn compute_values(&mut self, ctx: &mut DrawContext2D) {
                self.resolve_context_mut().set_dpi(ctx.dpi());
                let mut paddings: [i32; 4] = [0; 4];
                paddings[0] = resolve!(self, padding.top);
                paddings[1] = resolve!(self, padding.bottom);
                paddings[2] = resolve!(self, padding.left);
                paddings[3] = resolve!(self, padding.right);

                let mut margins: [i32; 4] = [0; 4];
                margins[0] = resolve!(self, margin.top);
                margins[1] = resolve!(self, margin.bottom);
                margins[2] = resolve!(self, margin.left);
                margins[3] = resolve!(self, margin.right);

                self.paddings_mut().copy_slice(&paddings);
                self.margins_mut().copy_slice(&margins);

                if self.style().width.is_set() {
                    self.set_content_width(resolve!(self, width));
                }

                if self.style().height.is_set() {
                    self.set_content_height(resolve!(self, height));
                }

                let bounding_width = self.content_width() + paddings[2] + paddings[3];
                let width = bounding_width + margins[2] + margins[3];
                let bounding_height = self.content_height() + paddings[0] + paddings[1];
                let height = bounding_width + margins[0] + margins[1];

                self.set_bounding_width(bounding_width);
                self.set_bounding_height(bounding_height);
                self.set_width(width);
                self.set_height(height);

                let origin = resolve!(self, origin);
                let position = resolve!(self, position);

                let mut x: i32 = 0;
                let mut y: i32 = 0;
                if position == Position::Absolute {
                    x = resolve!(self, x);
                    y = resolve!(self, y);

                    if let Origin::Custom(ox, oy) = origin {
                        self.set_x(x + ox);
                        self.set_y(y + oy);
                    } else if let Origin::Center = origin {
                        self.set_x(x + self.width() / 2);
                        self.set_y(y + self.height() / 2);
                    } else {
                        self.set_x(origin.is_right().yn(x - self.width(), x));
                        self.set_y(origin.is_left().yn(y - self.height(), y));
                    }
                } else if position == Position::Relative {
                    x = self.x();
                    y = self.y();
                }

                self.set_border_x(x + margins[2]);
                self.set_border_y(y + margins[1]);
                self.set_content_x(self.border_x() + paddings[2]);
                self.set_content_y(self.border_y() + paddings[1]);

                let rot_center = resolve!(self, rotation_origin);
                if let Origin::Custom(ox, oy) = rot_center {
                    self.set_origin_x(ox);
                    self.set_origin_y(oy);
                } else {
                    if rot_center == Origin::Center {
                        self.set_origin_x(x + width / 2);
                        self.set_origin_y(y + height / 2);
                    }
                    if rot_center == Origin::BottomLeft {
                        self.set_origin_x(x);
                        self.set_origin_y(y);
                    }
                    if rot_center == Origin::BottomRight {
                        self.set_origin_x(x + width);
                        self.set_origin_y(y);
                    }
                    if rot_center == Origin::TopLeft {
                        self.set_origin_x(x);
                        self.set_origin_y(y + height);
                    }
                    if rot_center == Origin::TopRight {
                        self.set_origin_x(x + width);
                        self.set_origin_y(y + height);
                    }
                }
            }
        }
    }
    .into()
}
