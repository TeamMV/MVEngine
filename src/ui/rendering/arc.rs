use mvutils::utils::TetrahedronOp;
use crate::color::RgbColor;
use crate::rendering::texture::Texture;
use crate::rendering::{InputVertex, Transform, Triangle, Vertex};
use crate::ui::rendering::ctx::{DrawShape, TextureCtx, TransformCtx};

pub enum ArcTriPoint {
    Last,
    Current,
    Center
}

pub struct ArcCtx {
    center: (i32, i32),
    radius: i32,
    triangle_count: u32,
    angle: f32,
    global_color: RgbColor,
    transform: Transform,
    custom_origin: bool,
    texture: Option<Texture>,
    blending: f32,
    z: f32,
}

impl ArcCtx {
    pub(crate) fn new() -> Self {
        Self {
            center: (0, 0),
            radius: 0,
            triangle_count: 50,
            angle: 90.0,
            global_color: RgbColor::transparent(),
            transform: Transform::new(),
            custom_origin: false,
            texture: None,
            blending: 0.0,
            z: f32::INFINITY,
        }
    }

    pub fn z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    pub fn center(mut self, x: i32, y: i32) -> Self {
        self.center = (x, y);
        self
    }

    pub fn radius(mut self, radius: i32) -> Self {
        self.radius = radius;
        self
    }

    pub fn angle(mut self, angle: f32) -> Self {
        self.angle = angle.to_radians();
        self
    }

    pub fn triangle_count(mut self, count: u32) -> Self {
        self.triangle_count = count;
        self
    }

    pub fn color(mut self, color: RgbColor) -> Self {
        self.global_color = color;
        self
    }

    pub fn transform(mut self, transform: TransformCtx) -> Self {
        self.transform = transform.transform;
        self.custom_origin = transform.origin_set;
        self
    }

    pub fn texture(mut self, texture: TextureCtx) -> Self {
        self.texture = texture.texture;
        self.blending = texture.blending;
        self
    }

    pub fn create(mut self) -> DrawShape {
        if !self.custom_origin {
            self.transform.origin.x = self.center.0 as f32;
            self.transform.origin.y = self.center.1 as f32;
        }

        let mut tris = Vec::with_capacity(self.triangle_count as usize);

        let tex_id = if let Some(ref t) = self.texture { t.id } else { 0 };

        let rad = self.radius as f32;
        let step_size = self.angle / self.triangle_count as f32;
        let mut last_x = self.center.0 + self.radius;
        let mut last_y = self.center.1;
        for i in 1..self.triangle_count + 1 {
            let current = i as f32 * step_size;
            let x = (self.center.0 as f32 + current.cos() * rad) as i32;
            let y = (self.center.1 as f32 + current.sin() * rad) as i32;

            let tex_coords = if let Some(ref tex) = self.texture {
                let uv: [(f32, f32); 4] = tex.get_uv();
                let center_u = 0.5;
                let center_v = 0.5;

                let last_u = 0.5 + (last_x as f32 - self.center.0 as f32) / (2.0 * self.radius as f32);
                let last_v = 0.5 + (last_y as f32 - self.center.1 as f32) / (2.0 * self.radius as f32);

                let current_u = 0.5 + (x as f32 - self.center.0 as f32) / (2.0 * self.radius as f32);
                let current_v = 0.5 + (y as f32 - self.center.1 as f32) / (2.0 * self.radius as f32);

                [(last_u, last_v), (center_u, center_v), (current_u, current_v)]
            } else { [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)] };

            let tri = Triangle {
                points: [
                    InputVertex {
                        transform: self.transform.clone(),
                        pos: (last_x as f32, last_y as f32, self.z),
                        color: self.global_color.as_vec4(),
                        uv: tex_coords[0],
                        texture: tex_id,
                        has_texture: self.texture.is_some().yn(1.0, 0.0),
                    },
                    InputVertex {
                        transform: self.transform.clone(),
                        pos: (self.center.0 as f32, self.center.1 as f32, self.z),
                        color: self.global_color.as_vec4(),
                        uv: tex_coords[1],
                        texture: tex_id,
                        has_texture: self.texture.is_some().yn(1.0, 0.0),
                    },
                    InputVertex {
                        transform: self.transform.clone(),
                        pos: (x as f32, y as f32, self.z),
                        color: self.global_color.as_vec4(),
                        uv: tex_coords[2],
                        texture: tex_id,
                        has_texture: self.texture.is_some().yn(1.0, 0.0),
                    }
                ],
            };
            tris.push(tri);

            last_x = x;
            last_y = y;
        }

        DrawShape {
            triangles: tris,
            extent: (0, 0),
        }
    }
}