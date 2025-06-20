use crate::math::vec::Vec4;
use crate::rendering::control::RenderController;
use crate::rendering::texture::Texture;
use crate::rendering::{Quad, RenderContext, Transform};
use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::arc::ArcCtx;
use crate::ui::rendering::rectangle::RectangleCtx;
use crate::ui::rendering::triangle::TriangleCtx;
use crate::ui::rendering::UiRenderer;
use crate::ui::styles::InheritSupplier;
use crate::window::Window;

pub fn transform() -> TransformCtx {
    TransformCtx::new()
}

pub fn texture() -> TextureCtx {
    TextureCtx::new()
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
}

impl DrawContext2D {
    pub fn new(renderer: UiRenderer) -> Self {
        Self { renderer }
    }

    pub fn shape(&mut self, shape: Shape) {
        if shape.is_quad {
            if shape.triangles.len() >= 2 {
                let tri1 = shape.triangles[0].points.clone();
                let tri2 = shape.triangles[1].points.clone();
                
                let z = self.renderer.gen_z();
                let tri1 = tri1.map(|mut v| { v.pos.2 = z; v });
                let tri2 = tri2.map(|mut v| { v.pos.2 = z; v });
                
                let [p11, p12, p13] = tri1;
                let [_, p22, _] = tri2;
                
                let quad = Quad {
                    points: [
                        p11,
                        p12,
                        p13,
                        p22
                    ],
                };
                self.renderer.add_quad(quad);
            }
        } else {
            for mut triangle in shape.triangles {
                if triangle.points[0].pos.2.is_infinite() {
                    let z = self.renderer.gen_z();
                    triangle.points.iter_mut().for_each(|v| v.pos.2 = z);
                }

                self.renderer.add_triangle(triangle);
            }
        }
    }

    pub fn draw(&mut self, window: &Window) {
        self.renderer.draw(window)
    }

    pub fn renderer(&self) -> &UiRenderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut UiRenderer {
        &mut self.renderer
    }

    pub fn resize(&mut self, window: &mut Window) {
        self.renderer.resize(window);
    }
}

impl RenderContext for DrawContext2D {
    fn controller(&mut self) -> &mut RenderController {
        &mut self.renderer.controller
    }
}

impl InheritSupplier for DrawContext2D {
    fn x(&self) -> i32 {
        0
    }

    fn y(&self) -> i32 {
        0
    }

    fn width(&self) -> i32 {
        self.renderer.dimension.0 as i32
    }

    fn height(&self) -> i32 {
        self.renderer.dimension.1 as i32
    }
}

pub struct TransformCtx {
    pub(crate) transform: Transform,
    pub(crate) origin_set: bool,
}

impl TransformCtx {
    fn new() -> Self {
        Self {
            transform: Transform::new(),
            origin_set: false,
        }
    }

    pub fn from_transform(transform: Transform, custom_origin: bool) -> Self {
        Self {
            transform,
            origin_set: custom_origin,
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
    
    pub fn of(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self.origin_set = true;
        self
    }

    pub fn get(&self) -> Transform {
        self.transform.clone()
    }
}

pub struct TextureCtx {
    pub(crate) texture: Option<Texture>,
    pub(crate) blending: f32,
    pub(crate) uv: Vec4,
}

impl TextureCtx {
    pub fn new() -> Self {
        Self {
            texture: None,
            blending: 0.0,
            uv: Vec4::default_uv(),
        }
    }

    pub fn source(mut self, texture: Option<Texture>) -> Self {
        self.texture = texture;
        self
    }

    pub fn blending(mut self, blending: f32) -> Self {
        self.blending = blending;
        self
    }

    pub fn uv(mut self, uv: Vec4) -> Self {
        self.uv = uv;
        self
    }
}
