use std::fmt::{Debug, Formatter, Write};
use crate::render::arc::ArcCtx;
use crate::render::rectangle::RectangleCtx;
use crate::render::triangle::TriangleCtx;
use crate::render::UiRenderer;
use mvcore::render::backend::swapchain::SwapchainError;
use mvcore::render::texture::{DrawTexture, Texture};
use mve2d::gpu::Transform;
use mve2d::renderer2d::{InputTriangle, SamplerType};
use std::sync::Arc;
use mvcore::color::RgbColor;

#[derive(Clone)]
pub struct DrawShape {
    pub triangles: Vec<InputTriangle>,
    pub textures: Vec<(Arc<Texture>, SamplerType)>
}

impl Debug for DrawShape {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{\n")?;
        for triangle in &self.triangles {
            triangle.vec2s().fmt(f)?;
            f.write_char('\n')?;
        }
        f.write_char('}')
    }
}

impl DrawShape {
    pub fn apply_transformations(&mut self) {
        for triangle in self.triangles.iter_mut() {
            let transform = &triangle.transform;
            let p1 = transform.apply_for_point(triangle.points[0]);
            let p2 = transform.apply_for_point(triangle.points[1]);
            let p3 = transform.apply_for_point(triangle.points[2]);
            triangle.transform = Transform::new();
            triangle.points = [p1, p2, p3];
        }
    }

    pub fn combine(&mut self, other: &DrawShape) {
        self.triangles.extend(other.triangles.iter().cloned());
        self.textures.extend(other.textures.iter().cloned());
    }

    pub fn recenter(&mut self) {
        let mut total_x = 0;
        let mut total_y = 0;
        for triangle in self.triangles.iter() {
            let center = triangle.center();
            total_x += center.0;
            total_y += center.1;
        }
        let new_center = (total_x as f32 / self.triangles.len() as f32, total_y as f32 / self.triangles.len() as f32);
        for triangle in self.triangles.iter_mut() {
            triangle.transform.origin.x = new_center.0;
            triangle.transform.origin.y = new_center.1;
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        for triangle in self.triangles.iter_mut() {
            triangle.transform = transform.clone();
        }
    }

    pub fn modify_transform<F: FnMut(&mut Transform)>(&mut self, mut transformation: F) {
        for triangle in self.triangles.iter_mut() {
            transformation(&mut triangle.transform);
        }
    }

    pub fn set_translate(&mut self, x: i32, y: i32) {
        self.modify_transform(|t| {
            t.translation.x = x as f32;
            t.translation.y = y as f32;
        });
    }

    pub fn set_scale(&mut self, x: i32, y: i32) {
        self.modify_transform(|t| {
            t.scale.x = x as f32;
            t.scale.y = y as f32;
        });
    }

    pub fn set_origin(&mut self, x: i32, y: i32) {
        self.modify_transform(|t| {
            t.origin.x = x as f32;
            t.origin.y = y as f32;
        });
    }

    pub fn set_rotation(&mut self, rot: f32) {
        self.modify_transform(|t| {
            t.rotation = rot.to_radians();
        });
    }

    pub fn set_texture(&mut self, texture: TextureCtx) {
        if let Some(tex) = texture.texture {
            self.textures.clear();
            self.textures.push((tex.get_texture(), texture.sampler.clone()));
            let mut min_x = i32::MAX;
            let mut max_x = i32::MIN;
            let mut min_y = i32::MAX;
            let mut max_y = i32::MIN;

            for triangle in &self.triangles {
                for &(x, y) in &triangle.points {
                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }
            }

            let width = (max_x - min_x) as f32;
            let height = (max_y - min_y) as f32;

            let uv_bounds = tex.get_uv();
            let (uv_tl, uv_tr, uv_br, uv_bl) = (
                uv_bounds[0], // Top-left
                uv_bounds[1], // Top-right
                uv_bounds[2], // Bottom-right
                uv_bounds[3], // Bottom-left
            );

            for triangle in self.triangles.iter_mut() {
                triangle.tex_id = Some(0);

                let tex_coords = triangle.points.map(|(x, y)| {
                    let normalized_u = (x as f32 - min_x as f32) / width;
                    let normalized_v = (y as f32 - min_y as f32) / height;

                    let u = uv_tl.0 + (uv_tr.0 - uv_tl.0) * normalized_u;
                    let v = uv_tl.1 + (uv_bl.1 - uv_tl.1) * normalized_v;

                    (u, v)
                });

                triangle.tex_coords = Some(tex_coords);
            }
        }
    }

    pub fn set_color(&mut self, color: RgbColor) {
        for triangle in self.triangles.iter_mut() {
            triangle.colors = [color.as_vec4(), color.as_vec4(), color.as_vec4()];
        }
    }
}

pub fn transform() -> TransformCtx {
    TransformCtx::new()
}

pub fn texture() -> TextureCtx {
    TextureCtx::new()
}

pub fn triangle() -> TriangleCtx {
    TriangleCtx::new()
}

pub fn rectangle() -> RectangleCtx {
    RectangleCtx::new()
}

pub fn arc() -> ArcCtx {
    ArcCtx::new()
}

pub struct DrawContext2D {
    renderer: UiRenderer,
    canvas_transform: Transform,
    custom_origin: bool,
}

impl DrawContext2D {
    pub fn new(renderer: UiRenderer) -> Self {
        Self {
            renderer,
            canvas_transform: Transform::new(),
            custom_origin: false,
        }
    }

