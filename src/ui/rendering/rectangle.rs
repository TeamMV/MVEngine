use mvutils::utils::TetrahedronOp;
use crate::color::RgbColor;
use crate::math::vec::Vec4;
use crate::rendering::texture::Texture;
use crate::rendering::{InputVertex, Transform, Triangle, Vertex};
use crate::ui::rendering::ctx::{DrawShape, TextureCtx, TransformCtx};

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
    texture: Option<Texture>,
    blending: f32,
    z: f32
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
            blending: 0.0,
            z: f32::INFINITY,
        }
    }

    pub fn z(mut self, z: f32) -> Self {
        self.z = z;
        self
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
        self
    }

    pub fn create(mut self) -> DrawShape {
        if !self.custom_origin {
            self.transform.origin.x = (self.points[0].0 + self.points[2].0) as f32 * 0.5;
            self.transform.origin.y = (self.points[0].1 + self.points[2].1) as f32 * 0.5;
        }

        let tex_id = if let Some(ref t) = self.texture { t.id } else { 0 };
        let mut tris = Vec::with_capacity(2);

        let uv = self.texture.as_ref().map(|tex| tex.get_uv()).unwrap_or([(0.0, 0.0); 4]);
        let tex_coords_1 = [uv[0], uv[3], uv[2]];
        let tex_coords_2 = [uv[0], uv[2], uv[1]];

        let colors: Vec<Vec4> = self
            .point_colors
            .iter()
            .cloned()
            .map(|c| c.unwrap_or_else(|| self.global_color.clone()).as_vec4())
            .collect();

        let vertices = [
            InputVertex {
                transform: self.transform.clone(),
                pos: (self.points[0].0 as f32, self.points[0].1 as f32, self.z),
                color: colors[0],
                uv: tex_coords_1[0],
                texture: tex_id,
                has_texture: self.texture.is_some().yn(1.0, 0.0),
            },
            InputVertex {
                transform: self.transform.clone(),
                pos: (self.points[1].0 as f32, self.points[1].1 as f32, self.z),
                color: colors[1],
                uv: tex_coords_1[1],
                texture: tex_id,
                has_texture: self.texture.is_some().yn(1.0, 0.0),
            },
            InputVertex {
                transform: self.transform.clone(),
                pos: (self.points[2].0 as f32, self.points[2].1 as f32, self.z),
                color: colors[2],
                uv: tex_coords_1[2],
                texture: tex_id,
                has_texture: self.texture.is_some().yn(1.0, 0.0),
            },
        ];

        let tri1 = Triangle {
            points: vertices.clone(),
        };

        let tri2 = Triangle {
            points: [
                vertices[0].clone(),
                vertices[2].clone(),
                InputVertex {
                    transform: self.transform.clone(),
                    pos: (self.points[3].0 as f32, self.points[3].1 as f32, self.z),
                    color: colors[3],
                    uv: tex_coords_2[2],
                    texture: tex_id,
                    has_texture: self.texture.is_some().yn(1.0, 0.0),
                },
            ],
        };

        tris.push(tri1);
        tris.push(tri2);

        DrawShape {
            triangles: tris,
            extent: (0, 0),
        }
    }

}