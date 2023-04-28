use std::cell::RefCell;
use std::rc::Rc;

use mvutils::utils::XTraIMath;

use crate::render::batch::{BatchController2D, Vertex2D, VertexGroup};
use crate::render::color::{Color, RGB};
use crate::render::shared::{RenderProcessor2D, Shader};

use super::color::Gradient;
use super::shared::TextureRegion;
use super::text::Font;

pub struct Draw2D {
    canvas: [f32; 6],
    size: [f32; 2],
    color: Gradient<RGB, f32>,
    font: Rc<Font>,
    batch: BatchController2D,
    vertices: VertexGroup<Vertex2D>,
    use_cam: bool,
    chroma_tilt: f32,
    chroma_compress: f32,
    frame: u64,
    screen_resolution: (i32, i32),
    dpi: f32
}

#[allow(clippy::too_many_arguments)]
impl Draw2D {
    pub(crate) fn new(shader: Rc<RefCell<Shader>>, font: Rc<Font>, width: i32, height: i32, res: (i32, i32), dpi: f32) -> Self {
        Draw2D {
            canvas: [0.0, 0.0, width as f32, height as f32, 0.0, 0.0],
            size: [width as f32, height as f32],
            color: Gradient::new(Color::<RGB, f32>::white()),
            font,
            batch: BatchController2D::new(shader, 10000),
            vertices: VertexGroup::new(),
            use_cam: true,
            chroma_tilt: -0.5,
            chroma_compress: 1.0,
            frame: 0,
            screen_resolution: res,
            dpi
        }
    }

    pub(crate) fn get_default_font(ctx: *mut Draw2D) -> Rc<Font> {
        unsafe { ctx.as_ref().unwrap().font.clone() }
    }

    pub(crate) fn render(&mut self, processor: &impl RenderProcessor2D) {
        self.frame += 1;
        self.batch.render(processor);
    }

    pub(crate) fn resize(&mut self, width: i32, height: i32) {
        self.size[0] = width as f32;
        self.size[1] = height as f32;
    }

    pub fn reset_canvas(&mut self) {
        self.canvas[0] = 0.0;
        self.canvas[1] = 0.0;
        self.canvas[2] = self.size[0];
        self.canvas[3] = self.size[1];
        self.canvas[4] = 0.0;
        self.canvas[5] = 0.0;
    }

    pub fn style_canvas(&mut self, style: CanvasStyle, radius: f32) {
        self.canvas[4] = style.id();
        self.canvas[5] = radius;
    }

    pub fn canvas(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.canvas[0] = x as f32;
        self.canvas[1] = y as f32;
        self.canvas[2] = width as f32;
        self.canvas[3] = height as f32;
    }

    pub fn reset_color(&mut self) {
        self.raw_rgba(1.0, 1.0, 1.0, 1.0);
    }

    pub fn color(&mut self, color: Color<RGB, f32>) {
        self.color.copy_color(color);
    }

    pub fn get_mut_gradient(&mut self) -> &mut Gradient<RGB, f32> {
        &mut self.color
    }

