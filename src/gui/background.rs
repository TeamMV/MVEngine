use crate::gui::elements::GuiElement;
use crate::gui::styles::{Dimension, GuiValue};
use crate::render::color::RgbColor;
use crate::render::draw2d::DrawContext2D;
use crate::resolve;
use std::any::{Any, TypeId};
use std::sync::Arc;

#[derive(Clone)]
pub struct BackgroundInfo {
    pub main_color: GuiValue<RgbColor>,
    pub border_color: GuiValue<RgbColor>,
    pub border_width: GuiValue<i32>,
}

impl Default for BackgroundInfo {
    fn default() -> Self {
        Self {
            main_color: GuiValue::Just(RgbColor::white()),
            border_color: GuiValue::Just(RgbColor::black()),
            border_width: GuiValue::Just(2),
        }
    }
}

pub trait Background {
    fn draw(&self, ctx: &mut DrawContext2D, elem: Arc<dyn GuiElement>);
}

pub trait IsTypeBackground {
    fn is_type<B>(&self) -> bool
    where
        B: Background + 'static;
}

impl<T: Background + ?Sized + 'static> IsTypeBackground for Arc<T> {
    fn is_type<B>(&self) -> bool
    where
        B: Background + 'static,
    {
        TypeId::of::<B>() == self.type_id()
    }
}

#[derive(Clone)]
pub struct RectangleBackground {}

impl Background for RectangleBackground {
    fn draw(&self, ctx: &mut DrawContext2D, elem: Arc<dyn GuiElement>) {
        let main = resolve!(elem, background.main_color);
        ctx.color(main);

        let rot = resolve!(elem, rotation);
        let rot_origin = resolve!(elem, rotation_origin).resolve(elem.as_ref());

        ctx.rectangle_origin_rotated(
            elem.border_x(),
            elem.border_y(),
            elem.width(),
            elem.height(),
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        let border = resolve!(elem, background.border_color);
        ctx.color(border);

        let width = resolve!(elem, background.border_width);

        ctx.void_rectangle_origin_rotated(
            elem.border_x(),
            elem.border_y(),
            elem.width(),
            elem.height(),
            width,
            rot,
            rot_origin.x,
            rot_origin.y,
        );
    }
}

#[derive(Clone)]
pub struct RoundedBackground {
    radius: Dimension<i32>,
}

impl RoundedBackground {
    pub fn new(radii: Dimension<i32>) -> Self {
        Self { radius: radii }
    }
}

impl Background for RoundedBackground {
    fn draw(&self, ctx: &mut DrawContext2D, elem: Arc<dyn GuiElement>) {
        let rot = resolve!(elem, rotation);
        let rot_origin = resolve!(elem, rotation_origin).resolve(elem.as_ref());
        let prec = (self.radius.width + self.radius.height) as f32 / 2.0;

        let main = resolve!(elem, background.main_color);
        ctx.color(main);

        //main
        ctx.rectangle_origin_rotated(
            elem.border_x() + self.radius.width,
            elem.border_y(),
            elem.width() - 2 * self.radius.width,
            elem.height(),
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.rectangle_origin_rotated(
            elem.border_x(),
            elem.border_y() + self.radius.height,
            self.radius.width,
            elem.height() - 2 * self.radius.height,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.rectangle_origin_rotated(
            elem.border_x() + elem.width() - self.radius.width,
            elem.border_y() + self.radius.height,
            self.radius.width,
            elem.height() - 2 * self.radius.height,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.ellipse_arc_origin_rotated(
            elem.border_x() + self.radius.width,
            elem.border_y() + self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            180,
            prec,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.ellipse_arc_origin_rotated(
            elem.border_x() + self.radius.width,
            elem.border_y() + elem.height() - 2 * self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            270,
            prec,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.ellipse_arc_origin_rotated(
            elem.border_x() + elem.width() - 2 * self.radius.width,
            elem.border_y() + elem.height() - 2 * self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            0,
            prec,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.ellipse_arc_origin_rotated(
            elem.border_x() + elem.width() - 2 * self.radius.width,
            elem.border_y() + self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            90,
            prec,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        let border = resolve!(elem, background.border_color);
        ctx.color(border);

        let border_width = resolve!(elem, background.border_width);

        //border
        ctx.rectangle_origin_rotated(
            elem.border_x() + self.radius.width,
            elem.border_y(),
            elem.width() - 2 * self.radius.width,
            border_width,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.rectangle_origin_rotated(
            elem.border_x() + self.radius.width,
            elem.border_y() + elem.height() - border_width,
            elem.width() - 2 * self.radius.width,
            border_width,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.rectangle_origin_rotated(
            elem.border_x(),
            elem.border_y() + self.radius.height,
            border_width,
            elem.height() - 2 * self.radius.height,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.rectangle_origin_rotated(
            elem.border_x() + elem.width() - border_width,
            elem.border_y() + self.radius.height,
            border_width,
            elem.height() - 2 * self.radius.height,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.void_ellipse_arc_origin_rotated(
            elem.border_x() + self.radius.width,
            elem.border_y() + self.radius.height,
            self.radius.width,
            self.radius.height,
            border_width,
            90,
            180,
            prec,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.void_ellipse_arc_origin_rotated(
            elem.border_x() + self.radius.width,
            elem.border_y() + elem.height() - 2 * self.radius.height,
            self.radius.width,
            self.radius.height,
            border_width,
            90,
            270,
            prec,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.void_ellipse_arc_origin_rotated(
            elem.border_x() + elem.width() - 2 * self.radius.width,
            elem.border_y() + elem.height() - 2 * self.radius.height,
            self.radius.width,
            self.radius.height,
            border_width,
            90,
            0,
            prec,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.void_ellipse_arc_origin_rotated(
            elem.border_x() + elem.width() - 2 * self.radius.width,
            elem.border_y() + self.radius.height,
            self.radius.width,
            self.radius.height,
            border_width,
            90,
            90,
            prec,
            rot,
            rot_origin.x,
            rot_origin.y,
        );
    }
}