    pub fn shape(&mut self, shape: DrawShape) {
        let mut ids = Vec::new();
        for (texture, sampler) in shape.textures {
            let id = self.renderer.set_texture(texture, sampler);
            ids.push(id);
        }

        for mut triangle in shape.triangles {
            triangle.z = self.renderer.gen_z();
            triangle.canvas_transform = self.canvas_transform.clone();
            if let Some(id) = triangle.tex_id {
                triangle.tex_id = Some(ids.get(id as usize).map(|x| *x).unwrap_or(0) as u16)
            }

            self.renderer.add_triangle(triangle);
        }
    }

    pub fn transform(&mut self, transform: TransformCtx) {
        self.canvas_transform = transform.transform;
        self.custom_origin = transform.origin_set;
    }

    pub fn draw(&mut self) -> Result<(), SwapchainError> {
        self.renderer.draw()
    }
}

pub struct TransformCtx {
    pub(crate) transform: Transform,
    pub(crate) origin_set: bool,
}

impl TransformCtx {
    fn new() -> Self {
        Self {
            transform: Transform::new(),
            origin_set: false,
        }
    }

    pub fn from_transform(transform: Transform, custom_origin: bool) -> Self {
        Self {
            transform,
            origin_set: custom_origin,
        }
    }

    pub fn translate(mut self, x: i32, y: i32) -> Self {
        self.transform.translation.x = x as f32;
        self.transform.translation.y = y as f32;
        self
    }

    pub fn rotate(mut self, rotation: f32) -> Self {
        self.transform.rotation = rotation.to_radians();
        self
    }

    pub fn origin(mut self, x: i32, y: i32) -> Self {
        self.transform.origin.x = x as f32;
        self.transform.origin.y = y as f32;
        self.origin_set = true;
        self
    }

    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.transform.scale.x = x;
        self.transform.scale.y = y;
        self
    }

    pub fn get(&self) -> Transform {
        self.transform.clone()
    }
}

pub struct TextureCtx {
    pub(crate) texture: Option<DrawTexture>,
    pub(crate) blending: f32,
    pub(crate) sampler: SamplerType
}

impl TextureCtx {
    pub fn new() -> Self {
        Self {
            texture: None,
            blending: 0.0,
            sampler: SamplerType::Linear,
        }
    }

    pub fn source(mut self, texture: Option<DrawTexture>) -> Self {
        self.texture = texture;
        self
    }

    pub fn blending(mut self, blending: f32) -> Self {
        self.blending = blending;
        self
    }

    pub fn sampler(mut self, sampler: SamplerType) -> Self {
        self.sampler = sampler;
        self
    }
}