    pub fn rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.color.set_all(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0); 
    }

    pub fn raw_rgba(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color.set_all(r, g, b, a);
    }

    pub fn tri(&mut self) {
        self.vertices.get_mut(0).set([100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 800.0, 600.0, 0.0, 0.0, 0.0]);
        self.vertices.get_mut(1).set([200.0, 100.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 800.0, 600.0, 0.0, 0.0, 0.0]);
        self.vertices.get_mut(2).set([150.0, 200.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 800.0, 600.0, 0.0, 0.0, 0.0]);
        self.vertices.set_len(3);
        self.batch.add_vertices(&self.vertices);
    }

    pub fn font(&mut self, font: Rc<Font>) {
        self.font = font;
    }

    pub fn use_camera(&mut self, use_camera: bool) {
        self.use_cam = use_camera;
    }

    pub fn chroma_tilt(&mut self, tilt: f32) {
        self.chroma_tilt = tilt;
    }

    pub fn chroma_compress(&mut self, compress: f32) {
        self.chroma_compress = compress;
    }

    pub fn chroma_stretch(&mut self, stretch: f32) {
        self.chroma_compress = 1.0 / stretch;
    }

    pub fn triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32) {
        self.triangle_origin_rotated(x1, y1, x2, y2, x3, y3, 0.0, 0, 0)
    }

    pub fn triangle_rotated(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, rotation: f32) {
        self.triangle_origin_rotated(x1, y1, x2, y2, x3, y3, rotation, (x1 + x2 + x3) / 3, (y1 + y2 + y3) / 3);
    }

    pub fn triangle_origin_rotated(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, rotation: f32, rx: i32, ry: i32) {
        let rad_rot = rotation.to_radians();
        self.vertices.get_mut(0).set_data(x1 as f32, y1 as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data(x2 as f32, y2 as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data(x3 as f32, y3 as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(2), self.canvas, self.use_cam);
        self.vertices.set_len(3);
        self.batch.add_vertices(&self.vertices);
    }

    pub fn rectangle(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.rectangle_origin_rotated(x, y, width, height, 0.0, 0, 0);
    }

    pub fn rectangle_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, rotation: f32) {
        self.rectangle_origin_rotated(x, y, width, height, rotation, x + width / 2, y + height / 2);
    }

    pub fn rectangle_origin_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, rotation: f32, rx: i32, ry: i32) {
        let rad_rot = rotation.to_radians();
        self.vertices.get_mut(0).set_data(x as f32, (y + height) as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data(x as f32, y as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data((x + width) as f32, y as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(2), self.canvas, self.use_cam);
        self.vertices.get_mut(3).set_data((x + width) as f32, (y + height) as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(3), self.canvas, self.use_cam);
        self.vertices.set_len(4);
        self.batch.add_vertices(&self.vertices);
    }

    pub fn void_rectangle(&mut self, x: i32, y: i32, width: i32, height: i32, thickness: i32) {
        self.void_rectangle_origin_rotated(x, y, width, height, thickness, 0.0, 0, 0);
    }

    pub fn void_rectangle_roteted(&mut self, x: i32, y: i32, width: i32, height: i32, thickness: i32, rotation: f32) {
        self.void_rectangle_origin_rotated(x, y, width, height, thickness, rotation, x + width / 2, y + height / 2);
    }

    pub fn void_rectangle_origin_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, thickness: i32, rotation: f32, rx: i32, ry: i32) {
        self.rectangle_origin_rotated(x, y, width, thickness, rotation, rx, ry);
        self.rectangle_origin_rotated(x, y + thickness, thickness, height - 2 * thickness, rotation, rx, ry);
        self.rectangle_origin_rotated(x, y + height - thickness, width, thickness, rotation, rx, ry);
        self.rectangle_origin_rotated(x + width - thickness, y + thickness, thickness, height - 2 * thickness, rotation, rx, ry);
    }

    pub fn rounded_rectangle(&mut self, x: i32, y: i32, width: i32, height: i32, radius: i32, precision: f32) {
        self.rounded_rectangle_origin_rotated(x, y, width, height, radius, precision, 0.0, 0, 0);
    }

    pub fn rounded_rectangle_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, radius: i32, precision: f32, rotation: f32) {
        self.rounded_rectangle_origin_rotated(x, y, width, height, radius, precision, rotation, width / 2, height / 2);
    }

    pub fn rounded_rectangle_origin_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, radius: i32, precision: f32, rotation: f32, rx: i32, ry: i32) {
        self.rectangle_origin_rotated(x, y + radius, width, height - 2 * radius, rotation, rx, ry);
        self.rectangle_origin_rotated(x + radius, y, width - 2 * radius, radius, rotation, rx, ry);
        self.rectangle_origin_rotated(x + radius, y + height - radius, width - 2 * radius, radius, rotation, rx, ry);
        self.arc_origin_rotated(x + radius, y + radius, radius, 90, 180, precision, rotation, rx, ry);
        self.arc_origin_rotated(x + radius, y + height - radius, radius, 90, 90, precision, rotation, rx, ry);
        self.arc_origin_rotated(x + width - radius, y + radius, radius, 90, 270, precision, rotation, rx, ry);
        self.arc_origin_rotated(x + width - radius, y + height - radius, radius, 90, 0, precision, rotation, rx, ry);
    }

    pub fn triangular_rectangle(&mut self, x: i32, y: i32, width: i32, height: i32, radius: i32) {
        self.triangular_rectangle_origin_rotated(x, y, width, height, radius, 0.0, 0, 0);
    }

    pub fn triangular_rectangle_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, radius: i32, rotation: f32) {
        self.triangular_rectangle_origin_rotated(x, y, width, height, radius, rotation, width / 2, height / 2);
    }

    pub fn triangular_rectangle_origin_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, radius: i32, rotation: f32, rx: i32, ry: i32) {
        self.rectangle_origin_rotated(x, y + radius, width, height - 2 * radius, rotation, rx, ry);
        self.rectangle_origin_rotated(x + radius, y, width - 2 * radius, radius, rotation, rx, ry);
        self.rectangle_origin_rotated(x + radius, y + height - radius, width - 2 * radius, radius, rotation, rx, ry);
        self.triangle_origin_rotated(x + radius, y + radius, x, y + radius, x + radius, y, rotation, rx, ry);
        self.triangle_origin_rotated(x, y + height - radius, x + radius, y + height - radius, x + radius, y + height, rotation, rx, ry);
        self.triangle_origin_rotated(x + width - radius, y + height, x + width - radius, y + height - radius, x + width, y + height - radius, rotation, rx, ry);
        self.triangle_origin_rotated(x + width, y + radius, x + width - radius, y + radius, x + width - radius, y, rotation, rx, ry);
    }

    pub fn void_rounded_rectangle(&mut self, x: i32, y: i32, width: i32, height: i32, thickness: i32, radius: i32, precision: f32) {
        self.void_rounded_rectangle_origin_rotated(x, y, width, height, thickness, radius, precision, 0.0, 0, 0);
    }

    pub fn void_rounded_rectangle_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, thickness: i32, radius: i32, precision: f32, rotation: f32) {
        self.void_rounded_rectangle_origin_rotated(x, y, width, height, thickness, radius, precision, rotation, width / 2, height / 2);
    }

    pub fn void_rounded_rectangle_origin_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, thickness: i32, radius: i32, precision: f32, rotation: f32, rx: i32, ry: i32) {
        self.rectangle_origin_rotated(x + radius, y, width - 2 * radius, thickness, rotation, rx, ry);
        self.rectangle_origin_rotated(x + radius, y + height - thickness, width - 2 * radius, thickness, rotation, rx, ry);
        self.rectangle_origin_rotated(x, y + radius, thickness, height - 2 * radius, rotation, rx, ry);
        self.rectangle_origin_rotated(x + width - thickness, y + radius, thickness, height - 2 * radius, rotation, rx, ry);
        self.void_arc_origin_rotated(x + radius, y + radius, radius - thickness / 2, thickness, 90, 180, precision, rotation, rx, ry);
        self.void_arc_origin_rotated(x + radius, y + height - radius, radius - thickness / 2, thickness, 90, 90, precision, rotation, rx, ry);
        self.void_arc_origin_rotated(x + width - radius, y + radius, radius - thickness / 2, thickness, 90, 270, precision, rotation, rx, ry);
        self.void_arc_origin_rotated(x + width - radius, y + height - radius, radius - thickness / 2, thickness, 90, 0, precision, rotation, rx, ry);
    }

    pub fn void_triangular_rectangle(mut self, x:i32, y: i32, width: i32, height: i32, thickness: i32, radius: i32) {
        self.void_triangular_rectangle_origin_rotated(x, y, width, height, thickness, radius, 0.0, 0, 0);
    }

    pub fn void_triangular_rectangle_rotated(&mut self, x:i32, y: i32, width: i32, height: i32, thickness: i32, radius: i32, rotation: f32) {
        self.void_triangular_rectangle_origin_rotated(x, y, width, height, thickness, radius, rotation, width / 2, height / 2);
    }

    pub fn void_triangular_rectangle_origin_rotated(&mut self, x:i32, y: i32, width: i32, height: i32, thickness: i32, radius: i32, rotation: f32, rx: i32, ry: i32) {
        self.rectangle_origin_rotated(x + radius, y, width - 2 * radius, thickness, rotation, rx, ry);
        self.rectangle_origin_rotated(x + radius, y + height - thickness, width - 2 * radius, thickness, rotation, rx, ry);
        self.rectangle_origin_rotated(x, y + radius, thickness, height - 2 * radius, rotation, rx, ry);
        self.rectangle_origin_rotated(x + width - thickness, y + radius, thickness, height - 2 * radius, rotation, rx, ry);
        let rad_rotation: f32 = rotation.to_radians();

        self.vertices.get_mut(0).set_data(x as f32, (y + height - radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data((x + radius) as f32, (y + height) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data((x + radius) as f32, (y + height - thickness) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(2), self.canvas, self.use_cam);
        self.vertices.get_mut(3).set_data((x + thickness) as f32, (y + height - radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(3), self.canvas, self.use_cam);
        self.vertices.set_len(4);
        self.batch.add_vertices(&self.vertices);
        
        self.vertices.get_mut(0).set_data((x + width - radius) as f32, (y + height - radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data((x + width - radius) as f32, (y + height - radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data((x + width) as f32, (y + height - radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(2), self.canvas, self.use_cam);
        self.vertices.get_mut(3).set_data((x + width - thickness) as f32, (y + height - radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(3), self.canvas, self.use_cam);
        self.batch.add_vertices(&self.vertices);

        self.vertices.get_mut(0).set_data((x + width - radius) as f32, y as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data((x + width - radius) as f32, (y + thickness) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data((x + width - thickness) as f32, (y + radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(2), self.canvas, self.use_cam);
        self.vertices.get_mut(3).set_data((x + width) as f32, (y + radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(3), self.canvas, self.use_cam);
        self.batch.add_vertices(&self.vertices);

        self.vertices.get_mut(0).set_data(x as f32, (y + radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data((x + thickness) as f32, (y + radius) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data((x + radius) as f32, (y + thickness) as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(2), self.canvas, self.use_cam);
        self.vertices.get_mut(3).set_data((x + radius) as f32, y as f32, 0.0, rad_rotation, rx as f32, ry as f32, self.color.get(3), self.canvas, self.use_cam);
        self.batch.add_vertices(&self.vertices);
    }

    pub fn circle(&mut self, x: i32, y: i32, radius: i32, precision: f32) {
        self.ellipse_origin_rotated(x, y, radius, radius, precision, 0.0, 0, 0);
    }

    pub fn circle_rotated(&mut self, x: i32, y: i32 , radius: i32, precision: f32, rotation: f32) {
        self.ellipse_origin_rotated(x, y, radius, radius, precision, rotation, x, y);
    }

    pub fn circle_origin_rotated(&mut self, x: i32, y: i32, radius: i32, precision: f32, rotation: f32, rx: i32, ry: i32) {
        self.ellipse_origin_rotated(x, y, radius, radius, precision, rotation, rx, ry);
    }

    pub fn ellipse(&mut self, x: i32, y: i32, radius_x: i32, radius_y: i32, precision: f32) {
        self.ellipse_origin_rotated(x, y, radius_x, radius_y, precision, 0.0, 0, 0);
    }

    pub fn ellipse_rotated(&mut self, x: i32, y: i32, radius_x: i32, radius_y: i32, precision: f32, rotation: f32) {
        self.ellipse_origin_rotated(x, y, radius_x, radius_y, precision, rotation, x, y)
    }

    pub fn ellipse_origin_rotated(&mut self, x: i32, y: i32, radius_x: i32, radius_y: i32, precision: f32, rotation: f32, rx: i32, ry: i32) {
        let step = std::f32::consts::TAU / precision;
        let rad_rot = rotation.to_radians();
        let mut i = 0.0;
        self.vertices.set_len(3);
        while i < std::f32::consts::TAU {
            self.vertices.get_mut(0).set_data(x as f32 + radius_x as f32 * i.cos(), y as f32 + radius_y as f32 * i.sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
            self.vertices.get_mut(1).set_data(x as f32 + radius_x as f32 * (i + step).cos(), y as f32 + radius_y as f32 * (i + step).sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
            self.vertices.get_mut(2).set_data(x as f32, y as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
            self.batch.add_vertices(&self.vertices);
            i += step;
        }
    }

    pub fn void_circle(&mut self, x: i32, y: i32, radius: i32, thickness: i32, precision: f32) {
        self.void_arc_origin_rotated(x, y, radius, thickness, 360, 0, precision, 0.0, 0, 0);
    }
    
    pub fn void_circle_rotated(&mut self, x: i32, y: i32, radius: i32, thickness: i32, precision: f32, rotation: f32) {
        self.void_arc_origin_rotated(x, y, radius, thickness, 360, 0, precision, rotation, x, y);
    }

    pub fn void_circle_origin_rotated(&mut self, x: i32, y: i32, radius: i32, thickness: i32, precision: f32, rotation: f32, rx: i32, ry: i32) {
        self.void_arc_origin_rotated(x, y, radius, thickness, 360, 0, precision, rotation, rx, ry);
    }

    pub fn arc(&mut self, x: i32, y: i32, radius: i32, range: i32, start: i32, precision: f32) {
        self.arc_origin_rotated(x, y, radius, range, start, precision, 0.0, 0, 0);
    }

    pub fn arc_rotated(&mut self, x: i32, y: i32, radius: i32, range: i32, start: i32, precision: f32, rotation: f32) {
        self.arc_origin_rotated(x, y, radius, range, start, precision, rotation, x, y);
    }

    pub fn arc_origin_rotated(&mut self, x: i32, y: i32, radius: i32, range: i32, start: i32, precision: f32, rotation: f32, rx: i32, ry: i32)  {
        let r_range = std::f32::consts::TAU - (range as f32).to_radians();
        let step = std::f32::consts::TAU / precision;
        let rad_rot = rotation.to_radians();
        let start = (start as f32).to_radians();
        let mut i = start;
        self.vertices.set_len(3);
        while i < std::f32::consts::TAU - r_range + start {
            self.vertices.get_mut(0).set_data(x as f32 + radius as f32 * i.cos(), y as f32 + radius as f32 * i.sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
            self.vertices.get_mut(1).set_data(x as f32 + radius as f32 * (i + step).cos(), y as f32 + radius as f32 * (i + step).sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
            self.vertices.get_mut(2).set_data(x as f32, y as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
            self.batch.add_vertices(&self.vertices);
            i += step;
        }
    }

    pub fn void_arc(&mut self, x: i32, y: i32, radius: i32, thickness: i32, range: i32, start: i32, precision: f32) {
        self.void_arc_origin_rotated(x, y, radius, thickness, range, start, precision, 0.0, 0, 0);
    }

    pub fn void_arc_rotated(&mut self, x: i32, y: i32, radius: i32, thickness: i32, range: i32, start: i32, precision: f32, rotation: f32) {
        self.void_arc_origin_rotated(x, y, radius, thickness, range, start, precision, rotation, x, y);
    }

    pub fn void_arc_origin_rotated(&mut self, x: i32, y: i32, radius: i32, thickness: i32, range: i32, start: i32, precision: f32, rotation: f32, rx: i32, ry: i32) {
        let r_radius = radius - (thickness as f32 / 2.0).ceil() as i32;
        let r_range = std::f32::consts::TAU - (range as f32).to_radians();
        let step = std::f32::consts::TAU / precision;
        let rad_rot = rotation.to_radians();

        let start = (start as f32).to_radians();
        let mut i = start;
        self.vertices.set_len(4);
        while i < std::f32::consts::TAU - r_range + start {
            self.vertices.get_mut(0).set_data(x as f32 + r_radius as f32 * i.cos(), y as f32 + r_radius as f32 * i.sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
            self.vertices.get_mut(1).set_data(x as f32 + (r_radius + thickness) as f32 * i.cos(), y as f32 + (r_radius + thickness) as f32 * i.sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
            self.vertices.get_mut(2).set_data(x as f32 + (r_radius + thickness) as f32 * (i + step).cos(), y as f32 + (r_radius + thickness) as f32 * (i + step).sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
            self.vertices.get_mut(3).set_data(x as f32 + r_radius as f32 * (i + step).cos(), y as f32 + r_radius as f32 * (i + step).sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
            self.batch.add_vertices(&self.vertices);
            i += step;
        }
    }

    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32) {
        self.line_origin_rotated(x1, y1, x2, y2, thickness, 0.0, 0, 0);
    }

    pub fn line_rotated(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, rotation: f32) {
        self.line_origin_rotated(x1, y1, x2, y2, thickness, rotation, (x1 + x2) / 2, (y1 + y2) / 2);
    }

    pub fn line_origin_rotated(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, rotation: f32, rx: i32, ry: i32) {
        let theta = (x2 as f32 - x1 as f32).atan2(y2 as f32 - y1 as f32);
        let theta_sin = theta.sin() * (thickness as f32 / 2.0);
        let theta_cos = theta.cos() * (thickness as f32 / 2.0);
        let rad_rot = rotation.to_radians();

        self.vertices.get_mut(0).set_data(x1 as f32 - theta_cos, y1 as f32 + theta_sin, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(0), self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data(x1 as f32 + theta_cos, y1 as f32 - theta_sin, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data(x2 as f32 + theta_cos, y2 as f32 - theta_sin, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(2), self.canvas, self.use_cam);
        self.vertices.get_mut(3).set_data(x2 as f32 - theta_cos, y2 as f32 + theta_sin, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(3), self.canvas, self.use_cam);
        self.vertices.set_len(4);
        self.batch.add_vertices(&self.vertices);
    }

    pub fn image(&mut self, x: i32, y: i32, width: i32, height: i32, texture: Rc<TextureRegion>) {
        self.image_origin_rotated(x, y, width, height, texture, 0.0, 0, 0);
    }

    pub fn image_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, texture: Rc<TextureRegion>, rotation: f32) {
        self.image_origin_rotated(x, y, width, height, texture, rotation, (x + width) / 2, (y + height) / 2);
    }

    pub fn image_origin_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, texture: Rc<TextureRegion>, rotation: f32, rx: i32, ry: i32) {
        let rad_rot = rotation.to_radians();

        let tex = self.batch.add_texture(texture.parent(), 4);
        let uv = texture.get_uv();

        self.vertices.get_mut(0).set_texture_data(x as f32, (y + height) as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(0), uv[0], uv[3], tex, self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_texture_data(x as f32, y as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), uv[0], uv[2], tex, self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_texture_data((x + width) as f32, y as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(2), uv[1], uv[2], tex, self.canvas, self.use_cam);
        self.vertices.get_mut(3).set_texture_data((x + width) as f32, (y + height) as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(3), uv[1], uv[3], tex, self.canvas, self.use_cam);
        self.vertices.set_len(4);
        self.batch.add_vertices(&self.vertices);
    }

    pub fn image_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, texture: Rc<TextureRegion>) {
        let theta = ((x2 - x1) as f32).atan2((y2 - y1) as f32);
        let theta_sin = thickness as f32 * theta.sin();
        let theta_cos = thickness as f32 * theta.cos();

        let tex = self.batch.add_texture(texture.parent(), 4);
        let uv = texture.get_uv();

        self.vertices.get_mut(0).set_norot_texture_data(x1 as f32 - theta_cos, y1 as f32 + theta_sin, 0.0, self.color.get(0), uv[0], uv[3], tex, self.canvas, self.use_cam);
        self.vertices.get_mut(0).set_norot_texture_data(x1 as f32 + theta_cos, y1 as f32 - theta_sin, 0.0, self.color.get(1), uv[0], uv[2], tex, self.canvas, self.use_cam);
        self.vertices.get_mut(0).set_norot_texture_data(x2 as f32 + theta_cos, y2 as f32 - theta_sin, 0.0, self.color.get(2), uv[1], uv[2], tex, self.canvas, self.use_cam);
        self.vertices.get_mut(0).set_norot_texture_data(x2 as f32 - theta_cos, y2 as f32 + theta_sin, 0.0, self.color.get(3), uv[1], uv[3], tex, self.canvas, self.use_cam);
        self.vertices.set_len(4);
        self.batch.add_vertices(&self.vertices);
    }

    pub fn text(&mut self, chroma: bool, x: i32, y: i32, height: i32, text: &str) {
        self.custom_text_origin_rotated(chroma, x, y, height, text, self.font.clone(), 0.0, 0, 0)
    }

    pub fn text_rotated(&mut self, chroma: bool, x: i32, y: i32, height: i32, text: &str, rotation: f32) {
        let width = self.font.get_metrics(text).width(height);
        self.custom_text_origin_rotated(chroma, x, y, height, text, self.font.clone(), rotation, x + width / 4, y + height / 4)
    }

    pub fn text_origin_rotated(&mut self, chroma: bool, x: i32, y: i32, height: i32, text: &str, rotation: f32, rx: i32, ry: i32) {
        self.custom_text_origin_rotated(chroma, x, y, height, text, self.font.clone(), rotation, rx, ry)
    }

    pub fn custom_text(&mut self, chroma: bool, x: i32, y: i32, height: i32, text: &str, font: Rc<Font>) {
        self.custom_text_origin_rotated(chroma, x, y, height, text, font, 0.0, 0, 0);
    }

    pub fn custom_text_rotated(&mut self, chroma: bool, x: i32, y: i32, height: i32, text: &str, font: Rc<Font>, rotation: f32) {
        let width = font.get_metrics(&text).width(height);
        self.custom_text_origin_rotated(chroma, x, y, height, text, font, rotation, x + width / 4, y + height / 4);
    }

    pub fn custom_text_origin_rotated(&mut self, chroma: bool, x: i32, y: i32, height: i32, text: &str, font: Rc<Font>, rotation: f32, rx: i32, ry: i32) {
        let mut char_x = 0;
        let rad_rot = rotation.to_radians();

        for c in text.chars() {
            if !font.supports(c) {
                continue;
            }
            let glyph = font.get_glyph(c);
            let y_off = glyph.get_y_offset(height) - height + glyph.get_height(height);

            let ax = x + char_x + glyph.get_x_offset(height);
            let ay = y - y_off;
            let ax2 = x + char_x + glyph.get_x_offset(height) + glyph.get_width(height);
            let ay2 = y + glyph.get_height(height)  - y_off;

            if chroma {
                self.reset_color();
                let a = self.get_hue(ax, ay).overlap(0, 359) as f32;
                let b = self.get_hue(ax2, ay).overlap(0, 359) as f32;
                let c = self.get_hue(ax, ay2).overlap(0, 359) as f32;
                let d = self.get_hue(ax2, ay2).overlap(0, 359) as f32;
                self.color.get_mut(0).copy_hue(a);
                self.color.get_mut(3).copy_hue(b);
                self.color.get_mut(1).copy_hue(c);
                self.color.get_mut(2).copy_hue(d);
            }

            char_x += glyph.get_x_advance(height);
            let uv = glyph.get_uv();

            let tex = self.batch.add_texture(font.get_texture(), 4);

            self.vertices.get_mut(0).set_texture_data(ax as f32, ay2 as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(0), uv[0], uv[3], tex, self.canvas, self.use_cam);
            self.vertices.get_mut(1).set_texture_data(ax as f32, ay as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(1), uv[0], uv[2], tex, self.canvas, self.use_cam);
            self.vertices.get_mut(2).set_texture_data(ax2 as f32, ay as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(2), uv[1], uv[2], tex, self.canvas, self.use_cam);
            self.vertices.get_mut(3).set_texture_data(ax2 as f32, ay2 as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color.get(3), uv[1], uv[3], tex, self.canvas, self.use_cam);
            self.vertices.set_len(4);
            self.batch.add_vertices(&self.vertices);
        }
        self.reset_color();
    }

    fn get_hue(&self, x: i32, y: i32) -> i32 {
        (((x as f32 * 180.0 / self.size[0]) + ((y as f32 * 180.0 / self.size[1]) * self.chroma_tilt) + (self.frame as f32)) * 5.0 * self.chroma_compress) as i32
    }

    //pub fn animatedText(TextAnimator animator) {
    //animatedText(animator, font, 0.0f, 0, 0);
    //}

    //pub fn animatedText(TextAnimator animator, BitmapFont font) {
    //animatedText(animator, font, 0.0f, 0, 0);
    //}

    //pub fn animatedText(TextAnimator animator, BitmapFont font, float rotation) {
    //int width = animator.getTotalWidth(font);
    //int height = animator.getHeight();
    //animatedText(animator, font, rotation, animator.getX() + width / 4, animator.getY() + height / 4);
    //}

    //pub fn animatedText(TextAnimator animator, BitmapFont font, float rotation, int rx, int ry) {
    //int charX = 0;
    //float radRotation = (float) Math.toRadians(rotation);

    //for (int i = 0; i < animator.getStates().length; i++) {
    //TextAnimation.TextState state = animator.getStates()[i];

    //char c = state.content;

    //if (c <= 31) continue;

    //Glyph glyph = font.getGlyph(c);

    //if (glyph == null) return;

    //int height = state.height;
    //int x = state.x;
    //int y = state.y;

    //int yOff = glyph.getYOffset(height) - (font.getMaxHeight(height) - glyph.getHeight(height));

    //float ax = x + charX + glyph.getXOffset(height);
    //float ay = y - yOff;
    //float ax2 = x + charX + glyph.getXOffset(height) + glyph.getWidth(height);
    //float ay2 = y + glyph.getHeight(height) - yOff;

    //charX += glyph.getXAdvance(height);

    //Vector2f[] uvs = glyph.getCoordinates();
    //float ux0 = uvs[0].x;
    //float ux1 = uvs[1].x;
    //float uy1 = uvs[0].y;
    //float uy0 = uvs[1].y;

    //int texID = window.getBatchController().addTexture(font.getBitmap(), isStripped);

    //window.getBatchController().addVertices(verts.set(
    //v1.put(ax, ay2, 0.0f, radRotation, rx, ry, state.color.r, state.color.g, state.color.b, state.color.a, ux0, uy0, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v2.put(ax, ay, 0.0f, radRotation, rx, ry, state.color.r, state.color.g, state.color.b, state.color.a, ux0, uy1, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v3.put(ax2, ay, 0.0f, radRotation, rx, ry, state.color.r, state.color.g, state.color.b, state.color.a, ux1, uy1, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v4.put(ax2, ay2, 0.0f, radRotation, rx, ry, state.color.r, state.color.g, state.color.b, state.color.a, ux1, uy0, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    //), useCamera, isStripped);
    //}
    //}

    pub fn screen_resolution(&self) -> (i32, i32) {
        self.screen_resolution
    }
    pub fn dpi(&self) -> f32 {
        self.dpi
    }
}

pub enum CanvasStyle {
    Square,
    Triangle,
    Round
}

impl CanvasStyle {
    pub(crate) fn id(&self) -> f32 {
        match self {
            CanvasStyle::Square => 0.0,
            CanvasStyle::Triangle => 1.0,
            CanvasStyle::Round => 2.0,
        }
    }
}