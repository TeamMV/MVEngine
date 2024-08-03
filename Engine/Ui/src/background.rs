use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::cmp::Ordering;
use std::f32::consts::PI;
use std::sync::Arc;

use crate::err::panic;
use mvutils::utils::{Percentage, Recover};

use crate::render::color::RgbColor;
use crate::render::draw2d::DrawContext2D;
use crate::render::window::Window;
use crate::render::ApplicationLoopCallbacks;
use crate::ui::ease::Easing;
use crate::ui::elements::{UiElement, UiElementState};
use crate::ui::styles::Resolve;
use crate::ui::styles::{Dimension, Interpolator, Origin, Point, UiStyle, UiValue};
use crate::ui::timing::{AnimationState, DurationTask, TIMING_MANAGER};
use crate::{blanked_partial_ord, interpolator_map_fn, resolve, ui};

#[derive(Clone)]
pub struct BackgroundInfo {
    pub main_color: Resolve<RgbColor>,
    pub border_color: Resolve<RgbColor>,
    pub border_width: Resolve<i32>,
}

impl Default for BackgroundInfo {
    fn default() -> Self {
        Self {
            main_color: UiValue::Just(RgbColor::white()).to_field().into(),
            border_color: UiValue::Just(RgbColor::black()).to_field().into(),
            border_width: UiValue::Just(2).to_field().into(),
        }
    }
}

//impl Interpolator<BackgroundInfo> for BackgroundInfo {
//    fn interpolate<E, F>(&mut self, end: &BackgroundInfo, percent: f32, elem: Arc<RwLock<E>>, f: F)
//    where
//        E: UiElement,
//        F: Fn(&UiStyle) -> Resolve<BackgroundInfo>,
//    {
//        self.border_width.interpolate(
//            &end.border_width,
//            percent,
//            elem.clone(),
//            interpolator_map_fn!(self, f, elem, border_width),
//        );
//        self.border_color.interpolate(
//            &end.border_color,
//            percent,
//            elem.clone(),
//            interpolator_map_fn!(self, f, elem, border_color),
//        );
//        self.main_color.interpolate(
//            &end.main_color,
//            percent,
//            elem.clone(),
//            interpolator_map_fn!(self, f, elem, main_color),
//        );
//    }
//}

blanked_partial_ord!(BackgroundInfo);

