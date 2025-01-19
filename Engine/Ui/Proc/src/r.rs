use proc_macro::TokenStream;
use std::env::current_dir;
use std::fs;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::Parse;
use syn::Path;
use ui_parsing::xml::{parse_rsx, Entity, XmlValue};

pub fn r(input: TokenStream) -> TokenStream {
    let content = input.to_string();
    let rsx = parse_rsx(content).unwrap();
    if rsx.name().as_str() != "resources" {
        panic!("Expected root tag to be resources!");
    }

    let mut struct_name = "R";
    let mut cdir = "./".to_string();
    let mut is_mv = false;

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
    if let Some(value) = rsx.get_attrib("superSecretTagWhichSpecifiesThisIsTheMVResourceStruct") {
        if let XmlValue::Str(s) = value {
            is_mv = s.as_str() == "andItsSuperSecretValue";
        }
    }

    let mut colors: Vec<(String, String)> = vec![];
    let mut shapes: Vec<(String, String)> = vec![];
    let mut adaptives: Vec<(String, String)> = vec![];
    let mut textures: Vec<(String, String)> = vec![];

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
                    "adaptives" => {
                        if let Some(inner2) = child.inner() {
                            if let XmlValue::Entities(children2) = inner2 {
                                for entity in children2 {
                                    adaptives.push(parse_adaptive(entity));
                                }
                            }
                        }
                    },
                    "textures" => {
                        if let Some(inner2) = child.inner() {
                            if let XmlValue::Entities(children2) = inner2 {
                                for entity in children2 {
                                    textures.push(parse_texture(entity));
                                }
                            }
                        }
                    },

                    _ => panic!("Invalid resource type {ty}")
                }
            }
        }
    }

    //code generation

    let r_ident = Ident::new(struct_name, Span::call_site());

    let mv_ts = if !is_mv {
        quote! { pub mv: &'static mvengine_ui::res::MVR, }
    } else {
        quote! {}
    };

    let mv_gen_ts = if !is_mv {
        quote! { mv: &mvengine_ui::res::MVR, }
    } else {
        quote! {}
    };

    let mut res_gens_ts = quote! {};
    let mut r_fields_ts = quote! {};
    let (color_struct_ts, color_resolve_fn_ts) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "color",
        "mvcore::color::RgbColor",
        colors,
        |lit| { quote! { mvcore::color::parse::parse_color(#lit).unwrap(), } }
    );
    let (shape_struct_ts, shape_resolve_fn_ts) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "shape",
        "mvengine_ui::render::ctx::DrawShape",
        shapes,
        |lit| {
            let path = get_src(cdir.as_str(), lit);
            quote! {
                {
                    let ast = mvengine_ui::render::shapes::ShapeParser::parse(include_str!(#path)).unwrap();
                    mvengine_ui::render::shapes::shape_gen::ShapeGenerator::generate(ast).unwrap()
                },
            }
        }
    );
    let (adaptive_struct_ts, adaptive_resolve_fn_ts) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "adaptive",
        "mvengine_ui::render::adaptive::AdaptiveShape",
        adaptives,
        |lit| {
            let path = get_src(cdir.as_str(), lit);
            quote! {
                {
                    let ast = mvengine_ui::render::shapes::ShapeParser::parse(include_str!(#path)).unwrap();
                    mvengine_ui::render::shapes::shape_gen::ShapeGenerator::generate_adaptive(ast).unwrap()
                },
            }
        }
    );
    let (texture_struct_ts, texture_resolve_fn_ts) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "texture",
        "mvcore::render::texture::Texture",
        textures,
        |lit| {
            let path = get_src(cdir.as_str(), lit);
            quote! {
                {
                    let handle = manager.include_asset(include_bytes!(#path), mvcore::asset::asset::AssetType::Texture);
                    handle.load(|asset, idk| {});
                    handle.wait();
                    let asset = handle.get();
                    let tex = asset.as_texture().expect(&format!("Cannot load texture '{}'", #path));
                    tex
                },
            }
        }
    );


    let init_fn_ts = if !is_mv {
        quote! {
            mvengine_ui::res::MVR::_initialize(manager.clone());
            #r_ident.create(|| #r_ident {
                #mv_gen_ts
                #res_gens_ts
            });
        }
    } else {
        quote! {
            #r_ident.create(|| #r_ident {
                #mv_gen_ts
                #res_gens_ts
            });
        }
    };

    let pm1 = quote! {
        mvutils::lazy! {
            pub static #r_ident: mvutils::once::CreateOnce<#r_ident> = mvutils::once::CreateOnce::new();
        }

        pub struct #r_ident {
            #mv_ts
            #r_fields_ts
        }

        #color_struct_ts
        #shape_struct_ts
        #adaptive_struct_ts
        #texture_struct_ts

        unsafe impl Send for #r_ident {}
        unsafe impl Sync for #r_ident {}

        impl #r_ident {
            pub fn initialize(device: mvcore::render::backend::device::Device) {
                let manager = mvcore::asset::manager::AssetManager::new(device, 1, 1);
                Self::_initialize(manager);
            }

            pub fn _initialize(manager: std::sync::Arc<mvcore::asset::manager::AssetManager>) {
                #init_fn_ts
            }
        }

        impl mvengine_ui::context::UiResources for #r_ident {
            #color_resolve_fn_ts
            #shape_resolve_fn_ts
            #adaptive_resolve_fn_ts
            #texture_resolve_fn_ts
        }
    };
    pm1.into()
}

