use log::warn;
use crate::rendering::InputVertex;
use crate::ui::geometry::shape::{Shape, VertexStream};
use crate::ui::geometry::SimpleRect;

pub struct CropStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) crop_area: SimpleRect,
    pub(crate) draw_area: SimpleRect
}

impl<S: VertexStream> VertexStream for CropStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        let point = self.base.next()?;

        warn!("CropStep on shapes are currently doing nothing cuz idk how that would work. Please use the vertex function and do the math yourself.");

        Some(point)
    }
}