pub trait Background {
    fn draw(&self, ctx: &mut DrawContext2D, state: &UiElementState, style: &UiStyle);
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
    fn draw(&self, ctx: &mut DrawContext2D, state: &UiElementState, style: &UiStyle) {
        let main_color = ui::utils::resolve_color(&style.background.main_color, RgbColor::black(), state, |s| &s.background.main_color);
        ctx.color(main_color);

        let rot = style.rotation.resolve(state.ctx.dpi, state.parent.clone(), |s| &s.rotation).unwrap_or_default();
        let rot_origin = style.rotation_origin.resolve(state.ctx.dpi, state.parent.clone(), |s| &s.rotation_origin).unwrap_or(Origin::Center);

        let rot_origin = (
            rot_origin.get_actual_x(state.x, state.width),
            rot_origin.get_actual_y(state.y, state.height),
        );

        ctx.rectangle_origin_rotated(
            state.x,
            state.y,
            state.width,
            state.height,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        let border_color = ui::utils::resolve_color(&style.background.border_color, RgbColor::black(), state, |s| &s.background.border_color);
        ctx.color(border_color);

        let width = style.background.border_width.resolve(state.ctx.dpi, state.parent.clone(), |s| &s.background.border_width).unwrap_or(2);

        ctx.void_rectangle_origin_rotated(
            state.x,
            state.y,
            state.width,
            state.height,
            width,
            rot,
            rot_origin.0,
            rot_origin.1,
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
    fn draw(&self, ctx: &mut DrawContext2D, state: &UiElementState, style: &UiStyle) {
        let rot = style.rotation.resolve(state.ctx.dpi, state.parent.clone(), |s| &s.rotation).unwrap_or_default();
        let rot_origin = style.rotation_origin.resolve(state.ctx.dpi, state.parent.clone(), |s| &s.rotation_origin).unwrap_or(Origin::Center);

        let rot_origin = (
            rot_origin.get_actual_x(state.x, state.width),
            rot_origin.get_actual_y(state.y, state.height),
        );

        let prec = (self.radius.width + self.radius.height) as f32 / 2.0;

        let main_color = ui::utils::resolve_color(&style.background.main_color, RgbColor::black(), state, |s| &s.background.main_color);
        ctx.color(main_color);

        //main
        ctx.rectangle_origin_rotated(
            state.x + self.radius.width,
            state.y,
            state.width - 2 * self.radius.width,
            state.height,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.rectangle_origin_rotated(
            state.x,
            state.y + self.radius.height,
            self.radius.width,
            state.height - 2 * self.radius.height,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.rectangle_origin_rotated(
            state.x + state.width - self.radius.width,
            state.y + self.radius.height,
            self.radius.width,
            state.height - 2 * self.radius.height,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.ellipse_arc_origin_rotated(
            state.x + self.radius.width,
            state.y + self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            180,
            prec,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.ellipse_arc_origin_rotated(
            state.x + self.radius.width,
            state.y + state.height - self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            90,
            prec,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.ellipse_arc_origin_rotated(
            state.x + state.width - self.radius.width,
            state.y + state.height - self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            0,
            prec,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.ellipse_arc_origin_rotated(
            state.x + state.width - self.radius.width,
            state.y + self.radius.height,
            self.radius.width,
            self.radius.height,
            90,
            270,
            prec,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        let border_color = ui::utils::resolve_color(&style.background.border_color, RgbColor::black(), state, |s| &s.background.border_color);
        ctx.color(border_color);

        let border_width = style.background.border_width.resolve(state.ctx.dpi, state.parent.clone(), |s| &s.background.border_width).unwrap_or(2);

        //border
        ctx.rectangle_origin_rotated(
            state.x + self.radius.width,
            state.y,
            state.width - 2 * self.radius.width,
            border_width,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.rectangle_origin_rotated(
            state.x + self.radius.width,
            state.y + state.height - border_width,
            state.width - 2 * self.radius.width,
            border_width,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.rectangle_origin_rotated(
            state.x,
            state.y + self.radius.height,
            border_width,
            state.height - 2 * self.radius.height,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.rectangle_origin_rotated(
            state.x + state.width - border_width,
            state.y + self.radius.height,
            border_width,
            state.height - 2 * self.radius.height,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.void_ellipse_arc_origin_rotated(
            state.x + self.radius.width + border_width / 2,
            state.y + self.radius.height + border_width / 2,
            self.radius.width,
            self.radius.height,
            border_width,
            90,
            180,
            prec,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.void_ellipse_arc_origin_rotated(
            state.x + self.radius.width + border_width / 2,
            state.y + state.height - self.radius.height - border_width / 2,
            self.radius.width,
            self.radius.height,
            border_width,
            90,
            90,
            prec,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.void_ellipse_arc_origin_rotated(
            state.x + state.width - self.radius.width - border_width / 2,
            state.y + state.height - self.radius.height - border_width / 2,
            self.radius.width,
            self.radius.height,
            border_width,
            90,
            0,
            prec,
            rot,
            rot_origin.0,
            rot_origin.1,
        );

        ctx.void_ellipse_arc_origin_rotated(
            state.x + state.width - self.radius.width - border_width / 2,
            state.y + self.radius.height + border_width / 2,
            self.radius.width,
            self.radius.height,
            border_width,
            90,
            270,
            prec,
            rot,
            rot_origin.0,
            rot_origin.1,
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
        let e = elem.read();
        let state = e.state();

        self.pos = Point::new(state.x + state.width / 2, state.y + state.height / 2);

        if options.is_some() {
            let options = options.unwrap();
            if options.position.is_some() {
                self.pos = match options.position.unwrap() {
                    Origin::TopLeft => Point::new(state.x, state.y + state.height),
                    Origin::BottomLeft => Point::new(state.x, state.y),
                    Origin::TopRight => Point::new(state.x + state.width, state.y + state.height),
                    Origin::BottomRight => Point::new(state.x + state.width, state.y),
                    Origin::Center => {
                        Point::new(state.x + state.width / 2, state.y + state.height / 2)
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
                        let percent = info.easing.get(percent);

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
                AnimationState::background(self.info.clone()),
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
        let e = elem.read();
        let state = e.state();
        let diameter = ((state.width * state.width + state.height * state.height) as f32).sqrt();

        let rot = resolve!(e, rotation);
        let rot_origin = resolve!(e, rotation_origin);

        let rot_origin = (
            rot_origin.get_actual_x(state.x, state.width),
            rot_origin.get_actual_y(state.y, state.height),
        );

        let pos = info.pos.as_ref().unwrap();

        let mut c = info.color.unwrap();
        c.set_a(percent.value(1f32));
        ctx.color(c);
        let r = percent.value(diameter);

        let pi2 = PI * 2.0;
        let step = pi2 / r;
        let mut done = step;

        let mut last_x =
            (pos.x as f32 + 0f32.cos() * r).clamp(state.x as f32, (state.x + state.width) as f32);
        let mut last_y =
            (pos.y as f32 + 0f32.sin() * r).clamp(state.y as f32, (state.y + state.height) as f32);
        loop {
            let x = (pos.x as f32 + done.cos() * r)
                .clamp(state.x as f32, (state.x + state.width) as f32);
            let y = (pos.y as f32 + done.sin() * r)
                .clamp(state.y as f32, (state.y + state.height) as f32);

            ctx.begin_shape();

            ctx.vertex(
                pos.x as f32,
                pos.y as f32,
                rot,
                rot_origin.0 as f32,
                rot_origin.1 as f32,
            );

            ctx.vertex(
                last_x,
                last_y,
                rot,
                rot_origin.0 as f32,
                rot_origin.1 as f32,
            );

            ctx.vertex(x, y, rot, rot_origin.0 as f32, rot_origin.1 as f32);

            last_x = x;
            last_y = y;

            done += step;

            if pi2 - done < step {
                let x = (pos.x as f32 + 0f32.cos() * r)
                    .clamp(state.x as f32, (state.x + state.width) as f32);
                let y = (pos.y as f32 + 0f32.sin() * r)
                    .clamp(state.y as f32, (state.y + state.height) as f32);

                ctx.begin_shape();

                ctx.vertex(
                    pos.x as f32,
                    pos.y as f32,
                    rot,
                    rot_origin.0 as f32,
                    rot_origin.1 as f32,
                );

                ctx.vertex(
                    last_x,
                    last_y,
                    rot,
                    rot_origin.0 as f32,
                    rot_origin.1 as f32,
                );

                ctx.vertex(x, y, rot, rot_origin.0 as f32, rot_origin.1 as f32);

                break;
            }

            /*if pi2 - done < step {
                let x = (pos.x as f32 + 0f32.cos() * r).clamp(state.x as f32, (state.x + state.width) as f32);
                let y = (pos.y as f32 + 0f32.sin() * r).clamp(state.y as f32, (state.y + state.height) as f32);

                ctx.vertex(
                    x,
                    y,
                    rot,
                    rot_origin.0 as f32,
                    rot_origin.1 as f32
                );
                break;
            } else {
                ctx.vertex(
                    last_x,
                    last_y,
                    rot,
                    rot_origin.0 as f32,
                    rot_origin.1 as f32
                );

                last_x = x;
                last_y = y;
            }*/
        }

        //ctx.circle_origin_rotated(
        //    pos.x,
        //    pos.y,
        //    percent.value(diameter) as i32,
        //    percent.value(diameter),
        //    rot,
        //    rot_origin.0,
        //    rot_origin.1,
        //);
    }
}