fn get_src(cdir: &str, given: &str) -> String {
    if given.starts_with(':') {
        return given[1..].to_string();
    }
    format!("{cdir}{given}")
}

type TS = proc_macro2::TokenStream;

fn extent_resource<F>(
    is_mv: bool,
    r_field_tokens: &mut TS,
    r_field_gens_tokens: &mut TS,
    struct_name: &str,
    res_name: &str,
    res_type: &str,
    parsed: Vec<(String, String)>,
    creator: F
) -> (TS, TS) where F: Fn(&str) -> TS {
    let struct_ident = Ident::new(&format!("{struct_name}_{res_name}"), Span::call_site());
    let field_ident = Ident::new(res_name, Span::call_site());
    let res_arr_ident = Ident::new(&format!("{res_name}_arr"), Span::call_site());
    let type_path: Path = syn::parse_str(res_type).unwrap();

    let mut arr_ts = quote! {};
    let mut gens_ts = quote! {};
    let mut res_fields_ts = quote! {};
    let mut index = 0usize;
    for entry in parsed {
        let ident = Ident::new(entry.0.as_str(), Span::call_site());
        if !is_mv {
            gens_ts.extend(quote! {
                #ident: #index + mvengine_ui::res::CR,
            });
        } else {
            gens_ts.extend(quote! {
                #ident: #index,
            });
        }
        res_fields_ts.extend(quote! {
            pub #ident: usize,
        });
        index += 1;
        let arr_entry = creator(entry.1.as_str());
        arr_ts.extend(arr_entry);
    }

    r_field_tokens.extend(quote! {
        pub #field_ident: #struct_ident,
    });

    r_field_gens_tokens.extend(quote! {
        #field_ident: #struct_ident {
            #res_arr_ident: [#arr_ts],
            #gens_ts
        },
    });

    let fn_ident = Ident::new(&format!("resolve_{res_name}"), Span::call_site());

    let res_fn_ts = if !is_mv {
        quote! {
            fn #fn_ident(&self, id: usize) -> Option<&#type_path> {
                if id >= mvengine_ui::res::CR {
                    self.#field_ident.#res_arr_ident.get(id - mvengine_ui::res::CR)
                } else {
                    self.mv.#fn_ident(id)
                }
            }
        }
    } else {
        quote! {
            fn #fn_ident(&self, id: usize) -> Option<&#type_path> {
                self.#field_ident.#res_arr_ident.get(id)
            }
        }
    };

    (quote! {
        pub struct #struct_ident {
            #res_arr_ident: [#type_path; #index],
            #res_fields_ts
        }
    }, res_fn_ts)
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

fn parse_adaptive(entity: &Entity) -> (String, String){
    if entity.name().as_str() != "adaptive" {
        panic!("Adaptive resource must be named adaptive, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    return (name_s.clone(), val_s.clone());
                }
            }
            panic!("Code blocks are not supported in adaptive!")
        } else {
            panic!("Expected a 'name' attribute on adaptive!")
        }
    } else {
        panic!("Expected a 'src' attribute on adaptive!")
    }
}

fn parse_texture(entity: &Entity) -> (String, String){
    if entity.name().as_str() != "texture" {
        panic!("Texture resource must be named texture, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    return (name_s.clone(), val_s.clone());
                }
            }
            panic!("Code blocks are not supported in texture!")
        } else {
            panic!("Expected a 'name' attribute on texture!")
        }
    } else {
        panic!("Expected a 'src' attribute on texture!")
    }
}