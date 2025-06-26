use crate::rendering::InputVertex;
use crate::ui::geometry::shape::{Shape, VertexStream};
use crate::ui::geometry::SimpleRect;

pub struct CropStep<S: VertexStream> {
    pub(crate) base: S,
    pub(crate) crop_area: SimpleRect
}

impl<S: VertexStream> VertexStream for CropStep<S> {
    fn shape(&mut self) -> &mut Shape {
        self.base.shape()
    }

    fn next(&mut self) -> Option<&mut InputVertex> {
        let point = self.base.next()?;

        let rect_x = self.crop_area.x as f32;
        let rect_y = self.crop_area.y as f32;
        let rect_w = self.crop_area.width as f32;
        let rect_h = self.crop_area.height as f32;

        let min_x = rect_x;
        let max_x = rect_x + rect_w;
        let min_y = rect_y;
        let max_y = rect_y + rect_h;

        let old_x = point.pos.0;
        let old_y = point.pos.1;

        point.pos.0 = old_x.clamp(min_x, max_x);
        point.pos.1 = old_y.clamp(min_y, max_y);

        if point.has_texture >= 1.0 {
            let tx = (point.pos.0 - min_x) / rect_w;
            let ty = (point.pos.1 - min_y) / rect_h;

            point.uv.0 = tx.clamp(0.0, 1.0);
            point.uv.1 = ty.clamp(0.0, 1.0);
        }

        Some(point)
    }
}