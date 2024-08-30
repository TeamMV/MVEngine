use mvcore::color::RgbColor;
use mvcore::math::vec::{Vec2, Vec3, Vec4};
use mve2d::renderer2d::Renderer2D;
use crate::styles::Dimension;
use crate::utils::OptionGetMapOr;

pub enum Shape {
    Rect(Rect),
    VoidRect(VoidRect),
}

impl Shape {
    pub fn draw(&self, renderer: &mut Renderer2D) {
        match self {
            Shape::Rect(s) => s.draw(renderer),
            Shape::VoidRect(s) => {}
        }
    }
}

pub struct UiTransform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec2,
}

pub struct ShapeBase {
    pub transform: UiTransform,
    pub color: RgbColor,
    pub texture: Option<ShapeTexture>
}

pub struct ShapeTexture {
    pub id: u16,
    pub coords: Vec4,
    pub blending: f32,
}

pub struct Rect {
    base: ShapeBase,
    dimension: Dimension<i32>
}

impl Rect {
    pub fn draw(&self, renderer: &mut Renderer2D) {
        renderer.add_shape(mve2d::renderer2d::Shape::Rectangle {
            position: self.base.transform.position,
            rotation: self.base.transform.rotation,
            scale: self.base.transform.scale.mul_xy(self.dimension.width as f32, self.dimension.height as f32),
            tex_id: Some(self.base.texture.get_map_or(|t| t.id.clone(), 0).clone()),
            tex_coord: self.base.texture.get_map_or(|t| t.coords, Vec4::default()),
            color: self.base.color.as_vec4(),
            blending: self.base.texture.get_map_or(|t| t.blending, 0.0),
        })
    }
}

pub struct VoidRect {
    base: ShapeBase,
    dimension: Dimension<i32>,
    thickness: i32,
}