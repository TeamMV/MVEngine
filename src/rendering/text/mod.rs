use std::iter::Peekable;
use crate::color::RgbColor;
use crate::math::vec::{Vec2, Vec4};
use crate::rendering::text::font::{AtlasData, PreparedAtlasData};
use crate::rendering::texture::Texture;
use crate::rendering::{InputVertex, Quad, RenderContext, Transform};
use crate::utils::savers::SaveArc;
use bytebuffer::ByteBuffer;
use mvutils::Savable;
use mvutils::save::Savable;
use std::sync::Arc;
use log::warn;

pub mod font;

#[derive(Clone)]
pub struct CharData {
    pub uv: Vec4,
    pub width: f32,
    pub size: f32,
    pub y_off: f32,
}

#[derive(Clone, Savable)]
pub struct Font {
    texture: Texture,
    atlas: SaveArc<PreparedAtlasData>,
}

impl Font {
    pub fn new(texture: Texture, data_bytes: &[u8]) -> Result<Self, String> {
        let mut buffer = ByteBuffer::from_bytes(data_bytes);
        let atlas = AtlasData::load(&mut buffer)?;
        let arc: Arc<PreparedAtlasData> = Arc::new(atlas.into());
        drop(buffer);
        Ok(Self {
            texture,
            atlas: arc.into(),
        })
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn get_char_data(&self, char: char, size: f32) -> CharData {
        let glyph = if let Some(glyph) = self.atlas.find_glyph(char) {
            glyph
        } else {
            self.atlas.find_glyph('?').unwrap_or_else(|| {
                log::error!("Font atlas missing 'missing character' glyph");
                panic!()
            })
        };

        let bounds_plane = &glyph.plane_bounds;
        let bounds_atlas = &glyph.atlas_bounds;

        let mut tex_coords = Vec4::new(
            bounds_atlas.left as f32,
            (self.atlas.atlas.height as f64 - bounds_atlas.top) as f32,
            (bounds_atlas.right - bounds_atlas.left) as f32,
            (bounds_atlas.top - bounds_atlas.bottom) as f32,
        );

        tex_coords.x /= self.atlas.atlas.width as f32;
        tex_coords.y /= self.atlas.atlas.height as f32;
        tex_coords.z /= self.atlas.atlas.width as f32;
        tex_coords.w /= self.atlas.atlas.height as f32;

        //tex_coords.y = 1.0 - tex_coords.y;
        //tex_coords.z = tex_coords.z;

        let font_scale = self.get_scale(size);

        let mut scale = Vec2::new(
            (bounds_plane.right - bounds_plane.left) as f32,
            (bounds_plane.top - bounds_plane.bottom) as f32,
        );
        scale.x = scale.x * font_scale as f32;
        scale.y = scale.y * font_scale as f32;

        CharData {
            uv: tex_coords,
            width: scale.x,
            size: scale.y,
            y_off: bounds_plane.bottom as f32 * font_scale as f32,
        }
    }

    pub fn get_space_advance(&self, height: f32) -> f32 {
        let scale = self.get_scale(height);
        let g = self.atlas.find_glyph(' ');
        if let Some(g) = g {
            let space_advance =g.advance;
            space_advance as f32 * scale as f32
        } else {
            warn!("A font does not contain the space character, so the space_advance is defaulted to 3.0!");
            3.0 * scale as f32
        }
    }
    
    pub fn get_max_y_off(&self, height: f32) -> f32 {
        //massive tape code again
        //TODO
        self.get_char_data('g', height).y_off
    }

    pub fn get_scale(&self, height: f32) -> f64 {
        let atlas = &self.atlas;
        //let mut font_scale = 1.0 / (atlas.metrics.ascender - atlas.metrics.descender);
        //font_scale *= height as f64 / atlas.metrics.line_height;
        //font_scale
        height as f64 / atlas.metrics.line_height
    }

    pub fn get_width(&self, chars: impl Iterator<Item=char>, height: f32) -> f32 {
        let atlas = &self.atlas;

        let font_scale = self.get_scale(height);

        let space_advance = atlas.find_glyph(' ').unwrap().advance;
        let mut width = 0.0;

        let mut chars = chars.peekable();

        let mut idx = 0;
        while let Some(char) = chars.next() {
            if char == '\t' {
                width += 6.0 + space_advance * font_scale;
                continue;
            } else if char == ' ' {
                width += space_advance * font_scale;
                continue;
            } else if char == '\n' {
                continue;
            }

            let glyph = if let Some(glyph) = atlas.find_glyph(char) {
                glyph
            } else {
                atlas.find_glyph('?').unwrap_or_else(|| {
                    log::error!("Font atlas missing 'missing character' glyph");
                    panic!()
                })
            };

            let bounds_plane = &glyph.plane_bounds;
            let mut scale = Vec2::new(
                (bounds_plane.right - bounds_plane.left) as f32,
                (bounds_plane.top - bounds_plane.bottom) as f32,
            );
            scale.x = scale.x * font_scale as f32;
            scale.y = scale.y * font_scale as f32;

            let next = chars.peek().cloned().unwrap_or('i');
            let kerning = atlas.get_kerning(char, next).unwrap_or_default();

            width += scale.x as f64 + kerning * font_scale;
            idx += 1;
        }

        width as f32
    }

    pub fn draw(
        &self,
        chars: impl Iterator<Item = char>,
        height: f32,
        transform: Transform,
        z: f32,
        color: &RgbColor,
        controller: &mut impl RenderContext,
    ) {
        let atlas = &self.atlas;

        let font_scale = self.get_scale(height) as f32;
        let space_advance = atlas.find_glyph(' ').unwrap().advance as f32;

        let mut x = 0.0f32;
        let mut y = 0.0f32;

        let mut chars = chars.enumerate().peekable();

        while let Some((idx, char)) = chars.next() {
            if char == '\t' {
                x += 6.0 + space_advance * font_scale;
                continue;
            } else if char == ' ' {
                x += space_advance * font_scale;
                continue;
            } else if char == '\n' {
                x = 0.0;
                y -= font_scale * atlas.metrics.line_height as f32;
                continue;
            }

            let glyph = if let Some(glyph) = atlas.find_glyph(char) {
                glyph
            } else {
                atlas.find_glyph('?').unwrap_or_else(|| {
                    log::error!("Font atlas missing 'missing character' glyph");
                    panic!()
                })
            };

            let bounds_plane = &glyph.plane_bounds;
            let bounds_atlas = &glyph.atlas_bounds;

            let mut tex_coords = Vec4::new(
                bounds_atlas.left as f32,
                (atlas.atlas.height as f64 - bounds_atlas.top) as f32,
                (bounds_atlas.right - bounds_atlas.left) as f32,
                (bounds_atlas.top - bounds_atlas.bottom) as f32,
            );

            tex_coords.x /= atlas.atlas.width as f32;
            tex_coords.y /= atlas.atlas.height as f32;
            tex_coords.z /= atlas.atlas.width as f32;
            tex_coords.w /= atlas.atlas.height as f32;

            let mut scale = Vec2::new(
                (bounds_plane.right - bounds_plane.left) as f32,
                (bounds_plane.top - bounds_plane.bottom) as f32,
            );
            scale.x *= font_scale;
            scale.y *= font_scale;

            let y_offset = bounds_plane.bottom as f32 * scale.y;

            let vertex = |p: (f32, f32), uv: (f32, f32)| -> InputVertex {
                InputVertex {
                    transform: transform.clone(),
                    pos: (p.0 + x, (p.1 + y) + y_offset, z),
                    color: color.as_vec4(),
                    uv: (uv.0, 1.0 - uv.1),
                    texture: self.texture.id,
                    has_texture: 2.0,
                }
            };

            let quad = Quad {
                points: [
                    vertex((0.0, 0.0), (tex_coords.x, tex_coords.y + tex_coords.w)),
                    vertex((0.0, scale.y), (tex_coords.x, tex_coords.y)),
                    vertex((scale.x, scale.y), (tex_coords.x + tex_coords.z, tex_coords.y)),
                    vertex((scale.x, 0.0), (tex_coords.x + tex_coords.z, tex_coords.y + tex_coords.w)),
                ],
            };

            controller.controller().push_quad(quad);

            let next = chars.peek().map(|(_, c)| *c).unwrap_or('i');
            let kerning = atlas.get_kerning(char, next).unwrap_or_default() as f32;

            x += (glyph.advance as f32 + kerning) * font_scale;
        }
    }
}
