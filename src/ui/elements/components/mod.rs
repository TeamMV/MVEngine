pub mod text;
pub mod edittext;

use crate::color::RgbColor;
use crate::graphics::comp::Drawable;
use crate::ui::context::{UiContext, UiResources};
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::adaptive::{AdaptiveFill, AdaptiveShape};
use crate::ui::rendering::ctx;
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::res::err::{ResType, UiResErr};
use crate::ui::res::MVR;
use crate::ui::styles::{BackgroundRes, Interpolator, ResolveResult, UiShape, UiStyle};
use crate::{get_adaptive, get_shape, resolve};
use std::marker::PhantomData;
use std::ops::Deref;
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::Percentage;
use crate::input::{Input, MouseAction, RawInputEvent};
use crate::ui::anim::easing;
use crate::ui::ease::{Easing, EasingGen, EasingMode};
use crate::ui::timing::{AnimationState, DurationTask, TIMING_MANAGER};

#[derive(Clone)]
enum State {
    In,
    Out,
}

#[derive(Clone)]
pub struct ElementBody<E: UiElementStub> {
    _phantom: PhantomData<E>,
    fade_time: u32,
    hover_style: Option<UiStyle>,
    hover_state: State,
    easing: Easing,
    initial_style: Option<UiStyle>
}

