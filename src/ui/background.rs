use crate::render::color::RgbColor;
use crate::render::draw2d::DrawContext2D;
use crate::render::window::Window;
use crate::render::ApplicationLoopCallbacks;
use crate::resolve;
use crate::ui::ease::Easing;
use crate::ui::elements::UiElement;
use crate::ui::styles::{Dimension, Origin, Point, UiValue};
use crate::ui::timing::{DurationTask, EffectState, TIMING_MANAGER};
use mvutils::utils::{Percentage, Recover};
use std::any::{Any, TypeId};
use std::ops::Deref;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct BackgroundInfo {
    pub main_color: UiValue<RgbColor>,
    pub border_color: UiValue<RgbColor>,
    pub border_width: UiValue<i32>,
}

impl Default for BackgroundInfo {
    fn default() -> Self {
        Self {
            main_color: UiValue::Just(RgbColor::white()),
            border_color: UiValue::Just(RgbColor::black()),
            border_width: UiValue::Just(2),
        }
    }
}

pub trait Background {
    fn draw(&self, ctx: &mut DrawContext2D, elem: Arc<RwLock<dyn UiElement>>);
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
        TypeId::of::<B>() == TypeId::of::<T>()
    }
}

#[derive(Clone)]
pub struct RectangleBackground {}

impl RectangleBackground {
    pub fn new() -> Self {
        Self {}
    }
}

