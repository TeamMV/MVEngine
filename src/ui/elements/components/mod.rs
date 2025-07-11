// yeah fuck this shit I know my code will work safely (50% accuracy on that statement)
#![allow(static_mut_refs)]

pub mod boring;
pub mod drag;
pub mod edittext;
pub mod scroll;
pub mod text;

use crate::input::{Input, MouseAction, RawInputEvent};
use crate::rendering::RenderContext;
use crate::ui::anim;
use crate::ui::anim::UiAnim;
use crate::ui::context::UiContext;
use crate::ui::ease::{Easing, EasingGen, EasingMode};
use crate::ui::elements::UiElementStub;
use crate::ui::geometry::{SimpleRect, shape};
use crate::ui::styles::UiStyle;
use crate::ui::styles::interpolate::Interpolator;
use mvutils::utils::Percentage;
use std::ops::Deref;
use crate::ui::elements::components::scroll::ScrollBars;

#[derive(Clone)]
enum State {
    In,
    Out,
}

#[derive(Clone)]
pub struct ElementBody {
    fade_time: u64,
    hover_state: State,
    hover_style: Option<UiStyle>,
    init_style: Option<UiStyle>,
    easing: Easing,
    scroll_bars: ScrollBars
}

impl ElementBody {
    pub fn new() -> Self {
        Self {
            fade_time: 0,
            hover_state: State::Out,
            hover_style: None,
            init_style: None,
            easing: anim::easing(EasingGen::linear(), EasingMode::InOut),
            scroll_bars: ScrollBars {},
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
        if let Some(hover) = &self.hover_style {
            self.init_style = elem.style().clone().into();
            let state = elem.state_mut();
            //state.animator.animate(UiAnim::new_simple(hover.clone(), self.easing.clone(), self.fade_time), elem.style());
        }
    }

    fn start_animation_out<E: UiElementStub + 'static>(&mut self, elem: &mut E) {
        if let Some(init) = &self.init_style {
            let state = elem.state_mut();
            //state.animator.animate(UiAnim::new_simple(init.clone(), self.easing.clone(), self.fade_time), elem.style());
        }
    }

    pub fn draw<E: UiElementStub + 'static>(
        &mut self,
        elem: &E,
        ctx: &mut impl RenderContext,
        context: &UiContext,
        crop_area: &SimpleRect,
    ) {
        let rect = elem.state().rect.bounding.clone();
        let style = elem.style();
        shape::utils::draw_shape_style_at(
            ctx,
            context,
            &rect,
            &style.background,
            elem,
            |s| &s.background,
            Some(crop_area.clone()),
        );
        shape::utils::draw_shape_style_at(
            ctx,
            context,
            &rect,
            &style.border,
            elem,
            |s| &s.border,
            Some(crop_area.clone()),
        );
    }

    pub fn draw_height_square<E: UiElementStub + 'static>(
        &mut self,
        elem: &E,
        ctx: &mut impl RenderContext,
        context: &UiContext,
        crop_area: &SimpleRect,
    ) {
        let rect = &elem.state().rect.bounding;
        let rect = SimpleRect::new(rect.x, rect.y, rect.height, rect.height);
        let style = elem.style();
        shape::utils::draw_shape_style_at(
            ctx,
            context,
            &rect,
            &style.background,
            elem,
            |s| &s.background,
            Some(crop_area.clone()),
        );
        shape::utils::draw_shape_style_at(
            ctx,
            context,
            &rect,
            &style.border,
            elem,
            |s| &s.border,
            Some(crop_area.clone()),
        );
    }
    
    pub fn draw_scrollbars<E: UiElementStub + 'static>(&mut self, elem: &E,
                           ctx: &mut impl RenderContext,
                           context: &UiContext,
                           crop_area: &SimpleRect) {
        self.scroll_bars.draw(elem, ctx, context, crop_area);
    }
}
