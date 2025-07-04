pub mod geometry;
pub mod msf;
pub mod msfx;
pub mod shapes;
pub mod transform;
pub mod utils;
pub mod visual;

use crate::color::RgbColor;
use crate::math::vec::{Vec2, Vec4};
use crate::rendering::{InputVertex, RenderContext, Transform};
use crate::ui::geometry::shape::geometry::CropStep;
use crate::ui::geometry::shape::transform::{
    OriginChangeStep, OriginSetStep, RotateStep, ScaleStep, TransformSetStep, TransformStep,
    TranslateStep,
};
use crate::ui::geometry::shape::visual::{ColorStep, TextureStep, UvStep};
use crate::ui::geometry::{SimpleRect, geom};
use gl::types::GLuint;
use mvutils::Savable;

pub enum Indices {
    Triangles,
    TriangleStrip,
    Manual(Vec<usize>),
}

impl Indices {
    pub fn get_them(self, verts: &[InputVertex]) -> Vec<usize> {
        match self {
            Indices::Triangles => {
                let count = verts.len();
                let mut indices = Vec::new();
                for i in (0..count).step_by(3) {
                    if i + 2 < count {
                        indices.extend_from_slice(&[i, i + 1, i + 2]);
                    }
                }
                indices
            }
            Indices::TriangleStrip => {
                let count = verts.len();
                let mut indices = Vec::new();
                for i in 0..(count - 2) {
                    if i % 2 == 0 {
                        indices.extend_from_slice(&[i, i + 1, i + 2]);
                    } else {
                        indices.extend_from_slice(&[i + 1, i, i + 2]);
                    }
                }
                indices
            }
            Indices::Manual(man) => man,
        }
    }
}

pub const SF_TEXTURE: u8 = 1;

#[derive(Clone, Savable, Debug)]
pub struct Shape {
    pub vertices: Vec<InputVertex>,
    pub indices: Vec<usize>,
    pub extent: SimpleRect,
    pub flags: u8,
}

impl Shape {
    pub fn new(vertices: Vec<InputVertex>, indices: Indices) -> Self {
        let indices = indices.get_them(&vertices);
        Self {
            vertices,
            indices,
            extent: SimpleRect::new(0, 0, 0, 0),
            flags: 0,
        }
    }

    pub fn new_with_extent(
        vertices: Vec<InputVertex>,
        indices: Indices,
        extent: SimpleRect,
    ) -> Self {
        let indices = indices.get_them(&vertices);
        Self {
            vertices,
            indices,
            extent: SimpleRect::new(0, 0, 0, 0),
            flags: 0,
        }
    }

    pub fn recompute(&mut self) {
        let mut has_tex = false;
        let (min_x, max_x, min_y, max_y) = self
            .vertices
            .iter()
            .inspect(|v| has_tex |= v.has_texture > 0.0)
            .map(|p| (p.pos.0, p.pos.1))
            .fold(
                (
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                ),
                |(min_x, max_x, min_y, max_y), (x, y)| {
                    (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
                },
            );

        self.extent = SimpleRect::new(
            min_x as i32,
            min_y as i32,
            (max_x - min_x) as i32,
            (max_y - min_y) as i32,
        );

        if has_tex {
            self.flags |= SF_TEXTURE
        }
    }

    pub fn recenter(&mut self) {
        self.recompute();
        let origin = self.extent.center();
        let origin = Vec2::new(origin.0 as f32, origin.1 as f32);
        self.stream().set_origin(origin).compute();
    }

    pub fn combine(&mut self, other: &Shape) {
        let off = self.vertices.len();
        self.vertices.extend_from_slice(&other.vertices);
        for index in &other.indices {
            self.indices.push(*index + off);
        }
    }

    pub fn stream(&mut self) -> BaseStream<'_> {
        BaseStream {
            shape: self,
            index: 0,
        }
    }

    pub fn draw<F: Fn(&mut InputVertex)>(&self, ctx: &mut impl RenderContext, vertex_function: F) {
        let z = ctx.next_z();
        ctx.controller().push_raw(
            &self.vertices,
            &self.indices,
            self.flags & SF_TEXTURE == SF_TEXTURE,
            Some(|v: &mut InputVertex| {
                v.pos.2 = z;

                vertex_function(v);
            }),
        );
    }

    pub fn draw_at<F: Fn(&mut InputVertex)>(
        &self,
        ctx: &mut impl RenderContext,
        area: &SimpleRect,
        vertex_function: F,
    ) {
        let z = ctx.next_z();
        ctx.controller().push_raw(
            &self.vertices,
            &self.indices,
            self.flags & SF_TEXTURE == SF_TEXTURE,
            Some(|v: &mut InputVertex| {
                let point = Vec2::new(v.pos.0, v.pos.1);
                let remapped = geom::remap_point(point, &self.extent, area);
                v.pos.0 = remapped.x;
                v.pos.1 = remapped.y;
                v.pos.2 = z;

                vertex_function(v);
            }),
        );
    }
}

pub trait VertexStream: Sized {
    fn shape(&mut self) -> &mut Shape;

    fn next(&mut self) -> Option<&mut InputVertex>;

    fn compute(&mut self) {
        while let Some(_) = self.next() {}
        self.shape().recompute();
    }

    fn transform(self, transform: Transform) -> TransformStep<Self> {
        TransformStep {
            base: self,
            transform,
        }
    }

    fn set_transform(self, transform: Transform) -> TransformSetStep<Self> {
        TransformSetStep {
            base: self,
            transform,
        }
    }

    fn translate(self, offset: Vec2) -> TranslateStep<Self> {
        TranslateStep { base: self, offset }
    }

    fn scale(self, scale: Vec2) -> ScaleStep<Self> {
        ScaleStep { base: self, scale }
    }

    fn rotate(self, angle: f32) -> RotateStep<Self> {
        RotateStep {
            base: self,
            angle_radians: angle,
        }
    }

    fn set_origin(self, origin: Vec2) -> OriginSetStep<Self> {
        OriginSetStep {
            base: self,
            new_origin: origin,
        }
    }

    fn change_origin(self, delta: Vec2) -> OriginChangeStep<Self> {
        OriginChangeStep {
            base: self,
            new_origin: delta,
        }
    }

    fn crop(self, draw_area: SimpleRect, crop_area: SimpleRect) -> CropStep<Self> {
        CropStep {
            base: self,
            crop_area,
            draw_area,
        }
    }

    fn color(self, color: RgbColor) -> ColorStep<Self> {
        ColorStep { base: self, color }
    }

    fn texture(self, texture: GLuint) -> TextureStep<Self> {
        TextureStep {
            base: self,
            texture,
        }
    }

    fn uv(self, uv: Vec4) -> UvStep<Self> {
        UvStep { base: self, uv }
    }
}

pub struct BaseStream<'a> {
    shape: &'a mut Shape,
    index: usize,
}

impl VertexStream for BaseStream<'_> {
    fn shape(&mut self) -> &mut Shape {
        self.shape
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        self.index += 1;
        self.shape.vertices.get_mut(self.index - 1)
    }
}
