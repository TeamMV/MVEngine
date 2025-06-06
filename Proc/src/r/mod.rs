mod color;
mod shape;
mod adaptive;
mod texture;
mod font;
mod animation;
mod composite;
mod tileset;
mod drawable;

use proc_macro::TokenStream;
use mvutils::utils::TetrahedronOp;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_str, Expr, Path};
use animation::ParsedAnimation;
use composite::{ParsedComposite, PartRes};
use tileset::ParsedTileSet;
use ui_parsing::xml::{parse_rsx, XmlValue};
use crate::r::adaptive::parse_adaptive;
use crate::r::animation::parse_animation;
use crate::r::color::parse_color;
use crate::r::composite::parse_composite;
use crate::r::drawable::{parse_drawable, DrawableType, ParsedDrawable};
use crate::r::font::parse_font;
use crate::r::shape::parse_shape;
use crate::r::texture::parse_texture;
use crate::r::tileset::parse_tileset;

pub fn r(input: TokenStream) -> TokenStream {
    let content = input.to_string();
    let rsx = parse_rsx(content).unwrap();
    if rsx.name().as_str() != "resources" {
        panic!("Expected root tag to be resources!");
    }

    let mut struct_name = "R";
    let mut cdir = "./".to_string();
    let mut is_mv = false;
    let mut is_noctx= false;

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
    if let Some(value) = rsx.get_attrib("noctx") {
        if let XmlValue::Str(_) = value {
            is_noctx = true;
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
    let mut composites: Vec<(String, ParsedComposite)> = vec![];
    let mut drawables: Vec<(String, ParsedDrawable)> = vec![];

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
                    "composites" => branch!(composites, parse_composite),
                    "drawables" => branch!(drawables, parse_drawable),

                    _ => panic!("Invalid resource type {ty}")
                }
            }
        }
    }

    //code generation

    let r_ident = Ident::new(struct_name, Span::call_site());

    let mv_ts = if !is_mv && !is_noctx {
        quote! { pub mv: &'static mvengine::ui::res::MVR, }
    } else {
        quote! {}
    };

    let mv_gen_ts = if !is_mv && !is_noctx {
        quote! { mv: &mvengine::ui::res::MVR, }
    } else {
        quote! {}
    };

    let mut res_gens_ts = quote! {};
    let mut r_fields_ts = quote! {};

    // #################################################
    // ########## Extending the resources ##############
    // #################################################

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
        "mvengine::ui::geometry::shape::Shape",
        shapes,
        |lit| {
            let path = get_src(cdir.as_str(), lit);
            quote! {
                {
                    let ast = mvengine::ui::rendering::shapes::ShapeParser::parse(include_str!(#path)).unwrap();
                    let mut shape = mvengine::ui::rendering::shapes::shape_gen::ShapeGenerator::generate(ast).unwrap();
                    shape.invalidate();
                    shape
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
        is_noctx.yn("mvengine::rendering::texture::NoCtxTexture", "mvengine::rendering::texture::Texture"),
        textures,
        |lit| {
            let (src, sampler) = lit.split_once('|').expect("Meta should always contain a colon");
            let path = get_src(cdir.as_str(), src);
            let linear = "linear" == sampler.to_lowercase();

            if is_noctx {
                quote! {
                    {
                        let tex = mvengine::rendering::texture::NoCtxTexture::new(include_bytes!(#path), #linear);
                        tex
                    }
                }
            } else {
                quote! {
                    {
                        let tex = mvengine::rendering::texture::Texture::from_bytes_sampled(include_bytes!(#path), #linear).expect("Cannot load texture!");
                        tex
                    },
                }
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

    let (tile_struct_ts, tile_resolve_fn_ts, _) = extend_tiles(&tilesets, &mut r_fields_ts, &mut res_gens_ts, struct_name, is_mv);

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
                    let anim = mvengine::graphics::animation::GlobalAnimation::new(tileset, #tileset_id, #range_ts, #fps);
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

    let (composite_struct_ts, _) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "composite",
        "mvutils::once::Lazy<mvengine::graphics::comp::CompositeSprite>",
        composites,
        |composite| {

            let mut vec_ts = quote! {};
            for res in &composite.parts {
                let ts = match res {
                    PartRes::Texture(tex) => {
                        let ident: Expr = parse_str(tex).unwrap();
                        quote! { mvengine::graphics::Drawable::Texture(#r_ident.#ident), }
                    }
                    PartRes::Anim(anim) => {
                        let ident: Expr = parse_str(anim).unwrap();
                        quote! { mvengine::graphics::Drawable::Animation(#r_ident.#ident), }
                    }
                    PartRes::TileSet(tileset, tile) => {
                        let ident_ts: Expr = parse_str(tileset).unwrap();
                        let ident_t: Expr = parse_str(tile).unwrap();

                        quote! { mvengine::graphics::Drawable::TileSet(#r_ident.tileset.#ident_ts, #r_ident.#ident_t), }
                    }
                };
                vec_ts.extend(ts);
            }

            let rig = &composite.rig;
            let rig = get_src(cdir.as_str(), rig);
            quote! {
                {
                    mvutils::once::Lazy::new(|| {
                        let comp = mvengine::graphics::comp::CompositeSprite::from_expr_and_resources(include_str!(#rig), vec![#vec_ts]);
                        comp.unwrap()
                    })
                },
            }
        }
    );

    let composite_resolve_fn_ts = if !is_mv {
        quote! {
            fn resolve_composite(&self, id: usize) -> Option<&mvengine::graphics::comp::CompositeSprite> {
                if id >= mvengine::ui::res::CR {
                    self.composite.composite_arr.get(id - mvengine::ui::res::CR).map(std::ops::Deref::deref)
                } else {
                    self.mv.resolve_composite(id)
                }
            }
        }
    } else {
        quote! {
             fn resolve_composite(&self, id: usize) -> Option<&mvengine::graphics::comp::CompositeSprite> {
                self.composite.composite_arr.get(id).map(std::ops::Deref::deref)
            }
        }
    };


    let (drawable_struct_ts, _) = extent_resource(
        is_mv,
        &mut r_fields_ts,
        &mut res_gens_ts,
        struct_name,
        "drawable",
        "mvutils::once::Lazy<mvengine::graphics::Drawable>",
        drawables,
        |parsed| {
            let init_ts = match parsed.drawable_type {
                DrawableType::Color => {
                    let v = &parsed.thingies[0];
                    let ident = Ident::new(v, Span::call_site());
                    quote! { mvengine::graphics::Drawable::Color(#r_ident.color.#ident) }
                }
                DrawableType::Texture => {
                    let v = &parsed.thingies[0];
                    let ident = Ident::new(v, Span::call_site());
                    quote! { mvengine::graphics::Drawable::Texture(#r_ident.texture.#ident) }
                }
                DrawableType::Animation => {
                    let v = &parsed.thingies[0];
                    let ident = Ident::new(v, Span::call_site());
                    quote! { mvengine::graphics::Drawable::Animation(#r_ident.animation.#ident) }
                }
                DrawableType::Tileset => {
                    let v = &parsed.thingies[0];
                    let v1 = &parsed.thingies[1];
                    let ident = Ident::new(v, Span::call_site());
                    let ident2 = Ident::new(v1, Span::call_site());
                    quote! { mvengine::graphics::Drawable::TileSet(#r_ident.tileset.#v, #r_ident.tile.#ident.#ident2) }
                }
            };

            quote! { mvutils::once::Lazy::new(|| #init_ts) }
        }
    );

    let drawable_resolve_fn_ts = if !is_mv {
        quote! {
            fn resolve_drawable(&self, id: usize) -> Option<&mvengine::graphics::Drawable> {
                if id >= mvengine::ui::res::CR {
                    self.drawable.drawable_arr.get(id - mvengine::ui::res::CR).map(std::ops::Deref::deref)
                } else {
                    self.mv.resolve_drawable(id)
                }
            }
        }
    } else {
        quote! {
             fn resolve_drawable(&self, id: usize) -> Option<&mvengine::graphics::Drawable> {
                self.drawable.drawable_arr.get(id).map(std::ops::Deref::deref)
            }
        }
    };


    // ########################################
    // ###########  R struct setup ############
    // ########################################

    let init_fn_ts = if !is_mv && !is_noctx {
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

    let impl_ctx = if is_noctx {
        quote! {}
    } else {
        quote! {
            impl mvengine::ui::context::UiResources for #r_ident {
            #color_resolve_fn_ts
            #shape_resolve_fn_ts
            #adaptive_resolve_fn_ts
            #texture_resolve_fn_ts
            #font_resolve_fn_ts
            #tile_resolve_fn_ts
            #tileset_resolve_fn_ts
            #animation_resolve_fn_ts
            #composite_resolve_fn_ts
            #drawable_resolve_fn_ts

            fn tick_all_animations(&self) {
                use std::ops::Deref;
                    for anim in &self.animation.animation_arr {
                        let anim = anim.deref();
                        mvutils::unsafe_cast_mut!(anim, mvengine::graphics::animation::GlobalAnimation).tick();
                    }
                    #propagate_tick
                }
            }
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
        #font_struct_ts
        #tileset_struct_ts
        #animation_struct_ts
        #composite_struct_ts
        #drawable_struct_ts

        #tile_struct_ts

        unsafe impl Send for #r_ident {}
        unsafe impl Sync for #r_ident {}

        impl #r_ident {
            pub fn initialize() {
                #init_fn_ts
            }
        }

        #impl_ctx

        impl mvengine::ui::res::runtime::ResourceSavable for #r_ident {
            fn save_res(&self, saver: &mut impl mvutils::save::Saver) {
                use mvengine::ui::res::runtime::save_array_as_vec;
                use mvengine::ui::res::runtime::save_res_array_as_vec;

                save_array_as_vec(saver, &self.color.color_arr);
                save_array_as_vec(saver, &self.shape.shape_arr);
                save_array_as_vec(saver, &self.adaptive.adaptive_arr);
                save_array_as_vec(saver, &self.texture.texture_arr);
                save_array_as_vec(saver, &self.font.font_arr);
                //save_array_as_vec()//self.tile.tile_arr.to_veO;
                save_array_as_vec(saver, &self.tileset.tileset_arr);
                save_res_array_as_vec(saver, &self.animation.animation_arr);
                save_array_as_vec(saver, &self.composite.composite_arr);
                save_array_as_vec(saver, &self.drawable.drawable_arr);
            }

            fn load_res(loader: &mut impl mvutils::save::Loader, resources: &impl mvengine::ui::context::UiResources) -> Result<Self, String> {
                Err("resource structs cannot be loaded. Use RuntimeResources instead.".to_string())
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

fn extend_tiles(tilesets: &[(String, ParsedTileSet)], r_field_tokens: &mut TS, r_field_gens_tokens: &mut TS, struct_name: &str, is_mv: bool) -> (TS, TS, TS) {
    let mut tile_struct_fields_ts = quote! {};
    let mut tile_struct_fields_init_ts = quote! {};
    let mut structs = quote! {};
    let tile_save_ts = quote! {};
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

        //tile_save_ts.extend(quote! {
        //    self.tile.#ident
        //});

        for (name, value) in &tileset.entries {
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

    (pm, res_fn_ts, tile_save_ts)
}