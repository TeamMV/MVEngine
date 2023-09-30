use std::sync::{Arc, RwLock};
use bytemuck::Contiguous;

use mvutils::utils::{Percentage, Recover};
use crate::gui::components::GuiElementInfo;
use crate::gui::ease::Easing;

use crate::gui::styles::{GuiStyle, GuiValue, Interpolator};
use crate::resolve;

#[derive(Copy, Clone)]
pub enum AnimationEasing {
    None,
    SinIn,
    SinOut,
    SinInOut
}

pub trait Manipulator {
    fn manipulate(&mut self, state: Arc<RwLock<GuiElementInfo>>, target: Arc<RwLock<GuiElementInfo>>, progress: f32, easing_type: AnimationEasing);
}

macro_rules! interpolate {
    ($target:expr, $state:expr, $ease:expr, $progress:expr, $($prop:ident,)*) => {
        $(
            let value = resolve!($state.read().recover(), $prop);
            let target_value = resolve!($target.read().recover(), $prop);
            $state.write().recover().style.$prop = GuiValue::Just(value.interpolate(value, value, target_value, $progress, $ease));
        )*
    };
}

pub struct KeyframeManipulator {
    keyframes: Vec<(f32, Arc<RwLock<GuiElementInfo>>)>,
    current_frame: (f32, Arc<RwLock<GuiElementInfo>>)
}

impl Manipulator for KeyframeManipulator {
    fn manipulate(&mut self, mut state: Arc<RwLock<GuiElementInfo>>, target: Arc<RwLock<GuiElementInfo>>, progress: f32, easing_type: AnimationEasing) {
        if progress > self.current_frame.0 {
            let next : (usize, &(f32, Arc<RwLock<GuiElementInfo>>)) = self.keyframes.iter().enumerate().find(|(_, time)| time.0 < progress).unwrap();
            self.current_frame = next.1.clone();
        }
        let easing = get_easing(easing_type);

        let value = resolve!(state.read().recover(), x);
        let target_value = resolve!(self.current_frame.1.read().recover(), x);
        state.write().recover().style.x = GuiValue::Just(value.interpolate(value, value, target_value, progress, easing));

        //let fields =

        //interpolate!(self.current_frame.1, state, easing, progress,

        //);

        if progress >= 100.0 {
            state = target;
        }
    }
}

fn get_easing(ease_type: AnimationEasing) -> Easing {
    match ease_type {
        AnimationEasing::None => {Easing::new_linear(0.0..100.0, 0.0..1.0)}
        AnimationEasing::SinIn => {Easing::new_sin_in(0.0..100.0, 0.0..1.0)}
        AnimationEasing::SinOut => {Easing::new_sin_out(0.0..100.0, 0.0..1.0)}
        AnimationEasing::SinInOut => {Easing::new_sin(0.0..100.0, 0.0..1.0)}
    }
}

pub(crate) struct ElementAnimation {
    state: Arc<RwLock<GuiStyle>>,
    target: Arc<RwLock<GuiStyle>>,
    duration: f32,
    current_time: f32,
    easing: AnimationEasing,
    manipulator: Box<dyn Manipulator>
}

pub(crate) struct Choreographer {
    queue: Vec<ElementAnimation>
}

impl Choreographer {
    pub(crate) fn is_frame_requested(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn queue(&mut self, anim: ElementAnimation) {
        self.queue.push(anim)
    }

    pub(crate) fn frame(&mut self, fps: i32, dt: f32) {
        let mut removals = vec![];
        for (i, anim) in self.queue.iter_mut().enumerate() {
            if anim.current_time >= anim.duration {
                removals.push(i);
                continue;
            }

            anim.current_time += (1.0 / fps as f32) * dt;
            //anim.manipulator.manipulate(anim.state.clone(), anim.current_time.percentage(anim.duration), anim.easing);
        }

        for idx in removals {
            self.queue.remove(idx);
        }
    }
}