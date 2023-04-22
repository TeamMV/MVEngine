use std::cell::RefCell;
use std::rc::Rc;

use crate::render::batch::{BatchController2D, Vertex2D, VertexGroup};
use crate::render::color::{Color, RGB};
use crate::render::shared::{RenderProcessor2D, Shader};

pub struct Draw2D {
    canvas: [f32; 6],
    size: [f32; 2],
    color: Color<RGB, f32>,
    batch: BatchController2D,
    vertices: VertexGroup<Vertex2D>,
    use_cam: bool,
    chroma_tilt: f32,
    chroma_compress: f32,
    frame: u64
}

impl Draw2D {
    pub(crate) fn new(shader: Rc<RefCell<Shader>>, width: i32, height: i32) -> Self {
        Draw2D {
            canvas: [0.0, 0.0, width as f32, height as f32, 0.0, 0.0],
            size: [width as f32, height as f32],
            color: Color::<RGB, f32>::white(),
            batch: BatchController2D::new(shader, 10000),
            vertices: VertexGroup::new(),
            use_cam: true,
            chroma_tilt: -0.5,
            chroma_compress: 1.0,
            frame: 0
        }
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

    pub fn color(&mut self, color: &Color<RGB, f32>) {
        self.color.copy_of(color);
    }

    pub fn rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.color.normalize(r, g, b, a);
    }

    pub fn raw_rgba(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color.set(r, g, b, a);
    }

    pub fn tri(&mut self) {
        self.vertices.get_mut(0).set([100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 800.0, 600.0, 0.0, 0.0, 0.0]);
        self.vertices.get_mut(1).set([200.0, 100.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 800.0, 600.0, 0.0, 0.0, 0.0]);
        self.vertices.get_mut(2).set([150.0, 200.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 800.0, 600.0, 0.0, 0.0, 0.0]);
        self.vertices.set_len(3);
        self.batch.add_vertices(&self.vertices);
    }

    //pub fn font(BitmapFont font) {
    //    this.font = font;
    //}

    pub fn use_camera(&mut self, use_camera: bool) {
        self.use_cam = use_camera;
    }

    pub fn chroma_tilt(&mut self, tilt: f32) {
        self.chroma_tilt = tilt;
    }

    pub fn chromaCompress(&mut self, compress: f32) {
        self.chroma_compress = compress;
    }

    pub fn chromaStretch(&mut self, stretch: f32) {
        self.chroma_compress = 1.0 / stretch;
    }

