use crate::input::consts::MouseButton;
use crate::input::{Input, KeyboardAction, MouseAction};
use crate::ui::elements::child::Child;
use crate::ui::elements::{UiElement, UiElementStub};
use crate::ui::styles::types::Point;
use crate::window::Window;
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::TetrahedronOp;

pub struct UiEvents {
    last_inside: bool,
    pub click_event: Option<UiClickEvent>,
    pub hover_event: Option<UiHoverEvent>,
    pub scroll_event: Option<UiScrollEvent>,
    pub move_event: Option<UiMoveEvent>,
    pub global_move_event: Option<UiMoveEvent>,
}

impl UiEvents {
    pub fn create() -> Self {
        Self {
            last_inside: false,
            click_event: None,
            hover_event: None,
            scroll_event: None,
            move_event: None,
            global_move_event: None,
        }
    }

    pub fn after_frame(&mut self) {
        self.click_event = None;
        self.hover_event = None;
        self.scroll_event = None;
        self.move_event = None;
        self.global_move_event = None;
    }

    pub(crate) fn mouse_change(
        &mut self,
        action: MouseAction,
        elem: &mut UiElement,
        input: &Input,
        window: &mut Window,
    ) -> bool {
        let state = elem.state();
        let state = unsafe { Unsafe::cast_static(state) };

        let mut used = false;
        for child in &state.children {
            match child {
                Child::Element(e) => unsafe {
                    let child_guard = e.get_mut();
                    let child_events = &mut child_guard.state_mut().events;
                    let child_events: &mut UiEvents = Unsafe::cast_mut_static(child_events);
                    let res =
                        child_events.mouse_change(action.clone(), &mut *child_guard, input, window);
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
                let base = UiEventBase {
                    action: UiMoveAction::Moving,
                    pos: Point::new(max, may),
                    pos_rel: Point::new(max - state.rect.x(), may - state.rect.y()),
                    synthetic: false,
                };
                self.global_move_event = Some(UiMoveEvent { base });

                if self.last_inside {
                    if !elem.inside(max, may) {
                        self.last_inside = false;

                        let base = UiEventBase {
                            action: UiHoverAction::Leave,
                            pos: Point::new(max, may),
                            pos_rel: Point::new(max - state.rect.x(), may - state.rect.y()),
                            synthetic: false,
                        };

                        self.hover_event = Some(UiHoverEvent { base });
                    }
                } else {
                    if elem.inside(max, may) {
                        self.last_inside = true;

                        let base = UiEventBase {
                            action: UiHoverAction::Enter,
                            pos: Point::new(max, may),
                            pos_rel: Point::new(max - state.rect.x(), may - state.rect.y()),
                            synthetic: false,
                        };

                        self.hover_event = Some(UiHoverEvent { base });
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
                        let base = UiEventBase {
                            action: action.clone(),
                            pos: Point::new(mx, my),
                            pos_rel: Point::new(mx - state.rect.x(), my - state.rect.y()),
                            synthetic: false,
                        };

                        self.scroll_event = Some(UiScrollEvent {
                            base,
                            direction: dir.clone(),
                        });
                    }
                    MouseAction::Move(_x, _y) => {
                        let base = UiEventBase {
                            action: UiMoveAction::Moving,
                            pos: Point::new(mx, my),
                            pos_rel: Point::new(mx - state.rect.x(), my - state.rect.y()),
                            synthetic: false,
                        };

                        self.move_event = Some(UiMoveEvent { base });
                    }
                    MouseAction::Press(b) => {
                        let base = UiEventBase {
                            action: UiClickAction::Click,
                            pos: Point::new(mx, my),
                            pos_rel: Point::new(mx - state.rect.x(), my - state.rect.y()),
                            synthetic: false,
                        };

                        self.click_event = Some(UiClickEvent { base, button: b });
                    }
                    MouseAction::Release(b) => {
                        let base = UiEventBase {
                            action: UiClickAction::Release,
                            pos: Point::new(mx, my),
                            pos_rel: Point::new(mx - state.rect.x(), my - state.rect.y()),
                            synthetic: false,
                        };

                        self.click_event = Some(UiClickEvent { base, button: b });
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
        _: KeyboardAction,
        _: &mut UiElement,
        _: &Input,
    ) -> bool {
        //There are no keyboard events on ui elements yet, so this makes sense that its not used. they will be tho in the near future
        true
    }
}

pub struct UiEventBase<Action: Clone> {
    pub action: Action,
    pub pos: Point<i32>,
    pub pos_rel: Point<i32>,
    pub synthetic: bool,
}

pub struct UiClickEvent {
    pub base: UiEventBase<UiClickAction>,
    pub button: MouseButton,
}

#[derive(Clone, PartialEq)]
pub enum UiClickAction {
    Click,
    Release,
}

pub struct UiScrollEvent {
    pub base: UiEventBase<UiScrollAction>,
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

pub struct UiHoverEvent {
    pub base: UiEventBase<UiHoverAction>,
}

#[derive(Clone, PartialEq)]
pub enum UiHoverAction {
    Enter,
    Leave,
}

pub struct UiMoveEvent {
    pub base: UiEventBase<UiMoveAction>,
}

#[derive(Clone, PartialEq)]
pub enum UiMoveAction {
    Moving,
}
