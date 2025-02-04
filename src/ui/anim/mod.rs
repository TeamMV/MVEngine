pub mod complex;

use crate::ui::ease::{Easing, EasingGen, EasingMode};
use crate::ui::elements::{UiElement, UiElementStub};
use crate::ui::styles::{Interpolator, UiStyle};
use crate::ui::timing::{AnimationState, DurationTask, TIMING_MANAGER};
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::Percentage;

pub fn easing(gen: EasingGen, mode: EasingMode) -> Easing {
    Easing::new(gen, mode, 0.0..100.0, 0.0..100.0)
}

///Specifies if the end result of the animation should be kept or reverted to the initial style
#[derive(Clone)]
pub enum FillMode {
    Keep,
    Revert,
}

///Specifies how to handle an animation call, when this element is being animated already
#[derive(Clone)]
pub enum AnimationMode {
    StartOver,
    BlockNew,
    KeepProgress,
}

///MAKE SURE TO NOT DROP ELEM DURING ANIMATION
pub fn animate_self(
    elem: &mut UiElement,
    target: &UiStyle,
    time_ms: u32,
    easing: Easing,
    fill_mode: FillMode,
    animation_mode: AnimationMode,
) -> u64 {
    let style = unsafe { Unsafe::cast_mut_static(elem.style_mut()) };
    animate(
        elem,
        style,
        target,
        time_ms,
        easing,
        fill_mode,
        animation_mode,
    )
}

///MAKE SURE TO NOT DROP ELEM DURING ANIMATION
pub fn animate(
    elem: &mut UiElement,
    initial: &mut UiStyle,
    target: &UiStyle,
    time_ms: u32,
    easing: Easing,
    fill_mode: FillMode,
    animation_mode: AnimationMode,
) -> u64 {
    if !elem.state().is_animating {
        elem.state_mut().is_animating = true;
    } else {
        match animation_mode {
            AnimationMode::StartOver => {
                unsafe {
                    if TIMING_MANAGER.is_present(elem.state().last_animation) {
                        //extra check cuz why not and i dont want crash or smth
                        TIMING_MANAGER.cancel(elem.state().last_animation);
                    }

                    if elem.state().last_style.is_some() {
                        let backup = Unsafe::cast_static(elem.state().last_style.as_ref().unwrap());
                        elem.style_mut().clone_from(backup);
                    }
                }
            }
            AnimationMode::KeepProgress => {
                unsafe {
                    if TIMING_MANAGER.is_present(elem.state().last_animation) {
                        //extra check cuz why not and i dont want crash or smth
                        TIMING_MANAGER.cancel(elem.state().last_animation);
                    }
                }
            }
            AnimationMode::BlockNew => return elem.state().last_animation,
        }
    }

    elem.state_mut().last_style = Some(initial.clone());

    let static_elem = unsafe { Unsafe::cast_mut_static(elem) };

    let id = unsafe {
        TIMING_MANAGER.request(
            DurationTask::new(
                time_ms,
                move |state, time| match state.element {
                    None => {}
                    Some(ref mut em) => {
                        let percent = (time as f32).percentage(time_ms as f32);
                        let percent = em.easing.get(percent);

                        let backup_style = em.elem.state().last_style.as_ref().unwrap();

                        let elem_ref = em.elem;
                        em.initial
                            .interpolate(backup_style, em.target, percent, elem_ref, |s| s);

                        if percent >= 100.0 {
                            match em.fill_mode {
                                FillMode::Keep => {
                                    //let elem_style = guard.style_mut();
                                    em.initial.clone_from(em.target);
                                }
                                FillMode::Revert => {
                                    //let elem_style = guard.style_mut();
                                    em.initial.clone_from(backup_style);
                                }
                            }
                        }
                    }
                },
                AnimationState::element(ElementAnimationInfo::new(
                    time_ms, fill_mode, easing, initial, target, elem,
                )),
            ),
            Some(Box::new(move || {
                static_elem.state_mut().is_animating = false
            })),
        )
    };
    elem.state_mut().last_animation = id;
    id
}

pub(crate) struct ElementAnimationInfo {
    pub(crate) fill_mode: FillMode,
    pub(crate) duration: u32,
    pub(crate) easing: Easing,
    pub(crate) initial: &'static mut UiStyle,
    pub(crate) target: &'static UiStyle,
    pub(crate) elem: &'static UiElement,
}

impl ElementAnimationInfo {
    pub(crate) fn new(
        duration_ms: u32,
        fill_mode: FillMode,
        easing: Easing,
        initial: &mut UiStyle,
        target: &UiStyle,
        elem: &UiElement,
    ) -> Self {
        unsafe {
            Self {
                fill_mode,
                duration: duration_ms,
                easing,
                initial: Unsafe::cast_mut_static(initial),
                target: Unsafe::cast_static(target),
                elem: Unsafe::cast_static(elem),
            }
        }
    }
}