impl<E: UiElementStub + 'static> ElementBody<E> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::default(),
            fade_time: 0,
            hover_style: None,
            hover_state: State::Out,
            easing: easing(EasingGen::linear(), EasingMode::In),
            initial_style: None,
        }
    }
    
    
    
    pub fn on_input(&mut self, e: &mut E, action: RawInputEvent, input: &Input) {
        match action {
            RawInputEvent::Keyboard(_) => {}
            RawInputEvent::Mouse(ma) => {
                match ma {
                    MouseAction::Wheel(_, _) => {}
                    MouseAction::Move(x, y) => {
                        if e.inside(x, y) {
                            if let State::Out = self.hover_state {
                                self.hover_state = State::In;
                                self.start_animation_in(e);
                            }
                        } else {
                            if let State::In = self.hover_state {
                                self.hover_state = State::Out;
                                self.start_animation_out(e);
                            }
                        }
                    }
                    MouseAction::Press(_) => {}
                    MouseAction::Release(_) => {}
                }
            }
        };
    }

    fn start_animation_in(&mut self, elem: &mut E) {
        if self.hover_style.is_none() { return; }
        let hover_style = self.hover_style.as_ref().unwrap();
        
        unsafe {
            if TIMING_MANAGER.is_present(elem.state().last_animation) {
                TIMING_MANAGER.cancel(elem.state().last_animation);
            } else {
                self.initial_style = Some(elem.style().clone());
            }
        }

        let static_elem = unsafe { Unsafe::cast_mut_static(elem) };
        let static_from = unsafe { Unsafe::cast_static(self.initial_style.as_ref().unwrap()) };
        let static_to = unsafe { Unsafe::cast_static(hover_style) };

        let fade_time = self.fade_time;
        let easing = self.easing.clone();
        
        let id = unsafe {
            TIMING_MANAGER.request(
                DurationTask::new(
                    fade_time,
                    move |_, time| {
                        let percent = (time as f32).percentage(fade_time as f32);
                        let percent = easing.get(percent);

                        let static_style = Unsafe::cast_mut_static(static_elem.style_mut());
                        static_style.interpolate(
                            static_from,
                            static_to,
                            percent,
                            static_elem,
                            |s| s,
                        );

                        if percent >= 100.0 {
                            static_elem.style_mut().clone_from(static_to);
                        }
                    },
                    AnimationState::empty(),
                ),
                Some(Box::new(|| {})),
            )
        };
        elem.state_mut().last_animation = id;
    }

    fn start_animation_out(&mut self, elem: &mut E) {
        if self.initial_style.is_none() { return; }
        let initial_style = self.initial_style.as_ref().unwrap();

        unsafe {
            if TIMING_MANAGER.is_present(elem.state().last_animation) {
                TIMING_MANAGER.cancel(elem.state().last_animation);
            }
        }

        let static_elem = unsafe { Unsafe::cast_mut_static(elem) };
        let static_from = unsafe { Unsafe::cast_static(self.hover_style.as_ref().unwrap()) };
        let static_to = unsafe { Unsafe::cast_static(initial_style) };

        let fade_time = self.fade_time;
        let easing = self.easing.clone();

        let id = unsafe {
            TIMING_MANAGER.request(
                DurationTask::new(
                    fade_time,
                    move |_, time| {
                        let percent = (time as f32).percentage(fade_time as f32);
                        let percent = easing.get(percent);

                        let static_style = Unsafe::cast_mut_static(static_elem.style_mut());
                        static_style.interpolate(
                            static_from,
                            static_to,
                            percent,
                            static_elem,
                            |s| s,
                        );

                        if percent >= 100.0 {
                            static_elem.style_mut().clone_from(static_to);
                        }
                    },
                    AnimationState::empty(),
                ),
                Some(Box::new(|| {})),
            )
        };
        elem.state_mut().last_animation = id;
    }

    pub fn draw(&mut self, elem: &E, ctx: &mut DrawContext2D, context: &UiContext) {
        let res = context.resources;

        let resolved = resolve!(elem, background.shape);
        let resource = resolve!(elem, background.resource);
        if resolved.is_set() && !resource.is_none() {
            let resolved = resolved.unwrap();
            match resolved.deref().clone() {
                UiShape::Shape(s) => {
                    let shape = get_shape!(res, s).ok();
                    if let Some(shape) = shape {
                        Self::draw_background_shape(shape.clone(), elem, ctx, context);
                    }
                }
                UiShape::Adaptive(a) => {
                    let shape = get_adaptive!(res, a).ok();
                    if let Some(shape) = shape {
                        Self::draw_background_adaptive(shape, elem, ctx, context);
                    }
                }
            }
        }

        let resolved = resolve!(elem, border.shape);
        let resource = resolve!(elem, border.resource);
        if resolved.is_set() && !resource.is_none() {
            let resolved = resolved.unwrap();
            match resolved.deref().clone() {
                UiShape::Shape(s) => {
                    let shape = get_shape!(res, s).ok();
                    if let Some(shape) = shape {
                        Self::draw_border_shape(shape.clone(), elem, ctx, context);
                    }
                }
                UiShape::Adaptive(a) => {
                    let shape = get_adaptive!(res, a).ok();
                    if let Some(shape) = shape {
                        Self::draw_border_adaptive(shape, elem, ctx, context);
                    }
                }
            }
        }
    }

    fn draw_background_shape(
        mut background_shape: Shape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
    ) {
        let state = elem.state();
        let style = elem.style();
        let bounds = &state.rect;

        background_shape.invalidate();
        let (bgsw, bgsh) = background_shape.extent;
        let bg_scale_x = bounds.width() as f32 / bgsw as f32;
        let bg_scale_y = bounds.height() as f32 / bgsh as f32;
        let tmp = resolve!(elem, background.resource);
        if !tmp.is_set() {
            return;
        }
        let bg_res = tmp.unwrap();
        let bg_res = bg_res.deref();
        let mut bg_empty = false;
        match bg_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, background.color);
                if color.is_set() {
                    background_shape.set_color(color.unwrap());
                } else {
                    bg_empty = true;
                }
            }
            BackgroundRes::Texture => {
                if !style.background.texture.is_set() {
                    bg_empty = true;
                } else {
                    let tex = resolve!(elem, background.texture);
                    if let ResolveResult::Value(tex) = tex {
                        let tex = tex.deref().clone();
                        if let Some(tex) = context.resources.resolve_texture(tex) {
                            background_shape.set_texture(ctx::texture().source(Some(tex.clone())));
                        } else {
                            bg_empty = true;
                        }
                    } else {
                        bg_empty = true;
                    }
                }
            }
        }
        if !bg_empty {
            background_shape.set_translate(state.rect.x(), state.rect.y());
            background_shape.apply_transformations();
            background_shape.set_origin(state.rect.x(), state.rect.y());
            background_shape.set_scale(bg_scale_x, bg_scale_y);
            background_shape.apply_transformations();
            let ui_transform = state.inner_transforms.as_render_transform(state);
            background_shape.set_transform(ui_transform);
            ctx.shape(background_shape);
        }
    }

    fn draw_border_shape(
        mut border_shape: Shape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
    ) {
        let state = elem.state();
        let style = elem.style();
        let bounds = &state.rect;

        let (bdsw, bdsd) = border_shape.extent;
        let bd_scale_x = bounds.width() as f32 / bdsw as f32;
        let bd_scale_y = bounds.height() as f32 / bdsd as f32;
        let tmp = resolve!(elem, border.resource);
        if !tmp.is_set() {
            return;
        }
        let bd_res = tmp.unwrap();
        let bd_res = bd_res.deref();
        let mut bd_empty = false;
        match bd_res {
            BackgroundRes::Color => {
                let color = resolve!(elem, border.color);
                if color.is_set() {
                    border_shape.set_color(color.unwrap());
                } else {
                    bd_empty = true;
                }
            }
            BackgroundRes::Texture => {
                if !style.border.texture.is_set() {
                    bd_empty = true;
                } else {
                    let tex = resolve!(elem, border.texture);
                    if let ResolveResult::Value(tex) = tex {
                        let tex = tex.deref().clone();
                        if let Some(tex) = context.resources.resolve_texture(tex) {
                            border_shape.set_texture(ctx::texture().source(Some(tex.clone())));
                        } else {
                            bd_empty = true;
                        }
                    } else {
                        bd_empty = true;
                    }
                }
            }
        }
        if !bd_empty {
            border_shape.set_scale(bd_scale_x, bd_scale_y);
            ctx.shape(border_shape);
        }
    }

    fn draw_background_adaptive(
        bg_shape: &AdaptiveShape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
    ) {
        let rect = &elem.state().content_rect;
        let res = resolve!(elem, background.resource).unwrap_or(BackgroundRes::Color.into());
        let res = res.deref();
        let fill = match res {
            BackgroundRes::Color => {
                let color = resolve!(elem, background.color).unwrap_or(RgbColor::white());
                AdaptiveFill::Color(color)
            }
            BackgroundRes::Texture => {
                let texture = resolve!(elem, background.texture).unwrap_or(MVR.texture.missing.into());
                let texture = *texture.deref();
                AdaptiveFill::Drawable(Drawable::Texture(texture))
            }
        };
        bg_shape.draw(ctx, rect, fill, context);
    }

    fn draw_border_adaptive(
        bd_shape: &AdaptiveShape,
        elem: &E,
        ctx: &mut DrawContext2D,
        context: &UiContext,
    ) {
        let rect = &elem.state().content_rect;
        let res = resolve!(elem, border.resource).unwrap_or(BackgroundRes::Color.into());
        let res = res.deref();
        let fill = match res {
            BackgroundRes::Color => {
                let color = resolve!(elem, border.color).unwrap_or(RgbColor::white());
                AdaptiveFill::Color(color)
            }
            BackgroundRes::Texture => {
                let texture = resolve!(elem, border.texture).unwrap_or(MVR.texture.missing.into());
                let texture = *texture.deref();
                AdaptiveFill::Drawable(Drawable::Texture(texture))
            }
        };
        bd_shape.draw(ctx, rect, fill, context);
    }

    pub fn set_fade_time(&mut self, fade_time: u32) {
        self.fade_time = fade_time;
    }

    pub fn set_hover_style(&mut self, hover_style: Option<UiStyle>) {
        self.hover_style = hover_style;
    }

    pub fn set_easing(&mut self, easing: Easing) {
        self.easing = easing;
    }
}

