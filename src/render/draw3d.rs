use std::sync::Arc;
use crate::render::batch2d::{Vertex2D, VertexGroup};
use crate::render::color::{Gradient, RGB};
use crate::render::text::Font;

pub struct Draw3D {
    canvas: [f32; 6],
    size: [f32; 2],
    color: Gradient<RGB, f32>,
    font: Arc<Font>,
    vertices: VertexGroup<Vertex2D>,
    use_cam: bool,
    frame: u64,
    dpi: f32
}