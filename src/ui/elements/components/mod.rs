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
use crate::ui::elements::{UiElementState, UiElementStub};
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
    hover_state: State,
    active_style: Option<UiStyle>
}

impl ElementBody {
    pub fn new() -> Self {
        Self {
            hover_state: State::Out,
            active_style: None,
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
                            let s = e.style().get_hover();
                            self.active_style = Some(s);
                        }
                    } else {
                        if let State::In = self.hover_state {
                            self.hover_state = State::Out;
                            self.active_style = None;
                        }
                    }
                }
                MouseAction::Press(_) => {}
                MouseAction::Release(_) => {}
            },
        };
    }



    pub fn draw(
        &mut self,
        elem_style: &UiStyle,
        elem_state: &UiElementState,
        ctx: &mut impl RenderContext,
        context: &UiContext,
        crop_area: &SimpleRect,
    ) {
        let rect = elem_state.rect.bounding.clone();
        shape::utils::draw_shape_style_at(
            ctx,
            context,
            &rect,
            &elem_style.background,
            elem_state,
            self,
            |s| &s.background,
            Some(crop_area.clone()),
        );
        shape::utils::draw_shape_style_at(
            ctx,
            context,
            &rect,
            &elem_style.border,
            elem_state,
            self,
            |s| &s.border,
            Some(crop_area.clone()),
        );
    }

    pub fn draw_height_square(
        &mut self,
        elem_style: &UiStyle,
        elem_state: &UiElementState,
        ctx: &mut impl RenderContext,
        context: &UiContext,
        crop_area: &SimpleRect,
    ) {
        let rect = &elem_state.rect.bounding;
        let rect = SimpleRect::new(rect.x, rect.y, rect.height, rect.height);
        shape::utils::draw_shape_style_at(
            ctx,
            context,
            &rect,
            &elem_style.background,
            elem_state,
            self,
            |s| &s.background,
            Some(crop_area.clone()),
        );
        shape::utils::draw_shape_style_at(
            ctx,
            context,
            &rect,
            &elem_style.border,
            elem_state,
            self,
            |s| &s.border,
            Some(crop_area.clone()),
        );
    }
    
    pub fn draw_scrollbars(&mut self,
                           elem_style: &UiStyle,
                           elem_state: &UiElementState,
                           ctx: &mut impl RenderContext,
                           context: &UiContext,
                           crop_area: &SimpleRect
    ) {
        ScrollBars::draw(elem_style, elem_state, self, ctx, context, crop_area);
    }

    pub fn active_style(&self) -> Option<&UiStyle> {
        self.active_style.as_ref()
    }
}
