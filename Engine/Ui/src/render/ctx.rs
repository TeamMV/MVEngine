use mvcore::color::{Color, ColorFormat, RgbColor, RgbColorFormat};
use mvcore::render::backend::swapchain::SwapchainError;
use mve2d::gpu::Transform;
use mve2d::renderer2d::InputTriangle;
use crate::render::{UiRenderer, ZCoords};

pub type DrawShape = Vec<InputTriangle>;

pub fn transform() -> TransformCtx {
    TransformCtx::new()
}

pub fn triangle() -> TriangleCtx {
    TriangleCtx::new()
}

pub fn rectangle() -> RectangleCtx {
    RectangleCtx::new()
}

pub fn arc() -> ArcCtx {
    ArcCtx::new()
}

pub struct DrawContext2D {
    renderer: UiRenderer,
    canvas_transform: Transform
}

impl DrawContext2D {
    pub fn new(renderer: UiRenderer) -> Self {
        Self {
            renderer,
            canvas_transform: Transform::new(),
        }
    }

    pub fn shape(&mut self, shape: DrawShape) {
        for mut triangle in shape {
            triangle.z = self.renderer.gen_z();
            triangle.canvas_transform = self.canvas_transform.clone();

            self.renderer.add_triangle(triangle);
        }
    }

    pub fn draw(&mut self) -> Result<(), SwapchainError> {
        self.renderer.draw()
    }
}

pub struct TransformCtx {
    transform: Transform,
    origin_set: bool,
}

impl TransformCtx {
    fn new() -> Self {
        Self {
            transform: Transform::new(),
            origin_set: false,
        }
    }

    pub fn translate(mut self, x: i32, y: i32) -> Self {
        self.transform.translation.x = x as f32;
        self.transform.translation.y = y as f32;
        self
    }

    pub fn rotate(mut self, rotation: f32) -> Self {
        self.transform.rotation = rotation.to_radians();
        self
    }

    pub fn origin(mut self, x: i32, y: i32) -> Self {
        self.transform.origin.x = x as f32;
        self.transform.origin.y = y as f32;
        self.origin_set = true;
        self
    }

    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.transform.scale.x = x;
        self.transform.scale.y = y;
        self
    }
}

pub struct TriangleCtx {
    points: Vec<(i32, i32, Option<RgbColor>)>,
    global_color: RgbColor,
    transform: Transform,
    custom_origin: bool,
}

impl TriangleCtx {
    fn new() -> Self {
        Self {
            points: vec![],
            global_color: RgbColor::white(),
            transform: Transform::new(),
            custom_origin: false,
        }
    }

    pub fn point(mut self, xy: (i32, i32), color: Option<RgbColor>) -> Self {
        self.points.push((xy.0, xy.1, color));
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

    pub fn create(mut self) -> DrawShape {
        let mut iter = self.points.into_iter();
        let p1 = iter.next().expect("Expected 3 points on a triangle");
        let p2 = iter.next().expect("Expected 3 points on a triangle");
        let p3 = iter.next().expect("Expected 3 points on a triangle");

        let c1 = p1.2.unwrap_or(self.global_color.clone());
        let c2 = p2.2.unwrap_or(self.global_color.clone());
        let c3 = p3.2.unwrap_or(self.global_color.clone());

        if !self.custom_origin {
            self.transform.origin.x = (p1.0 + p2.0 + p3.0) as f32 / 3.0;
            self.transform.origin.y = (p1.1 + p2.1 + p3.1) as f32 / 3.0;
        }

        let tri = InputTriangle {
            points: [(p1.0, p1.1), (p2.0, p2.1), (p3.0, p3.1)],
            z: 0.0,
            transform: self.transform,
            canvas_transform: Transform::new(),
            tex_id: None,
            tex_coords: None,
            blending: 0.0,
            colors: [c1.as_vec4(), c2.as_vec4(), c3.as_vec4()],
            is_font: false,
        };

        vec![tri]
    }
}

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
}

impl RectangleCtx {
    fn new() -> Self {
        Self {
            points: vec![],
            point_colors: [0; 4].map(|_| None).to_vec(),
            global_color: RgbColor::white(),
            transform: Transform::new(),
            custom_origin: false,
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

        let tri1 = InputTriangle {
            points: [(p1.0, p1.1), (p2.0, p2.1), (p3.0, p3.1)],
            z: 0.0,
            transform: self.transform.clone(),
            canvas_transform: Transform::new(),
            tex_id: None,
            tex_coords: None,
            blending: 0.0,
            colors: [c1.as_vec4(), c2.as_vec4(), c3.as_vec4()],
            is_font: false,
        };

        let tri2 = InputTriangle {
            points: [(p1.0, p1.1), (p3.0, p3.1), (p4.0, p4.1)],
            z: 0.0,
            transform: self.transform,
            canvas_transform: Transform::new(),
            tex_id: None,
            tex_coords: None,
            blending: 0.0,
            colors: [c2.as_vec4(), c3.as_vec4(), c4.as_vec4()],
            is_font: false,
        };

        vec![tri1, tri2]
    }
}

pub enum ArcTriPoint {
    Last,
    Current,
    Center
}

pub struct ArcCtx {
    center: (i32, i32),
    radius: i32,
    triangle_count: u32,
    angle: f32,
    global_color: RgbColor,
    transform: Transform,
    custom_origin: bool,
}

impl ArcCtx {
    fn new() -> Self {
        Self {
            center: (0, 0),
            radius: 0,
            triangle_count: 50,
            angle: 90.0,
            global_color: RgbColor::white(),
            transform: Transform::new(),
            custom_origin: false,
        }
    }

    pub fn center(mut self, x: i32, y: i32) -> Self {
        self.center = (x, y);
        self
    }

    pub fn radius(mut self, radius: i32) -> Self {
        self.radius = radius;
        self
    }

    pub fn angle(mut self, angle: f32) -> Self {
        self.angle = angle.to_radians();
        self
    }

    pub fn triangle_count(mut self, count: u32) -> Self {
        self.triangle_count = count;
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

    pub fn create(mut self) -> DrawShape {
        if !self.custom_origin {
            self.transform.origin.x = self.center.0 as f32;
            self.transform.origin.y = self.center.1 as f32;
        }

        let mut tris = Vec::with_capacity(self.triangle_count as usize);

        let rad = self.radius as f32;
        let step_size = self.angle / self.triangle_count as f32;
        let mut last_x = self.center.0 + self.radius;
        let mut last_y = self.center.1;
        for i in 1..self.triangle_count + 1 {
            let current = i as f32 * step_size;
            let x = (self.center.0 as f32 + current.cos() * rad) as i32;
            let y = (self.center.1 as f32 + current.sin() * rad) as i32;

            let tri = InputTriangle {
                points: [(last_x, last_y), self.center, (x, y)],
                z: 0.0,
                transform: self.transform.clone(),
                canvas_transform: Transform::new(),
                tex_id: None,
                tex_coords: None,
                blending: 0.0,
                colors: [self.global_color.as_vec4(), self.global_color.as_vec4(), self.global_color.as_vec4()],
                is_font: false,
            };
            tris.push(tri);

            last_x = x;
            last_y = y;
        }
        tris
    }
}