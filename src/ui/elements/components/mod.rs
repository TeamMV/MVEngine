// yeah fuck this shit I know my code will work safely (50% accuracy on that statement)
#![allow(static_mut_refs)]

pub mod drag;
pub mod edittext;
pub mod scroll;
pub mod boring;
pub mod text;

use crate::input::{Input, MouseAction, RawInputEvent};
use crate::ui::anim::easing;
use crate::ui::context::UiContext;
use crate::ui::ease::{Easing, EasingGen, EasingMode};
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::{shape, SimpleRect};
use crate::ui::styles::interpolate::Interpolator;
use crate::ui::styles::UiStyle;
use crate::ui::timing::{AnimationState, DurationTask, TIMING_MANAGER};
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::Percentage;
use std::ops::Deref;
use crate::rendering::RenderContext;

#[derive(Clone)]
enum State {
    In,
    Out,
}

#[derive(Clone)]
pub struct ElementBody {
    fade_time: u32,
    hover_style: Option<UiStyle>,
    hover_state: State,
    easing: Easing,
    initial_style: Option<UiStyle>,
}

impl ElementBody {
    pub fn new() -> Self {
        Self {
            fade_time: 0,
            hover_style: None,
            hover_state: State::Out,
            easing: easing(EasingGen::linear(), EasingMode::In),
            initial_style: None,
        }
    }

    pub fn on_input<E: UiElementStub + 'static>(
        &mut self,
        e: &mut E,
        action: RawInputEvent,
        _: &Input,
    ) {
        match action {
            RawInputEvent::Keyboard(_) => {}
            RawInputEvent::Mouse(ma) => match ma {
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
            },
        };
    }

    fn start_animation_in<E: UiElementStub + 'static>(&mut self, elem: &mut E) {
        if self.hover_style.is_none() {
            return;
        }
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

    fn start_animation_out<E: UiElementStub + 'static>(&mut self, elem: &mut E) {
        if self.initial_style.is_none() {
            return;
        }
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

    pub fn draw<E: UiElementStub + 'static>(
        &mut self,
        elem: &E,
        ctx: &mut impl RenderContext,
        context: &UiContext,
        crop_area: &SimpleRect, //TODO
    ) {
        let rect = elem.state().rect.bounding.clone();
        let style = elem.style();
        shape::utils::draw_shape_style_at(ctx, context, &rect, &style.background, elem, |s| &s.background);
        shape::utils::draw_shape_style_at(ctx, context, &rect, &style.border, elem, |s| &s.border);
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

    pub fn set_initial_style(&mut self, initial_style: UiStyle) {
        self.initial_style = Some(initial_style);
    }

    pub fn hover_style(&self) -> &Option<UiStyle> {
        &self.hover_style
    }

    pub fn initial_style(&self) -> &Option<UiStyle> {
        &self.initial_style
    }

    pub fn hover_style_mut(&mut self) -> &mut Option<UiStyle> {
        &mut self.hover_style
    }

    pub fn initial_style_mut(&mut self) -> &mut Option<UiStyle> {
        &mut self.initial_style
    }
}
