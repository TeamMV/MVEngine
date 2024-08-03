use crate::ui::background::FillMode;
use crate::ui::ease::{Easing, EasingGen, EasingLinear, EasingSin, EasingSinIn, EasingSinOut};
use crate::ui::elements::UiElement;
use crate::ui::styles::{Interpolator, UiStyle};
use crate::ui::timing::{AnimationState, DurationTask, TIMING_MANAGER};
use mvutils::utils::Percentage;
use num_traits::ToPrimitive;
use parking_lot::RwLock;
use std::sync::Arc;
use mvutils::unsafe_utils::Unsafe;

pub const EASING_LINEAR: Easing = Easing {
    xr: 0.0..100.0,
    yr: 0.0..100.0,
    gen: EasingGen::Linear(EasingLinear::new(0.0..100.0, 0.0..100.0))
};

pub const EASING_SIN: Easing = Easing {
    xr: 0.0..100.0,
    yr: 0.0..100.0,
    gen: EasingGen::Sin(EasingSin::new(0.0..100.0, 0.0..100.0))
};

pub const EASING_SIN_IN: Easing = Easing {
    xr: 0.0..100.0,
    yr: 0.0..100.0,
    gen: EasingGen::SinIn(EasingSinIn::new(0.0..100.0, 0.0..100.0))
};

pub const EASING_SIN_OUT: Easing = Easing {
    xr: 0.0..100.0,
    yr: 0.0..100.0,
    gen: EasingGen::SinOut(EasingSinOut::new(0.0..100.0, 0.0..100.0))
};

pub fn animate(elem: Arc<RwLock<dyn UiElement>>, initial: &mut UiStyle, target: &UiStyle, time_ms: u32, easing: Easing, fill_mode: FillMode) -> u64 {
    let mut backup = None;
    if matches!(fill_mode, FillMode::Revert) {
        backup = Some(initial.clone());
    }

    let id = unsafe {
        TIMING_MANAGER.request(DurationTask::new(
            time_ms,
            move |state, time| match state.element {
                None => {}
                Some(ref mut em) => {
                    let percent = (time as f32).percentage(time_ms as f32);
                    let percent = em.easing.get(percent);

                    em.initial.interpolate(em.target, percent, em.elem.clone(), |s| s);

                    if percent >= 100.0 {
                        match em.fill_mode {
                            FillMode::Keep => {
                                //let elem_style = guard.style_mut();
                                em.initial.clone_from(em.target);
                            }
                            FillMode::Revert => {
                                //TODO: maybe get rid of this clone, its unnecessary
                                let backup_style = &em.backup_style.clone().unwrap();

                                //let elem_style = guard.style_mut();
                                em.initial.clone_from(backup_style);
                            }
                        }
                    }
                }
            },
            AnimationState::element(ElementAnimationInfo::new(
                time_ms,
                fill_mode,
                easing,
                initial,
                backup,
                target,
                elem
            )),
        ))
    };
    id
}

pub(crate) struct ElementAnimationInfo {
    pub(crate) fill_mode: FillMode,
    pub(crate) duration: u32,
    pub(crate) easing: Easing,
    pub(crate) initial: &'static mut UiStyle,
    pub(crate) backup_style: Option<UiStyle>,
    pub(crate) target: &'static UiStyle,
    pub(crate) elem: Arc<RwLock<dyn UiElement>>
}

impl ElementAnimationInfo {
    pub(crate) fn new(
        duration_ms: u32,
        fill_mode: FillMode,
        easing: Easing,
        initial: &mut UiStyle,
        backup_style: Option<UiStyle>,
        target: &UiStyle,
        elem: Arc<RwLock<dyn UiElement>>
    ) -> Self {
        unsafe {
            Self {
                fill_mode,
                duration: duration_ms,
                easing,
                initial: Unsafe::cast_mut_static(initial),
                backup_style,
                target: Unsafe::cast_static(target),
                elem
            }
        }
    }
}
