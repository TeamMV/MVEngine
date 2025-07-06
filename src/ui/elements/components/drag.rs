use crate::input::consts::MouseButton;
use crate::input::{Input, MouseAction, RawInputEvent};
use crate::ui::geometry::SimpleRect;

#[derive(Clone, Debug)]
pub struct DragAssistant {
    button: MouseButton,
    pub target: SimpleRect,
    pub in_drag: bool,
    last: (i32, i32),
    pub offset: (i32, i32),
    pub global_offset: (i32, i32),
    /// reference point to where the target would be at 0 0
    pub reference: (i32, i32),
}

impl DragAssistant {
    pub fn new(mouse_button: MouseButton) -> Self {
        Self {
            button: mouse_button,
            target: SimpleRect::new(0, 0, 0, 0),
            in_drag: false,
            last: (0, 0),
            offset: (0, 0),
            global_offset: (0, 0),
            reference: (0, 0),
        }
    }

    pub fn on_input(&mut self, action: RawInputEvent, input: &Input) -> bool {
        match action {
            RawInputEvent::Keyboard(_) => {}
            RawInputEvent::Mouse(ma) => match ma {
                MouseAction::Wheel(_, _) => {}
                MouseAction::Move(mx, my) => {
                    if self.in_drag {
                        self.offset = (mx - self.last.0, my - self.last.1);
                    }
                }
                MouseAction::Press(b) => {
                    if b == self.button {
                        if self.target.inside(input.mouse_x, input.mouse_y) {
                            self.in_drag = true;
                            self.last = (input.mouse_x, input.mouse_y);
                        }
                    }
                }
                MouseAction::Release(b) => {
                    if b == self.button {
                        self.in_drag = false;
                        self.offset = (0, 0);
                    }
                }
            },
        }

        self.global_offset = (
            self.target.x + self.offset.0 - self.reference.0,
            self.target.y + self.offset.1 - self.reference.1,
        );

        self.in_drag
    }
}
