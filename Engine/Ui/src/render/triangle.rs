use mvcore::color::RgbColor;
use mvcore::render::texture::DrawTexture;
use mve2d::gpu::Transform;
use mve2d::renderer2d::{InputTriangle, SamplerType};
use crate::render::ctx::{DrawShape, TextureCtx, TransformCtx};

pub struct TriangleCtx {
    points: Vec<(i32, i32, Option<RgbColor>)>,
    global_color: RgbColor,
    transform: Transform,
    custom_origin: bool,
    texture: Option<DrawTexture>,
    sampler: SamplerType,
    blending: f32
}

impl TriangleCtx {
    pub(crate) fn new() -> Self {
        Self {
            points: vec![],
            global_color: RgbColor::white(),
            transform: Transform::new(),
            custom_origin: false,
            texture: None,
            sampler: SamplerType::Linear,
            blending: 0.0,
        }
    }

    pub fn point(mut self, xy: (i32, i32), color: Option<RgbColor>) -> Self {
        self.points.push((xy.0, xy.1, color));
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
        self.sampler = texture.sampler;
        self
    }

    pub fn create(mut self) -> DrawShape {
        let mut iter = self.points.into_iter();
        let p1 = iter.next().expect("Expected 3 points on a triangle");
        let p2 = iter.next().expect("Expected 3 points on a triangle");
        let p3 = iter.next().expect("Expected 3 points on a triangle");

        let c1 = p1.2.unwrap_or(self.global_color.clone());
        let c2 = p2.2.unwrap_or(self.global_color.clone());
        let c3 = p3.2.unwrap_or(self.global_color.clone());

        if !self.custom_origin {
            self.transform.origin.x = (p1.0 + p2.0 + p3.0) as f32 / 3.0;
            self.transform.origin.y = (p1.1 + p2.1 + p3.1) as f32 / 3.0;
        }

        let tex_id = if let Some(_) = self.texture { Some(0) } else { None };
        let tex_coords = if let Some(ref tex) = self.texture {
            let uv: [(f32, f32); 4] = tex.get_uv();

            let min_x = p1.0.min(p2.0).min(p3.0) as f32;
            let max_x = p1.0.max(p2.0).max(p3.0) as f32;
            let min_y = p1.1.min(p2.1).min(p3.1) as f32;
            let max_y = p1.1.max(p2.1).max(p3.1) as f32;

            let u1 = (p1.0 as f32 - min_x) / (max_x - min_x);
            let v1 = (p1.1 as f32 - min_y) / (max_y - min_y);

            let u2 = (p2.0 as f32 - min_x) / (max_x - min_x);
            let v2 = (p2.1 as f32 - min_y) / (max_y - min_y);

            let u3 = (p3.0 as f32 - min_x) / (max_x - min_x);
            let v3 = (p3.1 as f32 - min_y) / (max_y - min_y);

            Some([(u1, v1), (u2, v2), (u3, v3)])
        } else { None };

        let tri = InputTriangle {
            points: [(p1.0, p1.1), (p2.0, p2.1), (p3.0, p3.1)],
            z: 0.0,
            transform: self.transform,
            canvas_transform: Transform::new(),
            tex_id,
            tex_coords,
            blending: 0.0,
            colors: [c1.as_vec4(), c2.as_vec4(), c3.as_vec4()],
            is_font: false,
        };

        let mut textures = Vec::new();
        if let Some(ref tex) = self.texture {
            textures.push((tex.get_texture(), self.sampler));
        }

        DrawShape {
            triangles: vec![tri],
            textures,
            extent: (0, 0),
        }
    }
}