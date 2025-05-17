use crate::color::RgbColor;
use crate::graphics::animation::GlobalAnimation;
use crate::graphics::comp::CompositeSprite;
use crate::graphics::tileset::TileSet;
use crate::math::vec::Vec4;
use crate::rendering::text::Font;
use crate::rendering::texture::{NoCtxTexture, Texture};
use crate::ui::context::UiResources;
use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::adaptive::AdaptiveShape;
use crate::ui::res::MVR;
use itertools::Itertools;
use mvutils::save::{Loader, Savable, Saver};
use std::ops::Deref;
use mvutils::once::Lazy;
use crate::ui;
use crate::ui::res;

pub struct RuntimeResources<'a> {
    colors: Vec<RgbColor>,
    shapes: Vec<Shape>,
    adaptives: Vec<AdaptiveShape>,
    textures: Vec<Texture>,
    fonts: Vec<Font>,
    tilesets: Vec<TileSet>,
    animations: Vec<GlobalAnimation<'a>>,
    composites: Vec<CompositeSprite>
}

pub fn save_array_as_vec<T: Savable, const N: usize>(saver: &mut impl Saver, arr: &[T; N]) {
    (N as u64).save(saver);
    for i in 0..N {
        arr[i].save(saver);
    }
}

pub fn save_res_array_as_vec<T: ResourceSavable, const N: usize>(saver: &mut impl Saver, arr: &[T; N]) {
    (N as u64).save(saver);
    for i in 0..N {
        arr[i].save_res(saver);
    }
}

pub trait ResourceSavable: Sized {
    fn save_res(&self, saver: &mut impl Saver);
    fn load_res(loader: &mut impl Loader, resources: &impl UiResources) -> Result<Self, String>;
}

impl<T: ResourceSavable> ResourceSavable for Vec<T> {
    fn save_res(&self, saver: &mut impl Saver) {
        self.len().save(saver);
        self.iter().for_each(|t| t.save_res(saver));
    }

    fn load_res(loader: &mut impl Loader, resources: &impl UiResources) -> Result<Self, String> {
        let n = usize::load(loader)?;
        let mut vec = Vec::with_capacity(n);
        for _ in 0..n {
            let t = T::load_res(loader, resources)?;
            vec.push(t);
        }
        Ok(vec)
    }
}

impl<T: ResourceSavable> ResourceSavable for Lazy<T> {
    fn save_res(&self, saver: &mut impl Saver) {
        let inner = self.deref();
        inner.save_res(saver);
    }

    fn load_res(loader: &mut impl Loader, resources: &impl UiResources) -> Result<Self, String> {
        let t = T::load_res(loader, resources)?;
        Ok(Self::new_initialized(t))
    }
}

impl Savable for RuntimeResources<'_> {
    fn save(&self, saver: &mut impl Saver) {
        self.colors.save(saver);
        self.shapes.save(saver);
        self.adaptives.save(saver);
        self.textures.save(saver);
        self.fonts.save(saver);
        self.tilesets.save(saver);
        self.animations.save_res(saver);
        self.composites.save(saver);
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let colors = Vec::<RgbColor>::load(loader)?;
        let shapes = Vec::<Shape>::load(loader)?;
        let adaptives = Vec::<AdaptiveShape>::load(loader)?;
        let textures = Vec::<NoCtxTexture>::load(loader)?;
        let textures = textures.into_iter().map(|t| {
            if let Ok(tex) = Texture::try_from(t) {
                tex
            } else {
                MVR.resolve_texture(MVR.texture.missing).expect("MVR not loaded").clone()
            }
        }).collect_vec();
        let fonts = Vec::<Font>::load(loader)?;
        let tilesets = Vec::<TileSet>::load(loader)?;
        let composites = Vec::<CompositeSprite>::load(loader)?;
        
        let mut this = Self {
            colors,
            shapes,
            adaptives,
            textures,
            fonts,
            tilesets,
            animations: Vec::new(),
            composites,
        };
        
        let animations = Vec::<GlobalAnimation>::load_res(loader, &this)?;
        this.animations = animations;
        Ok(this)
    }
}

impl UiResources for RuntimeResources<'_> {
    fn resolve_color(&self, id: usize) -> Option<&RgbColor> {
        if id < res::CR {
            MVR.resolve_color(id)
        } else {
            self.colors.get(id - res::CR)
        }
    }

    fn resolve_shape(&self, id: usize) -> Option<&Shape> {
        if id < res::CR {
            MVR.resolve_shape(id)
        } else {
            self.shapes.get(id - res::CR)
        }
    }

    fn resolve_adaptive(&self, id: usize) -> Option<&AdaptiveShape> {
        if id < res::CR {
            MVR.resolve_adaptive(id)
        } else {
            self.adaptives.get(id - res::CR)
        }
    }

    fn resolve_texture(&self, id: usize) -> Option<&Texture> {
        if id < res::CR {
            MVR.resolve_texture(id)
        } else {
            self.textures.get(id - res::CR)
        }
    }

    fn resolve_font(&self, id: usize) -> Option<&Font> {
        if id < res::CR {
            MVR.resolve_font(id)
        } else {
            self.fonts.get(id - res::CR)
        }
    }

    fn resolve_tile(&self, id: usize, index: usize) -> Option<(&Texture, Vec4)> {
        if id < res::CR {
            MVR.resolve_tile(id, index)
        } else {
            self.resolve_tileset(id).and_then(|set| set.get_tile(index))
        }
    }

    fn resolve_tileset(&self, id: usize) -> Option<&TileSet> {
        if id < res::CR {
            MVR.resolve_tileset(id)
        } else {
            self.tilesets.get(id - res::CR)
        }
    }

    fn resolve_animation(&self, id: usize) -> Option<&GlobalAnimation> {
        if id < res::CR {
            MVR.resolve_animation(id)
        } else {
            self.animations.get(id - res::CR)
        }
    }

    fn resolve_composite(&self, id: usize) -> Option<&CompositeSprite> {
        if id < res::CR {
            MVR.resolve_composite(id)
        } else {
            self.composites.get(id - res::CR)
        }
    }

    fn tick_all_animations(&self) {
        for anim in &self.animations {
            let anim = anim.deref();
            unsafe {
                ((anim as *const _) as *const GlobalAnimation)
                    .cast_mut()
                    .as_mut()
                    .unwrap()
            }.tick();
        }
    }
}