    pub fn triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32) {
        self.triangle_rotated(x1, y1, x2, y2, x3, y3, 0.0);
    }

    pub fn triangle_rotated(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, rotation: f32) {
        self.triangle_origin_rotated(x1, y1, x2, y2, x3, y3, rotation, (x1 + x2 + x3) / 3, (y1 + y2 + y3) / 3);
    }

    pub fn triangle_origin_rotated(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, rotation: f32, rx: i32, ry: i32) {
        let rad_rot = rotation.to_radians();
        self.vertices.get_mut(0).set_data(x1 as f32, y1 as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data(x2 as f32, y2 as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data(x3 as f32, y3 as f32, 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
        self.vertices.set_len(3);
        self.batch.add_vertices(&self.vertices);
    }

    pub fn rectangle(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.rectangle_rotated(x, y, width, height, 0.0);
    }

    pub fn rectangle_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, rotation: f32) {
        self.rectangle_origin_rotated(x, y, width, height, rotation, x + width / 2, y + height / 2);
    }

    pub fn rectangle_origin_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, rotation: f32, originX: i32, originY: i32) {
        self.vertices.get_mut(0).set_data(x as f32, y as f32, 0.0, rotation.to_radians(), originX as f32, originY as f32, self.color, self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data(x as f32, y as f32, 0.0, rotation.to_radians(), originX as f32, originY as f32, self.color, self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data(x as f32, y as f32, 0.0, rotation.to_radians(), originX as f32, originY as f32, self.color, self.canvas, self.use_cam);
        self.vertices.get_mut(3).set_data(x as f32, y as f32, 0.0, rotation.to_radians(), originX as f32, originY as f32, self.color, self.canvas, self.use_cam);
        self.vertices.set_len(4);
        self.batch.add_vertices(&self.vertices);
    }

    pub fn void_rectangle(&mut self, x: i32, y: i32, width: i32, height: i32, thickness: i32) {
    self.void_rectangle_roteted(x, y, width, height, thickness, 0.0);
    }

    pub fn void_rectangle_roteted(&mut self, x: i32, y: i32, width: i32, height: i32, thickness: i32, rotation: f32) {
    self.void_rectangle_origin_rotated(x, y, width, height, thickness, rotation, x + width / 2, y + height / 2);
    }

    pub fn void_rectangle_origin_rotated(&mut self, x: i32, y: i32, width: i32, height: i32, thickness: i32, rotation: f32, originX: i32, originY: i32) {
    self.rectangle_origin_rotated(x, y, width, height, rotation, originX, originY);
        self.rectangle_origin_rotated(x, y + thickness, thickness, height - 2 * thickness, rotation, originX, originY);
    self.rectangle_origin_rotated(x, y + height - thickness, width, thickness, rotation, originX, originY);
    self.rectangle_origin_rotated(x + width - thickness, y + thickness, thickness, height - 2 * thickness, rotation, originX, originY);
    }

    pub fn rounded_rectangle(&mut self, x: i32, y: i32, width: i23, height: i32, radius: i32, precision: f32) {
    self.rounded_rectangle_rotated(x, y, width, height, radius, precision, 0.0);
    }

    pub fn rounded_rectangle_rotated(&mut self, int x, int y, int width, int height, int radius, float precision, float rotation) {
    roundedRectangle(x, y, width, height, radius, precision, rotation, width / 2, height / 2);
    }

    pub fn rounded_rectangle_origin_rotated(&mut self, int x, int y, int width, int height, int radius, float precision, float rotation, int originX, int originY) {
    rectangle(x, y + radius, width, height - 2 * radius, rotation, originX, originY);
    rectangle(x + radius, y, width - 2 * radius, radius, rotation, originX, originY);
    rectangle(x + radius, y + height - radius, width - 2 * radius, radius, rotation, originX, originY);
    arc(x + radius, y + radius, radius, 90, 180, precision, rotation, originX, originY);
    arc(x + radius, y + height - radius, radius, 90, 90, precision, rotation, originX, originY);
    arc(x + width - radius, y + radius, radius, 90, 270, precision, rotation, originX, originY);
    arc(x + width - radius, y + height - radius, radius, 90, 0, precision, rotation, originX, originY);
    }

    pub fn triangularRectangle(int x, int y, int width, int height, int radius) {
    triangularRectangle(x, y, width, height, radius, 0.0f);
    }

    pub fn triangularRectangle(int x, int y, int width, int height, int radius, float rotation) {
    triangularRectangle(x, y, width, height, radius, rotation, width / 2, height / 2);
    }

    pub fn triangularRectangle(int x, int y, int width, int height, int radius, float rotation, int originX, int originY) {
    rectangle(x, y + radius, width, height - 2 * radius, rotation, originX, originY);
    rectangle(x + radius, y, width - 2 * radius, radius, rotation, originX, originY);
    rectangle(x + radius, y + height - radius, width - 2 * radius, radius, rotation, originX, originY);
    triangle(x + radius, y + radius, x, y + radius, x + radius, y, rotation, originX, originY);
    triangle(x, y + height - radius, x + radius, y + height - radius, x + radius, y + height, rotation, originX, originY);
    triangle(x + width - radius, y + height, x + width - radius, y + height - radius, x + width, y + height - radius, rotation, originX, originY);
    triangle(x + width, y + radius, x + width - radius, y + radius, x + width - radius, y, rotation, originX, originY);
    }

    pub fn fnRoundedRectangle(int x, int y, int width, int height, int thickness, int radius, float precision) {
    fnRoundedRectangle(x, y, width, height, thickness, radius, precision, 0.0f);
    }

    pub fn fnRoundedRectangle(int x, int y, int width, int height, int thickness, int radius, float precision, float rotation) {
    fnRoundedRectangle(x, y, width, height, thickness, radius, precision, rotation, width / 2, height / 2);
    }

    pub fn fnRoundedRectangle(int x, int y, int width, int height, int thickness, int radius, float precision, float rotation, int originX, int originY) {
    rectangle(x + radius, y, width - 2 * radius, thickness, rotation, originX, originY);
    rectangle(x + radius, y + height - thickness, width - 2 * radius, thickness, rotation, originX, originY);
    rectangle(x, y + radius, thickness, height - 2 * radius, rotation, originX, originY);
    rectangle(x + width - thickness, y + radius, thickness, height - 2 * radius);
    fnArc(x + radius, y + radius, radius - thickness / 2, thickness, 90, 180, precision, rotation, originX, originY);
    fnArc(x + radius, y + height - radius, radius - thickness / 2, thickness, 90, 90, precision, rotation, originX, originY);
    fnArc(x + width - radius, y + radius, radius - thickness / 2, thickness, 90, 270, precision, rotation, originX, originY);
    fnArc(x + width - radius, y + height - radius, radius - thickness / 2, thickness, 90, 0, precision, rotation, originX, originY);
    }

    pub fn fnTriangularRectangle(int x, int y, int width, int height, int thickness, int radius) {
    fnTriangularRectangle(x, y, width, height, thickness, radius, 0.0f);
    }

    pub fn fnTriangularRectangle(int x, int y, int width, int height, int thickness, int radius, float rotation) {
    fnTriangularRectangle(x, y, width, height, thickness, radius, rotation, width / 2, height / 2);
    }

    pub fn fnTriangularRectangle(int x, int y, int width, int height, int thickness, int radius, float rotation, int originX, int originY) {
    rectangle(x + radius, y, width - 2 * radius, thickness, rotation, originX, originY);
    rectangle(x + radius, y + height - thickness, width - 2 * radius, thickness, rotation, originX, originY);
    rectangle(x, y + radius, thickness, height - 2 * radius, rotation, originX, originY);
    rectangle(x + width - thickness, y + radius, thickness, height - 2 * radius, rotation, originX, originY);
    float radRotation = (float) Math.toRadians(rotation);
    window.getBatchController().addVertices(verts.set(
    v1.put(x, y + height - radius, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v2.put(x + radius, y + height, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v3.put(x + radius, y + height - thickness, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v4.put(x + thickness, y + height - radius, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    ), useCamera, isStripped);

    window.getBatchController().addVertices(verts.set(
    v1.put(x + width - radius, y + height - thickness, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v2.put(x + width - radius, y + height, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v3.put(x + width, y + height - radius, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v4.put(x + width - thickness, y + height - radius, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    ), useCamera, isStripped);

    window.getBatchController().addVertices(verts.set(
    v1.put(x + width - radius, y, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v2.put(x + width - radius, y + thickness, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v3.put(x + width - thickness, y + radius, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v4.put(x + width, y + radius, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    ), useCamera, isStripped);

    window.getBatchController().addVertices(verts.set(
    v1.put(x, y + radius, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v2.put(x + thickness, y + radius, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v3.put(x + radius, y + thickness, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v4.put(x + radius, y, 0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    ), useCamera, isStripped);
    }

    pub fn circle(int x, int y, int radius, float precision) {
    circle(x, y, radius, precision, 0.0f);
    }

    pub fn circle(int x, int y, int radius, float precision, float rotation) {
    circle(x, y, radius, precision, rotation, x, y);
    }

    pub fn circle(int x, int y, int radius, float precision, float rotation, int originX, int originY) {
    ellipse(x, y, radius, radius, precision, rotation, originX, originY);
    }

    pub fn ellipse(int x, int y, int radiusX, int radiusY, float precision) {
    ellipse(x, y, radiusX, radiusY, precision, 0.0f);
    }

    pub fn ellipse(int x, int y, int radiusX, int radiusY, float precision, float rotation) {
    ellipse(x, y, radiusX, radiusY, precision, rotation, x, y);
    }

    pub fn ellipse(int x, int y, int radiusX, int radiusY, float precision, float rotation, int originX, int originY) {
    double tau = Math.PI * 2.0;
    double step = tau / precision;
    float radRotation = (float) Math.toRadians(rotation);
    for (double i = 0.0; i < tau; i += step) {
    window.getBatchController().addVertices(verts.set(
    v1.put((float) (x + (radiusX * Math.cos(i))), (float) (y + (radiusY * Math.sin(i))), 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v2.put((float) (x + (radiusX * Math.cos(i + step))), (float) (y + (radiusY * Math.sin(i + step))), 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v3.put(x, y, 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    ), useCamera, isStripped);
    }
    }

    pub fn fnCircle(int x, int y, int radius, int thickness, float precision) {
    fnCircle(x, y, radius, thickness, precision, 0.0f);
    }

    pub fn fnCircle(int x, int y, int radius, int thickness, float precision, float rotation) {
    fnCircle(x, y, radius, thickness, precision, rotation, x, y);
    }

    pub fn fnCircle(int x, int y, int radius, int thickness, float precision, float rotation, int originX, int originY) {
    fnArc(x, y, radius, thickness, 360, 0, precision, rotation, originX, originY);
    }

    pub fn arc(int x, int y, int radius, int range, int start, float precision) {
    arc(x, y, radius, range, start, precision, 0.0f);
    }

    pub fn arc(int x, int y, int radius, int range, int start, float precision, float rotation) {
    arc(x, y, radius, range, start, precision, rotation, x, y);
    }

    pub fn arc(int x, int y, int radius, int range, int start, float precision, float rotation, int originX, int originY) {
    double tau = Math.PI * 2.0;
    double rRange = Math.PI * 2.0 - Math.toRadians(range);
    double step = tau / precision;
    float radRotation = (float) Math.toRadians(rotation);
    for (double i = Math.toRadians(start); i < tau - rRange + Math.toRadians(start); i += step) {
    window.getBatchController().addVertices(verts.set(
    v1.put((float) (x + (radius * Math.cos(i))), (float) (y + (radius * Math.sin(i))), 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v2.put((float) (x + (radius * Math.cos(i + step))), (float) (y + (radius * Math.sin(i + step))), 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    v3.put(x, y, 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, 0.0f, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    ), useCamera, isStripped);
    }
    }

    pub fn void_arc(&mut self, x: i32, y: i32, radius: i32, thickness: i32, range: i32, start: i32, precision: f32) {
        self.void_arc_rotated(x, y, radius, thickness, range, start, precision, 0.0);
    }

    pub fn void_arc_rotated(&mut self, x: i32, y: i32, radius: i32, thickness: i32, range: i32, start: i32, precision: f32, rotation: f32) {
        self.void_arc_origin_rotated(x, y, radius, thickness, range, start, precision, rotation, x, y);
    }

    pub fn void_arc_origin_rotated(&mut self, x: i32, y: i32, radius: i32, thickness: i32, range: i32, start: i32, precision: f32, rotation: f32, rx: i32, ry: i32) {
        let r_radius = radius - (thickness / 2) - 1;
        let r_range = std::f32::consts::TAU - (range as f32).to_radians();
        let step = std::f32::consts::TAU / precision;
        let rad_rot = rotation.to_radians();

        let start = (start as f32).to_radians();
        let mut i = start;
        self.vertices.set_len(4);
        while i < std::f32::consts::TAU - r_range + start {
            self.vertices.get_mut(0).set_data(x as f32 + r_radius as f32 * i.cos(), y as f32 + r_radius as f32 * i.sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
            self.vertices.get_mut(1).set_data(x as f32 + (r_radius + thickness) as f32 * i.cos(), y as f32 + (r_radius + thickness) as f32 * i.sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
            self.vertices.get_mut(2).set_data(x as f32 + (r_radius + thickness) as f32 * (i + step).cos(), y as f32 + (r_radius + thickness) as f32 * (i + step).sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
            self.vertices.get_mut(3).set_data(x as f32 + r_radius as f32 * (i + step).cos(), y as f32 + r_radius as f32 * (i + step).sin(), 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
            self.batch.add_vertices(&self.vertices);
            i += step;
        }
    }

    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32) {
        self.line_rotated(x1, y1, x2, y2, thickness, 0.0);
    }

    pub fn line_rotated(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, rotation: f32) {
        self.line_origin_rotated(x1, y1, x2, y2, thickness, rotation, (x1 + x2) / 2, (y1 + y2) / 2)
    }

    pub fn line_origin_rotated(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32, rotation: f32, rx: i32, ry: i32) {
        let theta = (x2 as f32 - x1 as f32).atan2(y2 as f32 - y1 as f32);
        let theta_sin = theta.sin() * (thickness as f32 / 2.0);
        let theta_cos = theta.cos() * (thickness as f32 / 2.0);
        let rad_rot = rotation.to_radians();

        self.vertices.get_mut(0).set_data(x1 - theta_cos, y1 + theta_sin, 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
        self.vertices.get_mut(1).set_data(x1 + theta_cos, y1 - theta_sin, 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
        self.vertices.get_mut(2).set_data(x2 + theta_cos, y2 - theta_sin, 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
        self.vertices.get_mut(3).set_data(x2 - theta_cos, y2 + theta_sin, 0.0, rad_rot, rx as f32, ry as f32, self.color, self.canvas, self.use_cam);
        self.vertices.set_len(4);
        self.batch.add_vertices(&self.vertices);
    }

    //pub fn image(int x, int y, int width, int height, Texture texture) {
    //image(x, y, width, height, texture, 0f, 0, 0);
    //}

    //pub fn image(int x, int y, int width, int height, TextureRegion texture) {
    //image(x, y, width, height, texture, 0f, 0, 0);
    //}

    //pub fn image(int x, int y, int width, int height, Texture texture, float rotation) {
    //image(x, y, width, height, texture, rotation, x + width / 2, y + height / 2);
    //}

    //pub fn image(int x, int y, int width, int height, TextureRegion texture, float rotation) {
    //image(x, y, width, height, texture, rotation, x + width / 2, y + height / 2);
    //}

    //pub fn image(int x, int y, int width, int height, Texture texture, float rotation, int originX, int originY) {
    //float ax = x;
    //float ay = y;
    //float ax2 = x + width;
    //float ay2 = y + height;

    //float radRotation = (float) (rotation * (Math.PI / 180));

    //int texID = window.getBatchController().addTexture(texture, isStripped);

    //window.getBatchController().addVertices(verts.set(
    //v1.put(ax, ay2, 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v2.put(ax, ay, 0.0f, radRotation, (float) originX, (float) originY, gradient.topLeft.getRed(), gradient.topLeft.getGreen(), gradient.topLeft.getBlue(), gradient.topLeft.getAlpha(), 0.0f, 1.0f, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v3.put(ax2, ay, 0.0f, radRotation, (float) originX, (float) originY, gradient.topRight.getRed(), gradient.topRight.getGreen(), gradient.topRight.getBlue(), gradient.topRight.getAlpha(), 1.0f, 1.0f, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v4.put(ax2, ay2, 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomRight.getRed(), gradient.bottomRight.getGreen(), gradient.bottomRight.getBlue(), gradient.bottomRight.getAlpha(), 1.0f, 0.0f, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    //), useCamera, isStripped);
    //}

    //pub fn image(int x, int y, int width, int height, TextureRegion texture, float rotation, int originX, int originY) {
    //float ax = x;
    //float ay = y;
    //float ax2 = x + width;
    //float ay2 = y + height;

    //float ux0 = texture.getUVCoordinates()[0];
    //float ux1 = texture.getUVCoordinates()[1];
    //float uy1 = texture.getUVCoordinates()[2];
    //float uy0 = texture.getUVCoordinates()[3];

    //float radRotation = (float) (rotation * (Math.PI / 180));

    //int texID = window.getBatchController().addTexture(texture.getParentTexture(), isStripped);

    //window.getBatchController().addVertices(verts.set(
    //v1.put(ax, ay2, 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), ux0, uy0, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v2.put(ax, ay, 0.0f, radRotation, (float) originX, (float) originY, gradient.topLeft.getRed(), gradient.topLeft.getGreen(), gradient.topLeft.getBlue(), gradient.topLeft.getAlpha(), ux0, uy1, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v3.put(ax2, ay, 0.0f, radRotation, (float) originX, (float) originY, gradient.topRight.getRed(), gradient.topRight.getGreen(), gradient.topRight.getBlue(), gradient.topRight.getAlpha(), ux1, uy1, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v4.put(ax2, ay2, 0.0f, radRotation, (float) originX, (float) originY, gradient.bottomRight.getRed(), gradient.bottomRight.getGreen(), gradient.bottomRight.getBlue(), gradient.bottomRight.getAlpha(), ux1, uy0, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    //), useCamera, isStripped);
    //}

    //pub fn imageFromTo(int x1, int y1, int x2, int y2, int thickness, Texture texture) {
    //float theta = (float) Math.atan2(x2 - x1, y2 - y1);
    //float thetaSin = (float) (Math.sin(theta) * thickness);
    //float thetaCos = (float) (Math.cos(theta) * thickness);

    //int texID = window.getBatchController().addTexture(texture, isStripped);

    //window.getBatchController().addVertices(verts.set(
    //v1.put(x1 - thetaCos, y1 + thetaSin, 0.0f, 0.0f, 0.0f, 0.0f, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), 0.0f, 0.0f, texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v2.put(x1 + thetaCos, y1 - thetaSin, 0.0f, 0.0f, 0.0f, 0.0f, gradient.topLeft.getRed(), gradient.topLeft.getGreen(), gradient.topLeft.getBlue(), gradient.topLeft.getAlpha(), 0.0f, 1.0f, texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v3.put(x2 + thetaCos, y2 - thetaSin, 0.0f, 0.0f, 0.0f, 0.0f, gradient.topRight.getRed(), gradient.topRight.getGreen(), gradient.topRight.getBlue(), gradient.topRight.getAlpha(), 1.0f, 1.0f, texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v4.put(x2 - thetaCos, y2 + thetaSin, 0.0f, 0.0f, 0.0f, 0.0f, gradient.bottomRight.getRed(), gradient.bottomRight.getGreen(), gradient.bottomRight.getBlue(), gradient.bottomRight.getAlpha(), 1.0f, 0.0f, texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    //), useCamera, isStripped);
    //}

    //pub fn imageFromTo(int x1, int y1, int x2, int y2, int thickness, TextureRegion texture) {
    //float theta = (float) Math.atan2(x2 - x1, y2 - y1);
    //float thetaSin = (float) (Math.sin(theta) * thickness);
    //float thetaCos = (float) (Math.cos(theta) * thickness);

    //float ux0 = texture.getUVCoordinates()[0];
    //float ux1 = texture.getUVCoordinates()[1];
    //float uy1 = texture.getUVCoordinates()[2];
    //float uy0 = texture.getUVCoordinates()[3];

    //int texID = window.getBatchController().addTexture(texture.getParentTexture(), isStripped);

    //window.getBatchController().addVertices(verts.set(
    //v1.put(x1 - thetaCos, y1 + thetaSin, 0.0f, 0.0f, 0.0f, 0.0f, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), ux0, uy0, texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v2.put(x1 + thetaCos, y1 - thetaSin, 0.0f, 0.0f, 0.0f, 0.0f, gradient.topLeft.getRed(), gradient.topLeft.getGreen(), gradient.topLeft.getBlue(), gradient.topLeft.getAlpha(), ux0, uy1, texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v3.put(x2 + thetaCos, y2 - thetaSin, 0.0f, 0.0f, 0.0f, 0.0f, gradient.topRight.getRed(), gradient.topRight.getGreen(), gradient.topRight.getBlue(), gradient.topRight.getAlpha(), ux1, uy1, texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v4.put(x2 - thetaCos, y2 + thetaSin, 0.0f, 0.0f, 0.0f, 0.0f, gradient.bottomRight.getRed(), gradient.bottomRight.getGreen(), gradient.bottomRight.getBlue(), gradient.bottomRight.getAlpha(), ux1, uy0, texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    //), useCamera, isStripped);
    //}

    //pub fn text(boolean textChroma, int x, int y, int height, String text) {
    //text(textChroma, x, y, height, text, font, 0.0f, 0, 0);
    //}

    //pub fn text(boolean textChroma, int x, int y, int height, String text, BitmapFont font) {
    //text(textChroma, x, y, height, text, font, 0.0f, 0, 0);
    //}

    //pub fn text(boolean textChroma, int x, int y, int height, String text, BitmapFont font, float rotation) {
    //int width = font.getWidth(text, height);
    //text(textChroma, x, y, height, text, font, rotation, x + width / 4, y + height / 4);
    //}

    //pub fn text(boolean textChroma, int x, int y, int height, String text, BitmapFont font, float rotation, int originX, int originY) {
    //int charX = 0;
    //float radRotation = (float) Math.toRadians(rotation);

    //for (int i = 0; i < text.length(); i++) {
    //char c = text.charAt(i);

    //if (!font.contains(c)) continue;

    //Glyph glyph = font.getGlyph(c);

    //int yOff = glyph.getYOffset(height) - (font.getMaxHeight(height) - glyph.getHeight(height));

    //float ax = x + charX + glyph.getXOffset(height);
    //float ay = y - yOff;
    //float ax2 = x + charX + glyph.getXOffset(height) + glyph.getWidth(height);
    //float ay2 = y + glyph.getHeight(height) - yOff;

    //if (textChroma) {
    //gradient.resetTo(0, 0, 0, 255);
    //gradient.bottomLeft.fromHue(Utils.overlap(getHue((int) ax, (int) ay), 0, 359));
    //gradient.bottomRight.fromHue(Utils.overlap(getHue((int) ax2, (int) ay), 0, 359));
    //gradient.topLeft.fromHue(Utils.overlap(getHue((int) ax, (int) ay2), 0, 359));
    //gradient.topRight.fromHue(Utils.overlap(getHue((int) ax2, (int) ay2), 0, 359));
    //}

    //charX += glyph.getXAdvance(height);

    //Vector2f[] uvs = glyph.getCoordinates();
    //float ux0 = uvs[0].x;
    //float ux1 = uvs[1].x;
    //float uy1 = uvs[0].y;
    //float uy0 = uvs[1].y;

    //int texID = window.getBatchController().addTexture(font.getBitmap(), isStripped);

    //window.getBatchController().addVertices(verts.set(
    //v1.put(ax, ay2, 0.0f, radRotation, originX, originY, gradient.bottomLeft.getRed(), gradient.bottomLeft.getGreen(), gradient.bottomLeft.getBlue(), gradient.bottomLeft.getAlpha(), ux0, uy0, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v2.put(ax, ay, 0.0f, radRotation, originX, originY, gradient.topLeft.getRed(), gradient.topLeft.getGreen(), gradient.topLeft.getBlue(), gradient.topLeft.getAlpha(), ux0, uy1, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v3.put(ax2, ay, 0.0f, radRotation, originX, originY, gradient.topRight.getRed(), gradient.topRight.getGreen(), gradient.topRight.getBlue(), gradient.topRight.getAlpha(), ux1, uy1, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v4.put(ax2, ay2, 0.0f, radRotation, originX, originY, gradient.bottomRight.getRed(), gradient.bottomRight.getGreen(), gradient.bottomRight.getBlue(), gradient.bottomRight.getAlpha(), ux1, uy0, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    //), useCamera, isStripped);
    //}
    //}

    fn get_hue(&self, x: i32, y: i32) -> i32 {
        (((x * 180 / self.size[0]) + ((y * 180 / self.size[1]) * self.chroma_tilt) + (self.frame)) * 5 * self.chroma_compress) as i32
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

    //pub fn animatedText(TextAnimator animator, BitmapFont font, float rotation, int originX, int originY) {
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
    //v1.put(ax, ay2, 0.0f, radRotation, originX, originY, state.color.r, state.color.g, state.color.b, state.color.a, ux0, uy0, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v2.put(ax, ay, 0.0f, radRotation, originX, originY, state.color.r, state.color.g, state.color.b, state.color.a, ux0, uy1, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v3.put(ax2, ay, 0.0f, radRotation, originX, originY, state.color.r, state.color.g, state.color.b, state.color.a, ux1, uy1, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius),
    //v4.put(ax2, ay2, 0.0f, radRotation, originX, originY, state.color.r, state.color.g, state.color.b, state.color.a, ux1, uy0, (float) texID, canvas.x, canvas.y, canvas.z, canvas.w, edgeStyle, edgeRadius)
    //), useCamera, isStripped);
    //}
    //}

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