impl Background for RectangleBackground {
    fn draw(&self, ctx: &mut DrawContext2D, elem: Arc<RwLock<dyn UiElement>>) {
        let elem = elem.read().recover();
        let main = resolve!(elem, background.main_color);
        ctx.color(main);

        let rot = resolve!(elem, rotation);
        let rot_origin = resolve!(elem, rotation_origin).resolve(&*elem);

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
    fn draw(&self, ctx: &mut DrawContext2D, elem: Arc<RwLock<dyn UiElement>>) {
        let elem = elem.read().recover();
        let rot = resolve!(elem, rotation);
        let rot_origin = resolve!(elem, rotation_origin).resolve(&*elem);
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
            elem.border_y() + elem.height() - self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            90,
            prec,
            rot,
            rot_origin.x,
            rot_origin.y,
        );

        ctx.ellipse_arc_origin_rotated(
            elem.border_x() + elem.width() - self.radius.width,
            elem.border_y() + elem.height() - self.radius.height,
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
            elem.border_x() + elem.width() - self.radius.width,
            elem.border_y() + self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            270,
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
            elem.border_x() + self.radius.width + border_width / 2,
            elem.border_y() + self.radius.height + border_width / 2,
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
            elem.border_x() + self.radius.width + border_width / 2,
            elem.border_y() + elem.height() - self.radius.height - border_width / 2,
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

        ctx.void_ellipse_arc_origin_rotated(
            elem.border_x() + elem.width() - self.radius.width - border_width / 2,
            elem.border_y() + elem.height() - self.radius.height - border_width / 2,
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
            elem.border_x() + elem.width() - self.radius.width - border_width / 2,
            elem.border_y() + self.radius.height + border_width / 2,
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
    }
}

#[derive(Clone)]
pub(crate) struct BackgroundEffectInfo {
    pub(crate) fill_mode: FillMode,
    pub(crate) duration: u32,
    pub(crate) color: Option<RgbColor>,
    pub(crate) pos: Option<Point<i32>>,
    pub(crate) easing: Easing,
    pub(crate) elem: Option<Arc<RwLock<dyn UiElement>>>,
}

#[derive(Clone)]
pub enum FillMode {
    Keep,
    Revert,
}

pub struct TriggerOptions {
    pub position: Option<Origin>,
}

pub trait BackgroundEffect {
    fn info(&self) -> &BackgroundEffectInfo;
    fn info_mut(&mut self) -> &mut BackgroundEffectInfo;
    fn trigger<T: ApplicationLoopCallbacks + Sized>(
        &mut self,
        options: Option<TriggerOptions>,
        elem: Arc<RwLock<dyn UiElement>>,
        win: Arc<Window<T>>,
    );
    fn cancel(&self);
    fn draw(
        info: &BackgroundEffectInfo,
        ctx: &mut DrawContext2D,
        percent: f32,
        elem: Arc<RwLock<dyn UiElement>>,
    );
}

pub struct RippleCircleBackgroundEffect {
    info: BackgroundEffectInfo,
    color: RgbColor,
    pos: Point<i32>,
    task_id: u64,
}

impl RippleCircleBackgroundEffect {
    pub fn new(color: RgbColor, duration: u32, fill_mode: FillMode, easing: Easing) -> Self {
        Self {
            info: BackgroundEffectInfo {
                fill_mode,
                duration,
                color: None,
                pos: None,
                easing,
                elem: None,
            },
            color,
            pos: Point::new(0, 0),
            task_id: 0,
        }
    }
}

impl BackgroundEffect for RippleCircleBackgroundEffect {
    fn info(&self) -> &BackgroundEffectInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut BackgroundEffectInfo {
        &mut self.info
    }

    fn trigger<T: ApplicationLoopCallbacks + Sized>(
        &mut self,
        options: Option<TriggerOptions>,
        elem: Arc<RwLock<dyn UiElement>>,
        win: Arc<Window<T>>,
    ) {
        let e = elem.read().recover();
        self.pos = Point::new(e.border_x() + e.width() / 2, e.border_y() + e.height() / 2);

        if options.is_some() {
            let options = options.unwrap();
            if options.position.is_some() {
                self.pos = match options.position.unwrap() {
                    Origin::TopLeft => Point::new(e.border_x(), e.border_y() + e.height()),
                    Origin::BottomLeft => Point::new(e.border_x(), e.border_y()),
                    Origin::TopRight => {
                        Point::new(e.border_x() + e.width(), e.border_y() + e.height())
                    }
                    Origin::BottomRight => Point::new(e.border_x() + e.width(), e.border_y()),
                    Origin::Center => {
                        Point::new(e.border_x() + e.width() / 2, e.border_y() + e.height() / 2)
                    }
                    Origin::Custom(x, y) => Point::new(x, y),
                }
            }
        }

        self.info.pos = Some(self.pos.clone());
        self.info.color = Some(self.color.clone());

        self.info.elem = Some(elem.clone());

        unsafe {
            let id = TIMING_MANAGER.request(DurationTask::new(
                self.info.duration,
                move |state, time| match state.background {
                    None => {}
                    Some(ref info) => {
                        let percent = (time as f32).percentage(info.duration as f32);
                        win.draw_2d_pass(|ctx| {
                            RippleCircleBackgroundEffect::draw(
                                info,
                                ctx,
                                percent,
                                info.elem.as_ref().unwrap().clone(),
                            );
                        })
                    }
                },
                EffectState::background(self.info.clone()),
            ));

            self.task_id = id;
        }
    }

    fn cancel(&self) {
        unsafe {
            TIMING_MANAGER.cancel(self.task_id);
        }
    }

    fn draw(
        info: &BackgroundEffectInfo,
        ctx: &mut DrawContext2D,
        percent: f32,
        elem: Arc<RwLock<dyn UiElement>>,
    ) {
        println!("print");
        let e = elem.read().recover();
        println!("print2");
        let diameter = ((e.width() * e.width() + e.height() * e.height()) as f32).sqrt();

        let rot = resolve!(e, rotation);
        let rot_origin = resolve!(e, rotation_origin).resolve(&*e);

        let pos = info.pos.as_ref().unwrap();

        let mut c = info.color.unwrap();
        c.set_a(percent.value(1f32));
        ctx.color(c);
        ctx.circle_origin_rotated(
            pos.x,
            pos.y,
            percent.value(diameter) as i32,
            percent.value(diameter),
            rot,
            rot_origin.x,
            rot_origin.y,
        );
    }
}
