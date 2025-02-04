use crate::ui::elements::child::Child;
use crate::ui::elements::{UiElement, UiElementStub};
use crate::ui::styles::Point;
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::TetrahedronOp;
use crate::input::{Input, KeyboardAction, MouseAction};
use crate::input::consts::MouseButton;

const CLICK_LISTENER: u32 = 0;
const SCROLL_LISTENER: u32 = 1;
const HOVER_LISTENER: u32 = 2;
const MOVE_LISTENER: u32 = 3;
const GLOBAL_MOVE_LISTENER: u32 = 4;

pub struct UiEvents {
    last_inside: bool,
    mouse: UiMouseEvents,
}

impl UiEvents {
    pub fn create() -> Self {
        unsafe {
            Self {
                last_inside: false,
                mouse: UiMouseEvents::new(),
            }
        }
    }

    pub(crate) fn mouse_change(
        &mut self,
        action: MouseAction,
        elem: &mut UiElement,
        input: &Input,
    ) -> bool {
        let state = elem.state();
        let state = unsafe { Unsafe::cast_static(state) };

        let mut used = false;
        for child in &state.children {
            match child {
                Child::Element(e) => unsafe {
                    let mut child_guard = e.get_mut();
                    let child_events = &mut child_guard.state_mut().events;
                    let mut child_events = Unsafe::cast_mut_static(child_events);
                    let res = child_events.mouse_change(action, (&mut *child_guard), input);
                    if res {
                        used = res;
                    }
                },
                _ => {}
            }
        }

        let (mx, my) = (input.mouse_x, input.mouse_y);

        match action {
            MouseAction::Move(max, may) => {
                for listener in &mut self.mouse.global_moves {
                    let base = UiEventBase {
                        elem,
                        action: UiMoveAction::Moving,
                        pos: Point::new(max, may),
                        pos_rel: Point::new(max - state.rect.x(), may - state.rect.y()),
                        synthetic: false,
                    };

                    listener(UiMoveEvent::<'_> { base });
                }

                if self.last_inside {
                    if !elem.inside(max, may) {
                        self.last_inside = false;

                        for listener in &mut self.mouse.hovers {
                            let base = UiEventBase {
                                elem,
                                action: UiHoverAction::Leave,
                                pos: Point::new(max, may),
                                pos_rel: Point::new(max - state.rect.x(), may - state.rect.y()),
                                synthetic: false,
                            };

                            listener(UiHoverEvent::<'_> { base });
                        }
                    }
                } else {
                    if elem.inside(max, may) {
                        self.last_inside = true;

                        for listener in &mut self.mouse.hovers {
                            let base = UiEventBase {
                                elem,
                                action: UiHoverAction::Enter,
                                pos: Point::new(max, may),
                                pos_rel: Point::new(max - state.rect.x(), may - state.rect.y()),
                                synthetic: false,
                            };

                            listener(UiHoverEvent::<'_> { base });
                        }
                    }
                }
            }
            _ => {}
        }

        if !used {
            if elem.inside(mx, my) {
                match action {
                    MouseAction::Wheel(dx, dy) => {
                        let mut action = UiScrollAction::X(0f32);
                        let mut dir = UiScrollDirection::Up;
                        if dx != 0.0 {
                            action = UiScrollAction::X(dx);
                            dir = (dx > 0.0).yn(UiScrollDirection::Right, UiScrollDirection::Left);
                            if dy != 0.0 {
                                action = UiScrollAction::Both(dx, dy);
                                dir = (dx > 0.0).yn(
                                    (dx > 0.0)
                                        .yn(UiScrollDirection::UpRight, UiScrollDirection::UpLeft),
                                    (dx > 0.0).yn(
                                        UiScrollDirection::DownRight,
                                        UiScrollDirection::DownLeft,
                                    ),
                                );
                            }
                        } else if dy != 0.0 {
                            action = UiScrollAction::Y(dy);
                            dir = (dy > 0.0).yn(UiScrollDirection::Up, UiScrollDirection::Down);
                            if dx != 0.0 {
                                action = UiScrollAction::Both(dx, dy);
                                dir = (dx > 0.0).yn(
                                    (dx > 0.0)
                                        .yn(UiScrollDirection::UpRight, UiScrollDirection::UpLeft),
                                    (dx > 0.0).yn(
                                        UiScrollDirection::DownRight,
                                        UiScrollDirection::DownLeft,
                                    ),
                                );
                            }
                        }
                        for listener in &mut self.mouse.scrolls {
                            let base = UiEventBase {
                                elem,
                                action: action.clone(),
                                pos: Point::new(mx, my),
                                pos_rel: Point::new(mx - state.rect.x(), my - state.rect.y()),
                                synthetic: false,
                            };

                            listener(UiScrollEvent::<'_> {
                                base,
                                direction: dir.clone(),
                            });
                        }
                    }
                    MouseAction::Move(x, y) => {
                        for listener in &mut self.mouse.moves {
                            let base = UiEventBase {
                                elem,
                                action: UiMoveAction::Moving,
                                pos: Point::new(mx, my),
                                pos_rel: Point::new(mx - state.rect.x(), my - state.rect.y()),
                                synthetic: false,
                            };

                            listener(UiMoveEvent::<'_> { base });
                        }
                    }
                    MouseAction::Press(b) => {
                        for listener in &mut self.mouse.clicks {
                            let base = UiEventBase {
                                elem,
                                action: UiClickAction::Click,
                                pos: Point::new(mx, my),
                                pos_rel: Point::new(mx - state.rect.x(), my - state.rect.y()),
                                synthetic: false,
                            };

                            listener(UiClickEvent::<'_> { base, button: b });
                        }
                    }
                    MouseAction::Release(b) => {
                        for listener in &mut self.mouse.clicks {
                            let base = UiEventBase {
                                elem,
                                action: UiClickAction::Release,
                                pos: Point::new(mx, my),
                                pos_rel: Point::new(mx - state.rect.x(), my - state.rect.y()),
                                synthetic: false,
                            };

                            listener(UiClickEvent::<'_> { base, button: b });
                        }
                    }
                }

                return true;
            }
        } else {
            return true;
        }

        false
    }

    pub(crate) fn keyboard_change(
        &mut self,
        action: KeyboardAction,
        elem: &mut UiElement,
        input: &Input,
    ) -> bool {
        true
    }

    pub fn clear_listener(&mut self, id: u64) {
        let idx = (id >> 32) as u32;
        let kind = (id & 0xFFFFFFFF) as u32;
        match kind {
            CLICK_LISTENER => {
                let _ = self.mouse.clicks.swap_remove(idx as usize);
            }
            SCROLL_LISTENER => {
                let _ = self.mouse.scrolls.swap_remove(idx as usize);
            }
            HOVER_LISTENER => {
                let _ = self.mouse.hovers.swap_remove(idx as usize);
            }
            MOVE_LISTENER => {
                let _ = self.mouse.moves.swap_remove(idx as usize);
            }
            GLOBAL_MOVE_LISTENER => {
                let _ = self.mouse.global_moves.swap_remove(idx as usize);
            }
            _ => unreachable!(),
        }
    }

    pub fn on_click<F>(&mut self, f: F) -> u64
    where
        F: FnMut(UiClickEvent) + 'static,
    {
        let idx = self.mouse.clicks.len() as u64;
        self.mouse.clicks.push(Box::new(f));
        ((idx << 32) as u32 | CLICK_LISTENER) as u64
    }

    pub fn on_scroll<F>(&mut self, f: F) -> u64
    where
        F: FnMut(UiScrollEvent) + 'static,
    {
        let idx = self.mouse.scrolls.len() as u64;
        self.mouse.scrolls.push(Box::new(f));
        ((idx << 32) as u32 | SCROLL_LISTENER) as u64
    }

    pub fn on_hover<F>(&mut self, f: F) -> u64
    where
        F: FnMut(UiHoverEvent) + 'static,
    {
        let idx = self.mouse.scrolls.len() as u64;
        self.mouse.hovers.push(Box::new(f));
        ((idx << 32) as u32 | HOVER_LISTENER) as u64
    }

    pub fn on_move<F>(&mut self, f: F) -> u64
    where
        F: FnMut(UiMoveEvent) + 'static,
    {
        let idx = self.mouse.scrolls.len() as u64;
        self.mouse.moves.push(Box::new(f));
        ((idx << 32) as u32 | MOVE_LISTENER) as u64
    }

    pub fn on_global_move<F>(&mut self, f: F) -> u64
    where
        F: FnMut(UiMoveEvent) + 'static,
    {
        let idx = self.mouse.scrolls.len() as u64;
        self.mouse.global_moves.push(Box::new(f));
        ((idx << 32) as u32 | GLOBAL_MOVE_LISTENER) as u64
    }
}

pub struct UiEventBase<'a, Action: Clone> {
    pub elem: &'a mut UiElement,
    pub action: Action,
    pub pos: Point<i32>,
    pub pos_rel: Point<i32>,
    pub synthetic: bool
}

pub struct UiMouseEvents {
    clicks: Vec<Box<dyn FnMut(UiClickEvent)>>,
    scrolls: Vec<Box<dyn FnMut(UiScrollEvent)>>,
    hovers: Vec<Box<dyn FnMut(UiHoverEvent)>>,
    moves: Vec<Box<dyn FnMut(UiMoveEvent)>>,
    global_moves: Vec<Box<dyn FnMut(UiMoveEvent)>>,
}

impl UiMouseEvents {
    pub fn new() -> Self {
        Self {
            clicks: vec![],
            scrolls: vec![],
            hovers: vec![],
            moves: vec![],
            global_moves: vec![],
        }
    }
}

pub struct UiClickEvent<'a> {
    pub base: UiEventBase<'a, UiClickAction>,
    pub button: MouseButton,
}

#[derive(Clone, PartialEq)]
pub enum UiClickAction {
    Click,
    Release,
}

pub struct UiScrollEvent<'a> {
    pub base: UiEventBase<'a, UiScrollAction>,
    pub direction: UiScrollDirection,
}

#[derive(Clone, PartialEq)]
pub enum UiScrollAction {
    X(f32),
    Y(f32),
    Both(f32, f32),
}
#[derive(Clone, PartialEq)]
pub enum UiScrollDirection {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

pub struct UiHoverEvent<'a> {
    pub base: UiEventBase<'a, UiHoverAction>,
}

#[derive(Clone, PartialEq)]
pub enum UiHoverAction {
    Enter,
    Leave,
}

pub struct UiMoveEvent<'a> {
    pub base: UiEventBase<'a, UiMoveAction>,
}

#[derive(Clone, PartialEq)]
pub enum UiMoveAction {
    Moving,
}
