use crate::game::timing::{DurationTask, TaskId};
use crate::ui::context::UiContext;
use crate::ui::ease::{Easing, EasingGen, EasingMode};
use crate::ui::elements::UiElementStub;
use crate::ui::styles::UiStyle;
use crate::ui::styles::interpolate::Interpolator;
use mvutils::unsafe_utils::Unsafe;
use std::convert::identity;

pub fn easing(ease_gen: EasingGen, mode: EasingMode) -> Easing {
    Easing::new(ease_gen, mode, 0.0..1.0, 0.0..1.0)
}

pub struct ElementAnimator {
    context: UiContext,
    anims: Vec<UiAnim>,
}

impl Clone for ElementAnimator {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            anims: vec![],
        }
    }
}

impl ElementAnimator {
    pub fn new(context: UiContext) -> Self {
        Self {
            context,
            anims: vec![],
        }
    }

    pub fn animate(&mut self, mut anim: UiAnim, s: &UiStyle) {
        match &mut anim {
            UiAnim::Simple(simple) => simple.start(s, &mut self.context),
            UiAnim::Complex => {}
        }
        self.anims.push(anim);
    }

    pub fn tick<E: UiElementStub + 'static>(&mut self, elem: &mut E) {
        self.anims.retain(|a| match a {
            UiAnim::Simple(simple) => simple.tick(elem),
            UiAnim::Complex => false,
        });
    }
}

pub enum UiAnim {
    Simple(SimpleAnim),
    Complex,
}

impl UiAnim {
    pub fn new_simple(target: UiStyle, easing: Easing, duration_ms: u64) -> Self {
        Self::Simple(SimpleAnim {
            task: None,
            target,
            initial: None,
            easing,
            duration_ms,
        })
    }
}

pub struct SimpleAnim {
    task: Option<TaskId>,
    target: UiStyle,
    initial: Option<UiStyle>,
    easing: Easing,
    duration_ms: u64,
}

impl SimpleAnim {
    pub fn start(&mut self, initial: &UiStyle, context: &mut UiContext) {
        self.initial = Some(initial.clone());
        let scheduler = &mut context.scheduler;
        let id = scheduler.queue(DurationTask::new(self.duration_ms));
        self.task = Some(id);
    }

    pub fn tick<E: UiElementStub + 'static>(&self, elem: &mut E) -> bool {
        let mut context = elem.context().clone();
        let static_elem = unsafe { Unsafe::cast_mut_static(elem) };
        let style = static_elem.style_mut();
        if let Some(id) = self.task
            && let Some(mut handle) = context.scheduler.handle(id)
        {
            if handle.tick() {
                let pg = handle.progress();
                let pg = self.easing.get(pg as f32);
                if pg >= 1.0 {
                    style.clone_from(&self.target);
                    false
                } else {
                    if let Some(init) = &self.initial {
                        style.interpolate(init, &self.target, pg * 100.0, elem, |s| s);
                    }
                    true
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}
