use proc_macro::TokenStream;
use std::env::current_dir;
use std::fs;
use proc_macro2::{Ident, Span};
use quote::quote;
use ui_parsing::xml::{parse_rsx, Entity, XmlValue};

pub fn r(input: TokenStream) -> TokenStream {
    let content = input.to_string();
    let rsx = parse_rsx(content).unwrap();
    if rsx.name().as_str() != "resources" {
        panic!("Expected root tag to be resources!");
    }

    let mut struct_name = "R";
    let mut cdir = "./".to_string();

    if let Some(value) = rsx.get_attrib("structName") {
        if let XmlValue::Str(s) = value {
            struct_name = s.as_str();
        }
    }
    if let Some(value) = rsx.get_attrib("cdir") {
        if let XmlValue::Str(s) = value {
            cdir = s.clone();
        }
    }

    let mut colors: Vec<(String, String)> = vec![];
    let mut shapes: Vec<(String, String)> = vec![];

    if let Some(inner) = rsx.inner() {
        if let XmlValue::Entities(children) = inner {
            for child in children {
                let name = child.name();
                let ty = name.as_str();
                match ty {
                    "colors" => {
                        if let Some(inner2) = child.inner() {
                            if let XmlValue::Entities(children2) = inner2 {
                                for entity in children2 {
                                    colors.push(parse_color(entity));
                                }
                            }
                        }
                    },
                    "shapes" => {
                        if let Some(inner2) = child.inner() {
                            if let XmlValue::Entities(children2) = inner2 {
                                for entity in children2 {
                                    shapes.push(parse_shape(entity));
                                }
                            }
                        }
                    },

                    _ => panic!("Invalid resource type {ty}")
                }
            }
        }
    }

    let mut colors_ts = quote! {};
    let mut colors_gens_ts = quote! {};
    let mut colors_arr_ts = quote! {};
    let mut index = 0usize;
    for color in colors {
        let name_ident = Ident::new(color.0.as_str(), Span::call_site());
        let lit = color.1;
        let generator = quote! {
            #name_ident: #index + mvengine_ui::res::CR,
        };

        colors_ts.extend(quote! { pub #name_ident: usize, });
        colors_gens_ts.extend(generator);
        colors_arr_ts.extend(quote! { mvcore::color::parse::parse_color(#lit).unwrap(), });
        index += 1;
    }

    let mut shapes_ts = quote! {};
    let mut shapes_gens_ts = quote! {};
    let mut shapes_arr_ts = quote! {};
    let mut index = 0usize;
    for shape in shapes {
        let name_ident = Ident::new(shape.0.as_str(), Span::call_site());
        let mut lit = cdir.clone();
        lit.push_str(shape.1.as_str());
        let generator = quote! {
            #name_ident: #index + mvengine_ui::res::CR,
        };

        shapes_ts.extend(quote! { pub #name_ident: usize, });
        shapes_gens_ts.extend(generator);
        shapes_arr_ts.extend(quote! {
            {
                let ast = mvengine_ui::render::shapes::ShapeParser::parse(include_str!(#lit)).unwrap();
                mvengine_ui::render::shapes::shape_gen::ShapeGenerator::generate(ast).unwrap()
            },
        });
        index += 1;
    }



    let r_ident = Ident::new(struct_name, Span::call_site());

    let pm1 = quote! {
        mvutils::lazy! {
            pub static #r_ident: #r_ident = #r_ident {
                color: R_color {
                    colors: [#colors_arr_ts],
                    #colors_gens_ts
                },
                shape: R_shape {
                    shapes: [#shapes_arr_ts],
                    #shapes_gens_ts
                }
            };
        }

        pub struct R_color {
            colors: [mvcore::color::RgbColor; #index],
            #colors_ts
        }

        pub struct R_shape {
            shapes: [mvengine_ui::render::ctx::DrawShape; #index],
            #shapes_ts
        }

        pub struct #r_ident {
            //pub mv: mvengine_ui::res::MVR,
            pub color: R_color,
            pub shape: R_shape,
        }

        impl #r_ident {
            pub fn _resolve<T>(&self, id: usize) -> Option<&T> {
                let ty = TypeId::of::<T>();
                if ty == TypeId::of::<mvcore::color::RgbColor>() {
                    if id >= mvengine_ui::res::CR {
                        return self.color.colors.get(id - mvengine_ui::res::CR);
                    } else {
                        return None;
                    }
                }
                if ty == TypeId::of::<mvengine_ui::render::ctx::DrawShape>() {
                    if id >= mvengine_ui::res::CR {
                        return self.shape.shapes.get(id - mvengine_ui::res::CR);
                    } else {
                        return None;
                    }
                }

                None
            }
        }

        impl mvengine_ui::context::UiResources for #r_ident {
            fn resolve_color(&self, id: usize) -> Option<&mvcore::color::RgbColor> {
                self._resolve::<mvcore::color::RgbColor>(id)
            }

            fn resolve_shape(&self, id: usize) -> Option<&mvengine_ui::render::ctx::DrawShape> {
                self._resolve::<mvengine_ui::render::ctx::DrawShape>(id)
            }
        }
    };
    pm1.into()
}

fn parse_color(entity: &Entity) -> (String, String){
    if entity.name().as_str() != "color" {
        panic!("Color resource must be named color, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("val") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    return (name_s.clone(), val_s.clone());
                }
            }
            panic!("Code blocks are not supported in color!")
        } else {
            panic!("Expected a 'name' attribute on color!")
        }
    } else {
        panic!("Expected a 'val' attribute on color!")
    }
}

fn parse_shape(entity: &Entity) -> (String, String){
    if entity.name().as_str() != "shape" {
        panic!("Shape resource must be named shape, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    return (name_s.clone(), val_s.clone());
                }
            }
            panic!("Code blocks are not supported in shape!")
        } else {
            panic!("Expected a 'name' attribute on shape!")
        }
    } else {
        panic!("Expected a 'src' attribute on shape!")
    }
}