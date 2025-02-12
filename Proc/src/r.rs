use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse, parse_str, Expr, Path};
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
    let mut fonts: Vec<(String, String)> = vec![];
    let mut tilesets: Vec<(String, ParsedTileSet)> = vec![];
    let mut animations: Vec<(String, ParsedAnimation)> = vec![];

    if let Some(inner) = rsx.inner() {
        if let XmlValue::Entities(children) = inner {
            for child in children {
                macro_rules! branch {
                    ($arr:ident, $fun:ident) => {
                        {
                            if let Some(inner2) = child.inner() {
                                if let XmlValue::Entities(children2) = inner2 {
                                    for entity in children2 {
                                        $arr.push($fun(entity));
                                    }
                                }
                            }
                        }
                    };
                }
                let name = child.name();
                let ty = name.as_str();
                match ty {
                    "colors" => branch!(colors, parse_color),
                    "shapes" => branch!(shapes, parse_shape),
                    "adaptives" => branch!(adaptives, parse_adaptive),
                    "textures" => branch!(textures, parse_texture),
                    "fonts" => branch!(fonts, parse_font),
                    "tilesets" => branch!(tilesets, parse_tileset),
                    "animations" => branch!(animations, parse_animation),

                    _ => panic!("Invalid resource type {ty}")
                }
            }
        }
    }

    //code generation

    let r_ident = Ident::new(struct_name, Span::call_site());

    let mv_ts = if !is_mv {
        quote! { pub mv: &'static mvengine::ui::res::MVR, }
    } else {
        quote! {}
    };

    let mv_gen_ts = if !is_mv {
        quote! { mv: &mvengine::ui::res::MVR, }
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
        "mvengine::color::RgbColor",
        colors,
        |lit| { quote! { mvengine::color::parse::parse_color(#lit).unwrap(), } }
    );
    let (shape_struct_ts, shape_resolve_fn_ts) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "shape",
        "mvengine::ui::rendering::ctx::DrawShape",
        shapes,
        |lit| {
            let path = get_src(cdir.as_str(), lit);
            quote! {
                {
                    let ast = mvengine::ui::rendering::shapes::ShapeParser::parse(include_str!(#path)).unwrap();
                    mvengine::ui::rendering::shapes::shape_gen::ShapeGenerator::generate(ast).unwrap()
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
        "mvengine::ui::rendering::adaptive::AdaptiveShape",
        adaptives,
        |lit| {
            let path = get_src(cdir.as_str(), lit);
            quote! {
                {
                    let ast = mvengine::ui::rendering::shapes::ShapeParser::parse(include_str!(#path)).unwrap();
                    mvengine::ui::rendering::shapes::shape_gen::ShapeGenerator::generate_adaptive(ast).unwrap()
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
        "mvengine::rendering::texture::Texture",
        textures,
        |lit| {
            let (src, sampler) = lit.split_once('|').expect("Meta should always contain a colon");
            let path = get_src(cdir.as_str(), src);
            let linear = "linear" == sampler.to_lowercase();

            quote! {
                {
                    let tex = mvengine::rendering::texture::Texture::from_bytes_sampled(include_bytes!(#path), #linear).expect("Cannot load texture!");
                    tex
                },
            }
        }
    );
    let (font_struct_ts, font_resolve_fn_ts) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "font",
        "mvengine::rendering::text::Font",
        fonts,
        |lit| {
            let (src, atlas) = lit.split_once('|').expect("Meta should always contain a colon");
            let path = get_src(cdir.as_str(), src);
            let atlas_path = get_src(cdir.as_str(), atlas);

            let err_message = format!("Cannot load font {path}!");

            quote! {
                {
                    let tex = mvengine::rendering::texture::Texture::from_bytes_sampled(include_bytes!(#atlas_path), true).expect("Cannot load texture!");
                    let font = mvengine::rendering::text::Font::new(tex, include_bytes!(#path)).expect(#err_message);
                    font
                },
            }
        }
    );
    let (tile_struct_ts, tile_resolve_fn_ts) = extend_tiles(&tilesets, &mut r_fields_ts, &mut res_gens_ts, struct_name, is_mv);

    let (tileset_struct_ts, tileset_resolve_fn_ts) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "tileset",
        "mvengine::graphics::tileset::TileSet",
        tilesets,
        |tileset| {
            let atlas = get_src(cdir.as_str(), &tileset.atlas);
            let width = tileset.width;
            let height = tileset.height;
            let count = tileset.count;
            let linear = tileset.linear;

            quote! {
                {
                    let tex = mvengine::rendering::texture::Texture::from_bytes_sampled(include_bytes!(#atlas), #linear).expect("Cannot load texture!");
                    let set = mvengine::graphics::tileset::TileSet::new(tex, #width, #height, #count);
                    set
                },
            }
        }
    );

    let (animation_struct_ts, _) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "animation",
        "mvutils::once::Lazy<mvengine::graphics::animation::GlobalAnimation<'static>>",
        animations,
        |animation| {
            let tileset = &animation.tileset;
            let range = &animation.range;
            let fps = animation.fps;

            let tile_ident = Ident::new(tileset, Span::call_site());
            let range_ts: Expr = parse_str(range).expect("range attribute must be a valid rust range");

            let err_msg = format!("Cannot find tileset '{tileset}'!");

            let tileset_id = if animation.use_mv {
                quote! { #r_ident.mv.tileset.#tile_ident }
            } else {
                quote! { #r_ident.tileset.#tile_ident }
            };

            quote! {
                mvutils::once::Lazy::new(|| {
                    let tileset = #r_ident.resolve_tileset(#tileset_id).expect(#err_msg);
                    let anim = mvengine::graphics::animation::GlobalAnimation::new(tileset, #range_ts, #fps);
                    anim
                }),
            }
        }
    );

    let animation_resolve_fn_ts = if !is_mv {
        quote! {
            fn resolve_animation(&self, id: usize) -> Option<&mvengine::graphics::animation::GlobalAnimation<'static>> {
                if id >= mvengine::ui::res::CR {
                    self.animation.animation_arr.get(id - mvengine::ui::res::CR).map(std::ops::Deref::deref)
                } else {
                    self.mv.resolve_animation(id)
                }
            }
        }
    } else {
        quote! {
             fn resolve_animation(&self, id: usize) -> Option<&mvengine::graphics::animation::GlobalAnimation<'static>> {
                self.animation.animation_arr.get(id).map(std::ops::Deref::deref)
            }
        }
    };


    let init_fn_ts = if !is_mv {
        quote! {
            mvengine::ui::res::MVR::initialize();
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

    let propagate_tick = if !is_mv {
        quote! { self.mv.tick_all_animations(); }
    } else { quote!{} };

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
        #font_struct_ts
        #tileset_struct_ts
        #animation_struct_ts

        #tile_struct_ts

        unsafe impl Send for #r_ident {}
        unsafe impl Sync for #r_ident {}

        impl #r_ident {
            pub fn initialize() {
                #init_fn_ts
            }
        }

        impl mvengine::ui::context::UiResources for #r_ident {
            #color_resolve_fn_ts
            #shape_resolve_fn_ts
            #adaptive_resolve_fn_ts
            #texture_resolve_fn_ts
            #font_resolve_fn_ts
            #tile_resolve_fn_ts
            #tileset_resolve_fn_ts
            #animation_resolve_fn_ts

            fn tick_all_animations(&self) {
                use std::ops::Deref;
                for anim in &self.animation.animation_arr {
                    let anim = anim.deref();
                    mvutils::unsafe_cast_mut!(anim, mvengine::graphics::animation::GlobalAnimation).tick();
                }
                #propagate_tick
            }
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

fn extent_resource<F, T>(
    is_mv: bool,
    r_field_tokens: &mut TS,
    r_field_gens_tokens: &mut TS,
    struct_name: &str,
    res_name: &str,
    res_type: &str,
    parsed: Vec<(String, T)>,
    creator: F
) -> (TS, TS) where F: Fn(&T) -> TS {
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
                #ident: #index + mvengine::ui::res::CR,
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
        let arr_entry = creator(&entry.1);
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
                if id >= mvengine::ui::res::CR {
                    self.#field_ident.#res_arr_ident.get(id - mvengine::ui::res::CR)
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

fn extend_tiles(tilesets: &[(String, ParsedTileSet)], r_field_tokens: &mut TS, r_field_gens_tokens: &mut TS, struct_name: &str, is_mv: bool) -> (TS, TS) {
    let mut total_count = 0usize;
    let mut tile_struct_fields_ts = quote! {};
    let mut tile_struct_fields_init_ts = quote! {};
    let mut structs = quote! {};
    for (tileset_name, tileset) in tilesets {
        let mut tile_tiles_struct_fields_ts = quote! {};
        let mut tile_tiles_struct_fields_init_ts = quote! {};

        if let Some(fps) = tileset.fps {
            tile_tiles_struct_fields_ts.extend(quote! {
                pub fps: u16,
            });
            tile_tiles_struct_fields_init_ts.extend(quote! {
                fps: #fps,
            });
        }

        let ty = Ident::new(&format!("{struct_name}_tile_{tileset_name}"), Span::call_site());
        let ident = Ident::new(&tileset_name, Span::call_site());
        tile_struct_fields_ts.extend(quote! {
            pub #ident: #ty,
        });

        for (name, value) in &tileset.entries {
            total_count += 1;
            let ident = Ident::new(name.as_str(), Span::call_site());
            tile_tiles_struct_fields_ts.extend(quote! {
                pub #ident: usize,
            });
            tile_tiles_struct_fields_init_ts.extend(quote! {
                #ident: #value,
            });
        }

        tile_struct_fields_init_ts.extend(quote! {
            #ident: #ty {
                #tile_tiles_struct_fields_init_ts
            },
        });

        let tile_tiles_struct_ts = quote! {
            pub struct #ty {
                #tile_tiles_struct_fields_ts
            }
        };
        structs.extend(tile_tiles_struct_ts);
    }

    let ty = Ident::new(&format!("{struct_name}_tiles"), Span::call_site());

    let pm = quote! {
        pub struct #ty {
            #tile_struct_fields_ts
        }

        #structs
    };

    r_field_tokens.extend(quote! {
       pub tile: #ty,
    });

    r_field_gens_tokens.extend(quote! {
        tile: #ty {
            #tile_struct_fields_init_ts
        },
    });

    let res_fn_ts = if !is_mv {
        quote! {
            fn resolve_tile(&self, id: usize, index: usize) -> Option<(&mvengine::rendering::texture::Texture, mvengine::math::vec::Vec4)> {
                if id >= mvengine::ui::res::CR {
                    self.tileset.tileset_arr[id - mvengine::ui::res::CR].get_tile(index)
                } else {
                    self.mv.resolve_tile(id, index)
                }
            }
        }
    } else {
        quote! {
            fn resolve_tile(&self, id: usize, index: usize) -> Option<(&mvengine::rendering::texture::Texture, mvengine::math::vec::Vec4)> {
                self.tileset.tileset_arr[id].get_tile(index)
            }
        }
    };

    (pm, res_fn_ts)
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
                    let mut sampler = format!("|nearest");
                    if let Some(XmlValue::Str(sam)) = entity.get_attrib("sampler") {
                        sampler = format!("|{sam}");
                    }
                    let mut cloned_val_s = val_s.clone();
                    cloned_val_s.push_str(sampler.as_str());
                    return (name_s.clone(), cloned_val_s);
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

fn parse_font(entity: &Entity) -> (String, String){
    if entity.name().as_str() != "font" {
        panic!("Font resource must be named font, got {}!", entity.name());
    }
    if let Some(val) = entity.get_attrib("src") {
        if let Some(name) = entity.get_attrib("name") {
            if let XmlValue::Str(val_s) = val {
                if let XmlValue::Str(name_s) = name {
                    if let Some(XmlValue::Str(atlas)) = entity.get_attrib("atlas") {
                        let lit = format!("{val_s}|{atlas}");
                        return (name_s.clone(), lit);
                    }
                }
            }
            panic!("Code blocks are not supported in font!")
        } else {
            panic!("Expected a 'name' attribute on font!")
        }
    } else {
        panic!("Expected a 'src' attribute on font!")
    }
}

struct ParsedTileSet {
    atlas: String,
    width: i32,
    height: i32,
    fps: Option<u16>,
    count: usize,
    linear: bool,
    entries: Vec<(String, usize)>,
}

fn parse_tileset(entity: &Entity) -> (String, ParsedTileSet) {
    if entity.name().as_str() != "tileset" {
        panic!("Tileset resource must be named tileset, got {}!", entity.name());
    }

    let atlas = entity.get_attrib("atlas");
    let name = entity.get_attrib("name");
    let width = entity.get_attrib("width");
    let height = entity.get_attrib("height");
    let count = entity.get_attrib("count");

    if let (Some(XmlValue::Str(atlas)), Some(XmlValue::Str(name)), Some(XmlValue::Str(width)), Some(XmlValue::Str(height)), Some(XmlValue::Str(count))) = (atlas, name, width, height, count) {
        let mut linear = false;
        if let Some(XmlValue::Str(sampler)) = entity.get_attrib("sampler") {
            linear = sampler == "linear";
        }

        let fps = if let Some(XmlValue::Str(fps)) = entity.get_attrib("fps") {
            Some(fps.parse::<u16>().expect("fps must be a positive number between 0-65535"))
        } else {
            None
        };

        let mut parsed = ParsedTileSet {
            atlas: atlas.to_string(),
            width: width.parse::<i32>().expect("Tileset width must be a number"),
            height: height.parse::<i32>().expect("Tileset height must be a number"),
            count: count.parse::<usize>().expect("Tileset count must be a number"),
            linear,
            fps,
            entries: vec![],
        };
        if let Some(XmlValue::Entities(entities)) = entity.inner() {
            for entity in entities {
                if entity.name() == "fps" {
                    if let Some(XmlValue::Str(fps)) = entity.get_attrib("value") {
                        parsed.fps = Some(fps.parse::<u16>().expect("fps must be a positive number between 0-65535"));
                    } else if let Some(XmlValue::Str(fps)) = entity.inner() {
                        parsed.fps = Some(fps.parse::<u16>().expect("fps must be a positive number between 0-65535"));
                    } else {
                        panic!("fps tag must either contain 'value' attribute or string child")
                    }
                    continue;
                }
                if entity.name() != "entry" {
                    panic!("Unsupported tileset entry type '{}'", entity.name());
                }
                let name = entity.get_attrib("name");
                let index = entity.get_attrib("index");

                if let (Some(XmlValue::Str(name)), Some(XmlValue::Str(index))) = (name, index) {
                    parsed.entries.push((name.to_string(), index.parse::<usize>().expect("Tileset entry index must be a number")));
                } else {
                    panic!("Tileset entry must contain 'name' and 'index' attributes");
                }
            }
        }

        (name.to_string(), parsed)
    } else {
        panic!("Tileset must contain 'altas', 'name', 'width', 'height' and 'count' attributes");
    }
}

struct ParsedAnimation {
    tileset: String,
    range: String,
    fps: u16,
    use_mv: bool,
}

fn parse_animation(entity: &Entity) -> (String, ParsedAnimation) {
    if entity.name().as_str() != "animation" {
        panic!("Animation resource must be named animation, got {}!", entity.name());
    }

    let tileset = entity.get_attrib("tileset");
    let name = entity.get_attrib("name");
    let fps = entity.get_attrib("fps");

    if let (Some(XmlValue::Str(tileset)), Some(XmlValue::Str(name)), Some(XmlValue::Str(fps))) = (tileset, name, fps) {
        let range = if let Some(XmlValue::Str(range)) = entity.get_attrib("range") {
            range.to_string()
        } else {
            "..".to_string()
        };

        let use_mv = if let Some(XmlValue::Str(mv)) = entity.get_attrib("mv") {
            !mv.is_empty()
        } else {
            false
        };

        (name.to_string(), ParsedAnimation {
            tileset: tileset.to_string(),
            range,
            fps: fps.parse::<u16>().expect("fps must be a positive number between 0-65535"),
            use_mv,
        })
    } else {
        panic!("Animation must contain 'tileset', 'name' and 'fps' attributes");
    }
}