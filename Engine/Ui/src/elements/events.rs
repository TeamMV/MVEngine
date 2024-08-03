/*use std::ops::Deref;
use std::sync::Arc;
use mvutils::unsafe_utils::Unsafe;
use parking_lot::{RwLock, RwLockWriteGuard};
use crate::input::{InputProcessor, KeyboardAction, MouseAction};
use crate::input::raw::Input;
use crate::{input, ui};
use crate::ui::elements::{UiElement, UiElementState};
use crate::ui::elements::child::Child;
use crate::ui::styles::Point;

const CLICK_LISTENER: u32 = 0;

pub struct UiEvents {
    mouse: UiMouseEvents
}

impl UiEvents {
    pub fn create() -> Self {
        unsafe {
            Self {
                mouse: UiMouseEvents::new(),
            }
        }
    }

    pub(crate) fn mouse_change<E>(&mut self, action: MouseAction, elem: &E, input: &Input) -> bool where E: Deref<Target=dyn UiElement> + Sized {
        let state = elem.state();

        let mut free = true;
        for child in &state.children {
            match child {
                Child::Element(e) => unsafe {
                    let mut child_guard = e.write();
                    let guard_ref: &dyn Deref<Target=dyn UiElement>  = Unsafe::cast_ref(&child_guard);
                    let mut child_events = &mut child_guard.state_mut().events;
                    let res = child_events.mouse_change(action, guard_ref, input);
                    if free {
                        free = res;
                    }
                },
                _ => {}
            }
        }

        if free {
            let (mx, my) = (input.positions[0], input.positions[1]);

            if elem.inside(mx, my) {
                match action {
                    MouseAction::Wheel(dx, dy) => {}
                    MouseAction::Move(x, y) => {}
                    MouseAction::Press(b) => {
                        for listener in self.mouse.clicks {
                            listener(UiClickEvent::<'_, _> {
                                elem,
                                action: UiClickAction::Click,
                                button: b,
                                pos: Point::new(mx, my),
                                pos_rel: Point::new(mx - state.x, my - state.y),
                            });
                        }
                    }
                    MouseAction::Release(b) => {}
                }

                return false
            }
        }

        true
    }

    pub(crate) fn keyboard_change<E>(&mut self, action: KeyboardAction, elem: &E, input: &Input) -> bool where E: Deref<Target=dyn UiElement> {
        true
    }

    pub fn clear_listener(&mut self, id: u64) {
        let idx = (id >> 32) as u32;
        let kind = (id & 0xFFFFFFFF) as u32;
        match kind {
            CLICK_LISTENER => { let _ = self.mouse.clicks.swap_remove(idx as usize); },
            _ => unreachable!()
        }
    }

    pub fn on_click<F>(&mut self, f: F) -> u64 where F: Fn(UiClickEvent<dyn Deref<Target=dyn UiElement>>) {
        let idx = self.mouse.clicks.len();
        self.mouse.clicks.push(Box::new(f));
        (((idx as u32) << 32) | CLICK_LISTENER) as u64
    }
}

pub struct UiMouseEvents {
    clicks: Vec<Box<dyn Fn(UiClickEvent<dyn Deref<Target=dyn UiElement>>)>>
}

impl UiMouseEvents {
    pub fn new() -> Self {
        Self {
            clicks: vec![],
        }
    }
}

pub struct UiClickEvent<'a, E: Deref<Target=dyn UiElement> + ?Sized> {
    elem: &'a E,
    action: UiClickAction,
    button: usize,
    pos: Point<i32>,
    pos_rel: Point<i32>
}

impl<E: Deref<Target=dyn UiElement> + ?Sized> Clone for UiClickEvent<'_, E> {
    fn clone(&self) -> Self {
        Self {
            elem: &self.elem,
            action: self.action.clone(),
            button: self.button,
            pos: self.pos.clone(),
            pos_rel: self.pos_rel.clone(),
        }
    }
}

#[derive(Clone)]
pub enum UiClickAction { Click, Release }*/