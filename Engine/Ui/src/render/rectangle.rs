use mvcore::color::RgbColor;
use mvcore::render::texture::DrawTexture;
use mve2d::gpu::Transform;
use mve2d::renderer2d::{InputTriangle, SamplerType};
use crate::render::ctx::{DrawShape, TextureCtx, TransformCtx};

#[derive(Copy, Clone)]
pub enum RectPoint {
    BottomLeft=0,
    TopLeft=1,
    BottomRight=2,
    TopRight=3
}

impl RectPoint {
    fn get_index(&self) -> usize {
        *self as usize
    }
}

pub struct RectangleCtx {
    points: Vec<(i32, i32)>,
    point_colors: Vec<Option<RgbColor>>,
    global_color: RgbColor,
    transform: Transform,
    custom_origin: bool,
    texture: Option<DrawTexture>,
    sampler: SamplerType,
    blending: f32
}

impl RectangleCtx {
    pub(crate) fn new() -> Self {
        Self {
            points: vec![],
            point_colors: [0; 4].map(|_| None).to_vec(),
            global_color: RgbColor::white(),
            transform: Transform::new(),
            custom_origin: false,
            texture: None,
            sampler: SamplerType::Linear,
            blending: 0.0,
        }
    }

    pub fn xywh(mut self, x: i32, y: i32, width: i32, height: i32) -> Self {
        self.points.push((x, y));
        self.points.push((x, y + height));
        self.points.push((x + width, y + height));
        self.points.push((x + width, y));

        self
    }

    pub fn xyxy(mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        self.points.push((x1, y1));
        self.points.push((x1, y2));
        self.points.push((x2, y2));
        self.points.push((x2, y1));

        self
    }

    pub fn point_color(mut self, point: RectPoint, color: Option<RgbColor>) -> Self {
        self.point_colors[point.get_index()] = color;
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
        let p1 = iter.next().expect("Expected 4 points on a rectangle");
        let p2 = iter.next().expect("Expected 4 points on a rectangle");
        let p3 = iter.next().expect("Expected 4 points on a rectangle");
        let p4 = iter.next().expect("Expected 4 points on a rectangle");

        let mut color_iter = self.point_colors.into_iter();
        let c1 = color_iter.next().unwrap().unwrap_or(self.global_color.clone());
        let c2 = color_iter.next().unwrap().unwrap_or(self.global_color.clone());
        let c3 = color_iter.next().unwrap().unwrap_or(self.global_color.clone());
        let c4 = color_iter.next().unwrap().unwrap_or(self.global_color.clone());

        if !self.custom_origin {
            self.transform.origin.x = (p1.0 + p3.0) as f32 * 0.5;
            self.transform.origin.y = (p1.1 + p3.1) as f32 * 0.5;
        }

        let tex_id = if let Some(_) = self.texture { Some(0) } else { None };
        let tex_coords_1 = if let Some(ref tex) = self.texture {
            let uv = tex.get_uv();
            Some([uv[0], uv[3], uv[2]])
        } else { None };

        let tex_coords_2 = if let Some(ref tex) = self.texture {
            let uv = tex.get_uv();
            Some([uv[0], uv[2], uv[1]])
        } else { None };

        let tri1 = InputTriangle {
            points: [(p1.0, p1.1), (p2.0, p2.1), (p3.0, p3.1)],
            z: 0.0,
            transform: self.transform.clone(),
            canvas_transform: Transform::new(),
            tex_id,
            tex_coords: tex_coords_1,
            blending: self.blending,
            colors: [c1.as_vec4(), c2.as_vec4(), c3.as_vec4()],
            is_font: false,
        };

        let tri2 = InputTriangle {
            points: [(p1.0, p1.1), (p3.0, p3.1), (p4.0, p4.1)],
            z: 0.0,
            transform: self.transform,
            canvas_transform: Transform::new(),
            tex_id,
            tex_coords: tex_coords_2,
            blending: self.blending,
            colors: [c2.as_vec4(), c3.as_vec4(), c4.as_vec4()],
            is_font: false,
        };

        let mut textures = Vec::new();
        if let Some(ref tex) = self.texture {
            textures.push((tex.get_texture(), self.sampler));
        }

        DrawShape {
            triangles: vec![tri1, tri2],
            textures,
            extent: (0, 0),
        }
    }
}