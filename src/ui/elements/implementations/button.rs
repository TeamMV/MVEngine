use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::Percentage;
use crate::ui::anim::{easing, ElementAnimationInfo, FillMode};
use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::ease::{Easing, EasingGen, EasingMode};
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::elements::child::Child;
use crate::ui::elements::components::ElementBody;
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::styles::{Dimension, Interpolator, UiStyle, EMPTY_STYLE};
use crate::ui::timing::{AnimationState, DurationTask, TIMING_MANAGER};

#[derive(Clone)]
enum State {
    In,
    Out
}

#[derive(Clone)]
pub struct Button {
    context: UiContext,
    state: UiElementState,
    style: UiStyle,
    initial_style: UiStyle,
    attributes: Attributes,
    body: ElementBody<Button>,
    hover_style: UiStyle,
    fade_time: u32,
    easing: Easing,
    hover_state: State,
}

impl Button {
    pub fn set_hover_style(&mut self, hover_style: UiStyle) {
        self.hover_style = hover_style;
    }

    pub fn set_fade_time(&mut self, fade_time: u32) {
        self.fade_time = fade_time;
    }

    pub fn set_easing(&mut self, easing: Easing) {
        self.easing = easing;
    }
    
    fn start_animation_in(&mut self) {
        unsafe {
            if TIMING_MANAGER.is_present(self.state.last_animation) {
                TIMING_MANAGER.cancel(self.state.last_animation);
            }
        }

        let static_elem = unsafe { Unsafe::cast_mut_static(self) };
        let static_from = unsafe { Unsafe::cast_static(&self.initial_style) };
        let static_to = unsafe { Unsafe::cast_static(&self.hover_style) };

        let id = unsafe {
            TIMING_MANAGER.request(
                DurationTask::new(
                    static_elem.fade_time,
                    |_, time| {
                        let percent = (time as f32).percentage(static_elem.fade_time as f32);
                        let percent = static_elem.easing.get(percent);

                        let static_style = Unsafe::cast_mut_static(&mut static_elem.style);
                        static_style.interpolate(static_from, static_to, percent, static_elem, |s| s);

                        if percent >= 100.0 {
                            static_elem.style.clone_from(static_to);
                        }
                    },
                    AnimationState::empty(),
                ),
                Some(Box::new(|| {})),
            )
        };
        self.state.last_animation = id;
    }

    fn start_animation_out(&mut self) {
        unsafe {
            if TIMING_MANAGER.is_present(self.state.last_animation) {
                TIMING_MANAGER.cancel(self.state.last_animation);
            }
        }

        let static_elem = unsafe { Unsafe::cast_mut_static(self) };
        let static_from = unsafe { Unsafe::cast_static(&self.hover_style) };
        let static_to = unsafe { Unsafe::cast_static(&self.initial_style) };

        let id = unsafe {
            TIMING_MANAGER.request(
                DurationTask::new(
                    static_elem.fade_time,
                    |_, time| {
                        let percent = (time as f32).percentage(static_elem.fade_time as f32);
                        let percent = static_elem.easing.get(percent);

                        let static_style = Unsafe::cast_mut_static(&mut static_elem.style);
                        static_style.interpolate(static_from, static_to, percent, static_elem, |s| s);

                        if percent >= 100.0 {
                            static_elem.style.clone_from(static_to);
                        }
                    },
                    AnimationState::empty(),
                ),
                Some(Box::new(|| {})),
            )
        };
        self.state.last_animation = id;
    }
}

impl UiElementCallbacks for Button {
    fn draw(&mut self, ctx: &mut DrawContext2D) {

        //todo movie into ui action processor (for each element have a fn(action: RawInputEvent))
        //let (mx, my) = (input.positions[0], input.positions[1]);
        //if self.inside(mx, my) {
        //    if let State::Out = self.hover_state {
        //        self.hover_state = State::In;
        //        self.initial_style = self.style.clone();
        //        if self.fade_time == 0 {
        //            self.style.clone_from(&self.hover_style);
        //        } else {
        //            self.start_animation_in();
        //        }
        //    }
        //} else {
        //    if let State::In = self.hover_state {
        //        self.hover_state = State::Out;
        //        if self.fade_time == 0 {
        //            self.style.clone_from(&self.initial_style);
        //        } else {
        //            self.start_animation_out();
        //        }
        //    }
        //}

        let this = unsafe { Unsafe::cast_static(self) };
        self.body.draw(this, ctx, &self.context);
        for children in &self.state.children {
            match children {
                Child::String(_) => {}
                Child::Element(e) => {
                    let mut guard = e.get_mut();
                    guard.draw(ctx);
                }
                Child::State(_) => {}
            }
        }
    }
}

impl UiElementStub for Button {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized
    {
        Self {
            context: context.clone(),
            state: UiElementState::new(),
            style: style.clone(),
            initial_style: style.clone(),
            attributes,
            body: ElementBody::new(),
            hover_style: style,
            fade_time: 0,
            easing: easing(EasingGen::linear(), EasingMode::In),
            hover_state: State::Out,
        }
    }

    fn wrap(self) -> UiElement {
        UiElement::Button(self)
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }

    fn state(&self) -> &UiElementState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut UiElementState {
        &mut self.state
    }

    fn style(&self) -> &UiStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut UiStyle {
        &mut self.style
    }

    fn components(&self) -> (&Attributes, &UiStyle, &UiElementState) {
        (&self.attributes, &self.style, &self.state)
    }

    fn components_mut(&mut self) -> (&mut Attributes, &mut UiStyle, &mut UiElementState) {
        (&mut self.attributes, &mut self.style, &mut self.state)
    }

    fn get_size(&self, s: &str) -> Dimension<i32> {
        todo!()
    }
}