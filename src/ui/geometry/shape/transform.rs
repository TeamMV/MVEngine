use crate::math::vec::Vec2;
use crate::rendering::{InputVertex, Transform};
use crate::ui::geometry::shape::{Shape, VertexStream};

pub struct TransformStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) transform: Transform,
}

impl<S: VertexStream> VertexStream for TransformStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        if let Some(p) = self.base.next() {
            let v = Vec2::new(p.pos.0, p.pos.1);
            let v = self.transform.apply_for_point(v);
            p.pos.0 = v.x;
            p.pos.1 = v.y;

            Some(p)
        } else {
            None
        }
    }
}

pub struct TransformSetStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) transform: Transform,
}

impl<S: VertexStream> VertexStream for TransformSetStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        if let Some(p) = self.base.next() {
            p.transform = self.transform.clone();

            Some(p)
        } else {
            None
        }
    }
}

pub struct TranslateStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) offset: Vec2,
}

impl<S: VertexStream> VertexStream for TranslateStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        if let Some(p) = self.base.next() {
            p.pos.0 += self.offset.x;
            p.pos.1 += self.offset.y;
            Some(p)
        } else {
            None
        }
    }
}

pub struct RotateStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) angle_radians: f32,
}

impl<S: VertexStream> VertexStream for RotateStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        if let Some(p) = self.base.next() {
            let cos = self.angle_radians.cos();
            let sin = self.angle_radians.sin();

            let x = p.pos.0;
            let y = p.pos.1;
            p.pos.0 = x * cos - y * sin;
            p.pos.1 = x * sin + y * cos;

            Some(p)
        } else {
            None
        }
    }
}

pub struct ScaleStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) scale: Vec2,
}

impl<S: VertexStream> VertexStream for ScaleStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        if let Some(p) = self.base.next() {
            p.pos.0 *= self.scale.x;
            p.pos.1 *= self.scale.y;

            Some(p)
        } else {
            None
        }
    }
}

pub struct OriginChangeStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) new_origin: Vec2,
}

impl<S: VertexStream> VertexStream for OriginChangeStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        if let Some(p) = self.base.next() {
            p.pos.0 += self.new_origin.x;
            p.pos.1 += self.new_origin.y;

            Some(p)
        } else {
            None
        }
    }
}

pub struct OriginSetStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) new_origin: Vec2,
}

impl<S: VertexStream> VertexStream for OriginSetStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        if let Some(p) = self.base.next() {
            p.pos.0 = self.new_origin.x;
            p.pos.1 = self.new_origin.y;

            Some(p)
        } else {
            None
        }
    }
}
