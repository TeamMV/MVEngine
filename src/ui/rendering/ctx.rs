use crate::color::RgbColor;
use crate::rendering::texture::Texture;
use crate::rendering::{Transform, Triangle};
use crate::window::Window;
use std::fmt::{Debug, Formatter, Write};
use crate::ui::rendering::arc::ArcCtx;
use crate::ui::rendering::rectangle::RectangleCtx;
use crate::ui::rendering::triangle::TriangleCtx;
use crate::ui::rendering::UiRenderer;

#[derive(Clone)]
pub struct DrawShape {
    pub triangles: Vec<Triangle>,
    pub extent: (i32, i32)
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
            for vertex in &mut triangle.points {
                let transform = &vertex.transform;
                let after = transform.apply_for_point((vertex.pos.0 as i32, vertex.pos.1 as i32));
                vertex.pos.0 = after.0 as f32;
                vertex.pos.1 = after.1 as f32;
                vertex.transform = Transform::new();
            }
        }
    }

    pub fn compute_extent(&mut self) {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for triangle in &self.triangles {
            for (x, y, _) in triangle.points.iter().map(|v| v.pos) {
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

        let width = max_x - min_x;
        let height = max_y - min_y;
        self.extent = (width as i32, height as i32);
    }

    fn set_z(&mut self, z: f32) {
        for tri in &mut self.triangles {
            tri.points.iter_mut().for_each(|v| v.pos.2 = z);
        }
    }

    pub fn combine(&mut self, other: &DrawShape) {
        self.triangles.extend(other.triangles.iter().cloned());
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
            triangle.points.iter_mut().for_each(|v| v.transform.origin.x = new_center.0);
            triangle.points.iter_mut().for_each(|v| v.transform.origin.y = new_center.1);
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        for triangle in self.triangles.iter_mut() {
            triangle.points.iter_mut().for_each(|v| v.transform = transform.clone());
        }
    }

    pub fn modify_transform<F: FnMut(&mut Transform)>(&mut self, mut transformation: F) {
        for triangle in self.triangles.iter_mut() {
            triangle.points.iter_mut().for_each(|v| transformation(&mut v.transform));
        }
    }

    pub fn set_translate(&mut self, x: i32, y: i32) {
        self.modify_transform(|t| {
            t.translation.x = x as f32;
            t.translation.y = y as f32;
        });
    }

    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.modify_transform(|t| {
            t.scale.x = x;
            t.scale.y = y;
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

    pub fn translated(mut self, x: i32, y: i32) -> Self {
        self.modify_transform(|t| {
            t.translation.x = x as f32;
            t.translation.y = y as f32;
        });
        self
    }

    pub fn rotated(mut self, r: f32) -> Self {
        self.modify_transform(|t| {
            t.rotation = r;
        });
        self
    }

    pub fn scaled(mut self, x: f32, y: f32) -> Self {
        self.modify_transform(|t| {
            t.scale.x = x;
            t.scale.y = y;
        });
        self
    }

    pub fn set_texture(&mut self, texture: TextureCtx) {
        if let Some(tex) = texture.texture {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;

            for triangle in &self.triangles {
                for (x, y) in triangle.points.iter().map(|v| (v.pos.0, v.pos.1)) {
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
                for vertex in triangle.points.iter_mut() {
                    let x = vertex.pos.0;
                    let y = vertex.pos.1;

                    let normalized_u = (x - min_x) / width;
                    let normalized_v = 1.0 - (y - min_y) / height;

                    let u = uv_tl.0 + (uv_tr.0 - uv_tl.0) * normalized_u;
                    let v = uv_tl.1 + (uv_bl.1 - uv_tl.1) * normalized_v;

                    vertex.uv = (u, v);
                    vertex.texture = tex.id;
                    vertex.has_texture = 1.0;
                }
            }
        }
    }

    pub fn set_color(&mut self, color: RgbColor) {
        for triangle in self.triangles.iter_mut() {
            triangle.points.iter_mut().for_each(|v| v.color = color.as_vec4());
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
}

impl DrawContext2D {
    pub fn new(renderer: UiRenderer) -> Self {
        Self {
            renderer,
        }
    }

    pub fn shape(&mut self, shape: DrawShape) {
        for mut triangle in shape.triangles {
            if triangle.points[0].pos.2 == f32::INFINITY {
                let z = self.renderer.gen_z();
                triangle.points.iter_mut().for_each(|v| v.pos.2 = z);
            }

            self.renderer.add_triangle(triangle);
        }
    }

    pub fn draw(&mut self, window: &Window) {
        self.renderer.draw(window)
    }

    pub fn renderer(&self) -> &UiRenderer {
        &self.renderer
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
    pub(crate) texture: Option<Texture>,
    pub(crate) blending: f32,
}

impl TextureCtx {
    pub fn new() -> Self {
        Self {
            texture: None,
            blending: 0.0,
        }
    }

    pub fn source(mut self, texture: Option<Texture>) -> Self {
        self.texture = texture;
        self
    }

    pub fn blending(mut self, blending: f32) -> Self {
        self.blending = blending;
        self
    }
}