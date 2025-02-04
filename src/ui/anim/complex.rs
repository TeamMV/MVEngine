use crate::ui::anim;
use crate::ui::anim::{AnimationMode, FillMode};
use crate::ui::ease::{Easing, EasingGen, EasingMode};
use crate::ui::elements::{UiElement, UiElementStub};
use crate::ui::styles::UiStyle;
use crate::ui::timing::{AnimationState, DelayTask, TIMING_MANAGER};
use mvutils::unsafe_utils::{DangerousCell, Nullable, Unsafe};
use mvutils::utils::{key, Percentage};
use num_traits::{ToPrimitive, Zero};
use std::mem;
use std::sync::Arc;

pub enum UiElementAnimation {
    Simple(SimpleAnimation),
    Keyframe(KeyframeAnimation),
}

pub trait UiElementAnimationStub {
    fn play(
        &self,
        elem: &mut UiElement,
        duration: u32,
        fill_mode: FillMode,
        animation_mode: AnimationMode,
    );
}

pub struct SimpleAnimation {
    target_style: UiStyle,
    easing: Easing,
}

impl UiElementAnimationStub for SimpleAnimation {
    fn play(
        &self,
        elem: &mut UiElement,
        duration: u32,
        fill_mode: FillMode,
        animation_mode: AnimationMode,
    ) {
        anim::animate_self(
            elem,
            &self.target_style,
            duration,
            self.easing.clone(),
            fill_mode,
            animation_mode,
        );
    }
}

pub struct KeyframeAnimation {
    start_style: UiStyle,
    keyframes: Vec<Keyframe>,
}

impl KeyframeAnimation {
    pub fn builder(start_style: UiStyle) -> KeyframeAnimationBuilder {
        KeyframeAnimationBuilder::new(start_style)
    }

    fn animate_next_keyframe<F>(
        &self,
        index: usize,
        complete_duration: u32,
        elem: &mut UiElement,
        animation_mode: AnimationMode,
        on_finish: Arc<DangerousCell<F>>,
    ) where
        F: FnMut() + 'static,
    {
        unsafe {
            if index + 1 > self.keyframes.len() {
                return;
            }

            let keyframe = &self.keyframes[index];
            let duration = (keyframe.duration_percent).value(complete_duration as f32) as u32;

            anim::animate_self(
                elem,
                &keyframe.style,
                duration,
                keyframe.easing.clone(),
                FillMode::Keep,
                animation_mode.clone(),
            );

            let static_self = Unsafe::cast_static(self);
            let static_elem = Unsafe::cast_mut_static(elem);

            let next_index = index + 1;

            TIMING_MANAGER.request(
                DelayTask::new(duration, |_, _| {}, AnimationState::empty()),
                Some(Box::new(move || {
                    static_self.animate_next_keyframe(
                        next_index,
                        complete_duration,
                        static_elem,
                        animation_mode.clone(),
                        on_finish.clone(),
                    );
                    if next_index + 1 > static_self.keyframes.len() {
                        on_finish.get_mut()();
                    }
                })),
            );
        }
    }
}

impl UiElementAnimationStub for KeyframeAnimation {
    fn play(
        &self,
        elem: &mut UiElement,
        duration: u32,
        fill_mode: FillMode,
        animation_mode: AnimationMode,
    ) {
        let original_style = elem.style().clone();
        let current_style = unsafe { Unsafe::cast_mut_static(elem.style_mut()) };

        self.animate_next_keyframe(
            0,
            duration,
            elem,
            animation_mode,
            Arc::new(DangerousCell::new(move || match fill_mode {
                FillMode::Keep => {}
                FillMode::Revert => {
                    *current_style = original_style.clone();
                }
            })),
        );
    }
}

pub struct KeyframeAnimationBuilder {
    remaining_duration: f32,
    keyframes: Vec<Keyframe>,
    last_style: UiStyle,
    start_style: UiStyle,
}

impl KeyframeAnimationBuilder {
    fn new(start_style: UiStyle) -> Self {
        Self {
            remaining_duration: 100.0,
            keyframes: vec![],
            start_style: start_style.clone(),
            last_style: start_style,
        }
    }

    pub fn next_keyframe(
        mut self,
        modifier: fn(&mut UiStyle),
        easing: Option<Easing>,
        duration: Option<f32>,
    ) -> Self {
        let duration = if duration.is_some() {
            let mut duration = duration.unwrap();
            if duration > self.remaining_duration {
                duration = self.remaining_duration;
            }
            duration
        } else {
            self.remaining_duration
        };

        self.remaining_duration -= duration;

        if duration.is_zero() {
            return self;
        }

        modifier(&mut self.last_style);

        self.keyframes.push(Keyframe::new(
            self.last_style.clone(),
            easing.unwrap_or(anim::easing(EasingGen::linear(), EasingMode::In)),
            duration,
        ));

        self
    }

    pub fn build(self) -> KeyframeAnimation {
        KeyframeAnimation {
            start_style: self.start_style,
            keyframes: self.keyframes,
        }
    }
}

pub struct Keyframe {
    style: UiStyle,
    easing: Easing,
    duration_percent: f32,
}

impl Keyframe {
    pub fn new(style: UiStyle, easing: Easing, duration: f32) -> Self {
        Self {
            style,
            easing,
            duration_percent: duration,
        }
    }
}
