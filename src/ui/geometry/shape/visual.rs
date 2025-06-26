use crate::color::RgbColor;
use crate::math::vec::Vec4;
use crate::rendering::InputVertex;
use crate::ui::geometry::shape::{Shape, VertexStream};
use gl::types::GLuint;

pub struct ColorStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) color: RgbColor
}

impl<S: VertexStream> VertexStream for ColorStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        if let Some(p) = self.base.next() {
            p.has_texture = 0.0;
            p.color = self.color.as_vec4();
            
            Some(p)
        } else {
            None
        }
    }
}

pub struct TextureStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) texture: GLuint
}

impl<S: VertexStream> VertexStream for TextureStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        if let Some(p) = self.base.next() {
            p.has_texture = 1.0;
            p.texture = self.texture;
            
            Some(p)
        } else {
            None
        }
    }
}

pub struct UvStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) uv: Vec4
}

impl<S: VertexStream> VertexStream for UvStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        //gotta give chatgpt the credit

        let min_x = (self.shape().extent.x) as f32;
        let max_x = (self.shape().extent.x + self.shape().extent.width) as f32;
        let min_y = (self.shape().extent.y) as f32;
        let max_y = (self.shape().extent.y + self.shape().extent.height) as f32;
        
        if let Some(p) = self.base.next() {
            let width = max_x - min_x;
            let height = max_y - min_y;

            let ux = self.uv.x;
            let uy = self.uv.y;
            let uw = self.uv.z;
            let uh = self.uv.w;

            // Normalize position within triangle bounding box
            let norm_x = if width != 0.0 { (p.pos.0 - min_x) / width } else { 0.5 };
            let norm_y = if height != 0.0 { (p.pos.1 - min_y) / height } else { 0.5 };

            // Linearly interpolate within UV rect
            let u = ux + norm_x * uw;
            let v = uy + norm_y * uh;

            p.uv = (u, v);

            Some(p)
        } else {
            None
        }
    }
}