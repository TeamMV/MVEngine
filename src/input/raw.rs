use super::consts::*;
use winit::event::{MouseButton};
use winit::keyboard::KeyCode;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum State {
    Pressed,
    Released,
    JustPressed,
    JustReleased,
}

pub struct Input {
    ///All the keys of a full-sized keyboard and whether they are pressed or not. Access them with the constants KEY_...
    pub keys: [bool; MAX_KEYS],
    ///All the keys of a full-sized keyboard and their exact state. Access them with the constants KEY_...
    pub keystates: [State; MAX_KEYS],
    ///All the buttons of a mouse with all the extra ones and whether they are pressed or not. Access them with the constants MOUSE_...
    pub mouse: [bool; MAX_MOUSE],
    ///All the buttons of a mouse with all the extra ones and their exact state. Access them with the constants MOUSE_...
    pub mousestates: [State; MAX_MOUSE],
    ///All the scroll directions of a mouse wheel and whether they are executed at the moment. Access them with the constants MOUSE_SCROLL_...
    pub scroll: [bool; 4],
    ///All the scroll directions of a mouse wheel and their exact float value at the moment. Access them with the constants MOUSE_SCROLL_...
    pub scrollstates: [f32; 4],
    ///Both mouse x and y position, access with MOUSE_POS_X or MOUSE_POS_Y or simply 0 or 1.
    pub positions: [i32; 2],
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys: [false; MAX_KEYS],
            keystates: [State::Released; MAX_KEYS],
            mouse: [false; MAX_MOUSE],
            mousestates: [State::Released; MAX_MOUSE],
            scroll: [false; 4],
            scrollstates: [0.0; 4],
            positions: [0, 0],
        }
    }

    pub(crate) fn loop_states(&mut self) {
        for i in 0..MAX_KEYS {
            if self.keystates[i] == State::JustPressed {
                self.keystates[i] = State::Pressed
            }
            if self.keystates[i] == State::JustReleased {
                self.keystates[i] = State::Released;
                self.keys[i] = false;
            }
        }
        for i in 0..MAX_MOUSE {
            if self.mousestates[i] == State::JustPressed {
                self.mousestates[i] = State::Pressed
            }
            if self.mousestates[i] == State::JustReleased {
                self.mousestates[i] = State::Released;
                self.mouse[i] = false;
            }
        }
        for i in 0..4 {
            self.scroll[i] = false;
            self.scrollstates[i] = 0.0;
        }
    }

    pub fn key_from_str(s: &str) -> usize {
        match s.to_lowercase().as_str() {
            "escape" => KEY_ESCAPE,
            "f1" => KEY_F1,
            "f2" => KEY_F2,
            "f3" => KEY_F3,
            "f4" => KEY_F4,
            "f5" => KEY_F5,
            "f6" => KEY_F6,
            "f7" => KEY_F7,
            "f8" => KEY_F8,
            "f9" => KEY_F9,
            "f10" => KEY_F10,
            "f11" => KEY_F11,
            "f12" => KEY_F12,
            "print_screen" => KEY_PRINT_SCREEN,
            "scroll_lock" => KEY_SCROLL_LOCK,
            "pause" => KEY_PAUSE,
            "grave_accent" => KEY_GRAVE_ACCENT,
            "1" => KEY_1,
            "2" => KEY_2,
            "3" => KEY_3,
            "4" => KEY_4,
            "5" => KEY_5,
            "6" => KEY_6,
            "7" => KEY_7,
            "8" => KEY_8,
            "9" => KEY_9,
            "0" => KEY_0,
            "minus" => KEY_MINUS,
            "equals" => KEY_EQUALS,
            "backspace" => KEY_BACKSPACE,
            "tab" => KEY_TAB,
            "q" => KEY_Q,
            "w" => KEY_W,
            "e" => KEY_E,
            "r" => KEY_R,
            "t" => KEY_T,
            "y" => KEY_Y,
            "u" => KEY_U,
            "i" => KEY_I,
            "o" => KEY_O,
            "p" => KEY_P,
            "left_bracket" => KEY_LEFT_BRACKET,
            "right_bracket" => KEY_RIGHT_BRACKET,
            "backslash" => KEY_BACKSLASH,
            "caps_lock" => KEY_CAPS_LOCK,
            "a" => KEY_A,
            "s" => KEY_S,
            "d" => KEY_D,
            "f" => KEY_F,
            "g" => KEY_G,
            "h" => KEY_H,
            "j" => KEY_J,
            "k" => KEY_K,
            "l" => KEY_L,
            "semicolon" => KEY_SEMICOLON,
            "apostrophe" => KEY_APOSTROPHE,
            "enter" => KEY_ENTER,
            "left_shift" => KEY_LEFT_SHIFT,
            "z" => KEY_Z,
            "x" => KEY_X,
            "c" => KEY_C,
            "v" => KEY_V,
            "b" => KEY_B,
            "n" => KEY_N,
            "m" => KEY_M,
            "comma" => KEY_COMMA,
            "period" => KEY_PERIOD,
            "slash" => KEY_SLASH,
            "right_shift" => KEY_RIGHT_SHIFT,
            "left_ctrl" => KEY_LEFT_CTRL,
            "left_alt" => KEY_LEFT_ALT,
            "space" => KEY_SPACE,
            "right_alt" => KEY_RIGHT_ALT,
            "right_ctrl" => KEY_RIGHT_CTRL,
            "left_arrow" => KEY_LEFT_ARROW,
            "up_arrow" => KEY_UP_ARROW,
            "down_arrow" => KEY_DOWN_ARROW,
            "right_arrow" => KEY_RIGHT_ARROW,
            "insert" => KEY_INSERT,
            "delete" => KEY_DELETE,
            "home" => KEY_HOME,
            "end" => KEY_END,
            "page_up" => KEY_PAGE_UP,
            "page_down" => KEY_PAGE_DOWN,
            "num_lock" => KEY_NUM_LOCK,
            "kp_divide" => KEY_KP_DIVIDE,
            "kp_multiply" => KEY_KP_MULTIPLY,
            "kp_minus" => KEY_KP_MINUS,
            "kp_plus" => KEY_KP_PLUS,
            "kp_enter" => KEY_KP_ENTER,
            "kp_1" => KEY_KP_1,
            "kp_2" => KEY_KP_2,
            "kp_3" => KEY_KP_3,
            "kp_4" => KEY_KP_4,
            "kp_5" => KEY_KP_5,
            "kp_6" => KEY_KP_6,
            "kp_7" => KEY_KP_7,
            "kp_8" => KEY_KP_8,
            "kp_9" => KEY_KP_9,
            "kp_0" => KEY_KP_0,
            "kp_period" => KEY_KP_PERIOD,
            "non_us_backslash" => KEY_NON_US_BACKSLASH,
            "application" => KEY_APPLICATION,
            "power" => KEY_POWER,
            _ => usize::MAX,
        }
    }

    pub fn string_from_key(key: usize) -> String {
        match key {
            KEY_ESCAPE => "ESCAPE".to_string(),
            KEY_F1 => "F1".to_string(),
            KEY_F2 => "F2".to_string(),
            KEY_F3 => "F3".to_string(),
            KEY_F4 => "F4".to_string(),
            KEY_F5 => "F5".to_string(),
            KEY_F6 => "F6".to_string(),
            KEY_F7 => "F7".to_string(),
            KEY_F8 => "F8".to_string(),
            KEY_F9 => "F9".to_string(),
            KEY_F10 => "F10".to_string(),
            KEY_F11 => "F11".to_string(),
            KEY_F12 => "F12".to_string(),
            KEY_PRINT_SCREEN => "PRINT_SCREEN".to_string(),
            KEY_SCROLL_LOCK => "SCROLL_LOCK".to_string(),
            KEY_PAUSE => "PAUSE".to_string(),
            KEY_GRAVE_ACCENT => "GRAVE_ACCENT".to_string(),
            KEY_1 => "1".to_string(),
            KEY_2 => "2".to_string(),
            KEY_3 => "3".to_string(),
            KEY_4 => "4".to_string(),
            KEY_5 => "5".to_string(),
            KEY_6 => "6".to_string(),
            KEY_7 => "7".to_string(),
            KEY_8 => "8".to_string(),
            KEY_9 => "9".to_string(),
            KEY_0 => "0".to_string(),
            KEY_MINUS => "MINUS".to_string(),
            KEY_EQUALS => "EQUALS".to_string(),
            KEY_BACKSPACE => "BACKSPACE".to_string(),
            KEY_TAB => "TAB".to_string(),
            KEY_Q => "Q".to_string(),
            KEY_W => "W".to_string(),
            KEY_E => "E".to_string(),
            KEY_R => "R".to_string(),
            KEY_T => "T".to_string(),
            KEY_Y => "Y".to_string(),
            KEY_U => "U".to_string(),
            KEY_I => "I".to_string(),
            KEY_O => "O".to_string(),
            KEY_P => "P".to_string(),
            KEY_LEFT_BRACKET => "LEFT_BRACKET".to_string(),
            KEY_RIGHT_BRACKET => "RIGHT_BRACKET".to_string(),
            KEY_BACKSLASH => "BACKSLASH".to_string(),
            KEY_CAPS_LOCK => "CAPS_LOCK".to_string(),
            KEY_A => "A".to_string(),
            KEY_S => "S".to_string(),
            KEY_D => "D".to_string(),
            KEY_F => "F".to_string(),
            KEY_G => "G".to_string(),
            KEY_H => "H".to_string(),
            KEY_J => "J".to_string(),
            KEY_K => "K".to_string(),
            KEY_L => "L".to_string(),
            KEY_SEMICOLON => "SEMICOLON".to_string(),
            KEY_APOSTROPHE => "APOSTROPHE".to_string(),
            KEY_ENTER => "ENTER".to_string(),
            KEY_LEFT_SHIFT => "LEFT_SHIFT".to_string(),
            KEY_Z => "Z".to_string(),
            KEY_X => "X".to_string(),
            KEY_C => "C".to_string(),
            KEY_V => "V".to_string(),
            KEY_B => "B".to_string(),
            KEY_N => "N".to_string(),
            KEY_M => "M".to_string(),
            KEY_COMMA => "COMMA".to_string(),
            KEY_PERIOD => "PERIOD".to_string(),
            KEY_SLASH => "SLASH".to_string(),
            KEY_RIGHT_SHIFT => "RIGHT_SHIFT".to_string(),
            KEY_LEFT_CTRL => "LEFT_CTRL".to_string(),
            KEY_LEFT_ALT => "LEFT_ALT".to_string(),
            KEY_SPACE => "SPACE".to_string(),
            KEY_RIGHT_ALT => "RIGHT_ALT".to_string(),
            KEY_RIGHT_CTRL => "RIGHT_CTRL".to_string(),
            KEY_LEFT_ARROW => "LEFT_ARROW".to_string(),
            KEY_UP_ARROW => "UP_ARROW".to_string(),
            KEY_DOWN_ARROW => "DOWN_ARROW".to_string(),
            KEY_RIGHT_ARROW => "RIGHT_ARROW".to_string(),
            KEY_INSERT => "INSERT".to_string(),
            KEY_DELETE => "DELETE".to_string(),
            KEY_HOME => "HOME".to_string(),
            KEY_END => "END".to_string(),
            KEY_PAGE_UP => "PAGE_UP".to_string(),
            KEY_PAGE_DOWN => "PAGE_DOWN".to_string(),
            KEY_NUM_LOCK => "NUM_LOCK".to_string(),
            KEY_KP_DIVIDE => "KP_DIVIDE".to_string(),
            KEY_KP_MULTIPLY => "KP_MULTIPLY".to_string(),
            KEY_KP_MINUS => "KP_MINUS".to_string(),
            KEY_KP_PLUS => "KP_PLUS".to_string(),
            KEY_KP_ENTER => "KP_ENTER".to_string(),
            KEY_KP_1 => "KP_1".to_string(),
            KEY_KP_2 => "KP_2".to_string(),
            KEY_KP_3 => "KP_3".to_string(),
            KEY_KP_4 => "KP_4".to_string(),
            KEY_KP_5 => "KP_5".to_string(),
            KEY_KP_6 => "KP_6".to_string(),
            KEY_KP_7 => "KP_7".to_string(),
            KEY_KP_8 => "KP_8".to_string(),
            KEY_KP_9 => "KP_9".to_string(),
            KEY_KP_0 => "KP_0".to_string(),
            KEY_KP_PERIOD => "KP_PERIOD".to_string(),
            KEY_NON_US_BACKSLASH => "NON_US_BACKSLASH".to_string(),
            KEY_APPLICATION => "APPLICATION".to_string(),
            KEY_POWER => "POWER".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    pub fn string_from_mouse(mouse: usize) -> String {
        match mouse {
            MOUSE_LEFT => "MOUSE_LEFT".to_string(),
            MOUSE_RIGHT => "MOUSE_RIGHT".to_string(),
            MOUSE_MIDDLE => "MOUSE_MIDDLE".to_string(),
            MOUSE_3 => "MOUSE_4".to_string(),
            MOUSE_4 => "MOUSE_5".to_string(),
            MOUSE_5 => "MOUSE_6".to_string(),
            MOUSE_6 => "MOUSE_7".to_string(),
            MOUSE_7 => "MOUSE_8".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    pub fn mouse_from_string(s: &str) -> usize {
        match s.to_lowercase().as_str() {
            "mouse_left" => MOUSE_LEFT,
            "mouse_right" => MOUSE_RIGHT,
            "mouse_middle" => MOUSE_MIDDLE,
            "mouse_4" => MOUSE_3,
            "mouse_5" => MOUSE_4,
            "mouse_6" => MOUSE_5,
            "mouse_7" => MOUSE_6,
            "mouse_8" => MOUSE_7,
            _ => usize::MAX,
        }
    }

    pub fn string_from_scroll(scroll: usize) -> String {
        match scroll {
            MOUSE_SCROLL_UP => "MOUSE_SCROLL_UP".to_string(),
            MOUSE_SCROLL_DOWN => "MOUSE_SCROLL_DOWN".to_string(),
            MOUSE_SCROLL_LEFT => "MOUSE_SCROLL_LEFT".to_string(),
            MOUSE_SCROLL_RIGHT => "MOUSE_SCROLL_RIGHT".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    pub fn scroll_from_string(s: &str) -> usize {
        match s.to_lowercase().as_str() {
            "mouse_scroll_up" => MOUSE_SCROLL_UP,
            "mouse_scroll_down" => MOUSE_SCROLL_DOWN,
            "mouse_scroll_left" => MOUSE_SCROLL_LEFT,
            "mouse_scroll_right" => MOUSE_SCROLL_RIGHT,
            _ => usize::MAX,
        }
    }

    pub fn string_from_position(pos: usize) -> String {
        match pos {
            MOUSE_POS_X => "MOUSE_POS_X".to_string(),
            MOUSE_POS_Y => "MOUSE_POS_Y".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    pub fn position_from_string(s: &str) -> usize {
        match s.to_lowercase().as_str() {
            "mouse_pos_x" => MOUSE_POS_X,
            "mouse_pos_y" => MOUSE_POS_Y,
            _ => usize::MAX,
        }
    }

    pub(crate) fn key_from_winit(key_code: KeyCode) -> usize {
        match key_code {
            KeyCode::Escape => KEY_ESCAPE,
            KeyCode::F1 => KEY_F1,
            KeyCode::F2 => KEY_F2,
            KeyCode::F3 => KEY_F3,
            KeyCode::F4 => KEY_F4,
            KeyCode::F5 => KEY_F5,
            KeyCode::F6 => KEY_F6,
            KeyCode::F7 => KEY_F7,
            KeyCode::F8 => KEY_F8,
            KeyCode::F9 => KEY_F9,
            KeyCode::F10 => KEY_F10,
            KeyCode::F11 => KEY_F11,
            KeyCode::F12 => KEY_F12,
            KeyCode::PrintScreen => KEY_PRINT_SCREEN,
            KeyCode::ScrollLock => KEY_SCROLL_LOCK,
            KeyCode::Pause => KEY_PAUSE,
            KeyCode::Backquote => KEY_GRAVE_ACCENT,
            KeyCode::Digit1 => KEY_1,
            KeyCode::Digit2 => KEY_2,
            KeyCode::Digit3 => KEY_3,
            KeyCode::Digit4 => KEY_4,
            KeyCode::Digit5 => KEY_5,
            KeyCode::Digit6 => KEY_6,
            KeyCode::Digit7 => KEY_7,
            KeyCode::Digit8 => KEY_8,
            KeyCode::Digit9 => KEY_9,
            KeyCode::Digit0 => KEY_0,
            KeyCode::Minus => KEY_MINUS,
            KeyCode::Equal => KEY_EQUALS,
            KeyCode::Backspace => KEY_BACKSPACE,
            KeyCode::Tab => KEY_TAB,
            KeyCode::KeyQ => KEY_Q,
            KeyCode::KeyW => KEY_W,
            KeyCode::KeyE => KEY_E,
            KeyCode::KeyR => KEY_R,
            KeyCode::KeyT => KEY_T,
            KeyCode::KeyY => KEY_Y,
            KeyCode::KeyU => KEY_U,
            KeyCode::KeyI => KEY_I,
            KeyCode::KeyO => KEY_O,
            KeyCode::KeyP => KEY_P,
            KeyCode::BracketLeft => KEY_LEFT_BRACKET,
            KeyCode::BracketRight => KEY_RIGHT_BRACKET,
            KeyCode::Backslash => KEY_BACKSLASH,
            KeyCode::CapsLock => KEY_CAPS_LOCK,
            KeyCode::KeyA => KEY_A,
            KeyCode::KeyS => KEY_S,
            KeyCode::KeyD => KEY_D,
            KeyCode::KeyF => KEY_F,
            KeyCode::KeyG => KEY_G,
            KeyCode::KeyH => KEY_H,
            KeyCode::KeyJ => KEY_J,
            KeyCode::KeyK => KEY_K,
            KeyCode::KeyL => KEY_L,
            KeyCode::Semicolon => KEY_SEMICOLON,
            KeyCode::Quote => KEY_APOSTROPHE,
            KeyCode::Enter => KEY_ENTER,
            KeyCode::ShiftLeft => KEY_LEFT_SHIFT,
            KeyCode::KeyZ => KEY_Z,
            KeyCode::KeyX => KEY_X,
            KeyCode::KeyC => KEY_C,
            KeyCode::KeyV => KEY_V,
            KeyCode::KeyB => KEY_B,
            KeyCode::KeyN => KEY_N,
            KeyCode::KeyM => KEY_M,
            KeyCode::Comma => KEY_COMMA,
            KeyCode::Period => KEY_PERIOD,
            KeyCode::Slash => KEY_SLASH,
            KeyCode::ShiftRight => KEY_RIGHT_SHIFT,
            KeyCode::ControlLeft => KEY_LEFT_CTRL,
            KeyCode::AltLeft => KEY_LEFT_ALT,
            KeyCode::Space => KEY_SPACE,
            KeyCode::AltRight => KEY_RIGHT_ALT,
            KeyCode::ControlRight => KEY_RIGHT_CTRL,
            KeyCode::ArrowLeft => KEY_LEFT_ARROW,
            KeyCode::ArrowUp => KEY_UP_ARROW,
            KeyCode::ArrowDown => KEY_DOWN_ARROW,
            KeyCode::ArrowRight => KEY_RIGHT_ARROW,
            KeyCode::Insert => KEY_INSERT,
            KeyCode::Delete => KEY_DELETE,
            KeyCode::Home => KEY_HOME,
            KeyCode::End => KEY_END,
            KeyCode::PageUp => KEY_PAGE_UP,
            KeyCode::PageDown => KEY_PAGE_DOWN,
            KeyCode::NumLock => KEY_NUM_LOCK,
            KeyCode::NumpadDivide => KEY_KP_DIVIDE,
            KeyCode::NumpadMultiply => KEY_KP_MULTIPLY,
            KeyCode::NumpadSubtract => KEY_KP_MINUS,
            KeyCode::NumpadAdd => KEY_KP_PLUS,
            KeyCode::NumpadEnter => KEY_KP_ENTER,
            KeyCode::Numpad1 => KEY_KP_1,
            KeyCode::Numpad2 => KEY_KP_2,
            KeyCode::Numpad3 => KEY_KP_3,
            KeyCode::Numpad4 => KEY_KP_4,
            KeyCode::Numpad5 => KEY_KP_5,
            KeyCode::Numpad6 => KEY_KP_6,
            KeyCode::Numpad7 => KEY_KP_7,
            KeyCode::Numpad8 => KEY_KP_8,
            KeyCode::Numpad9 => KEY_KP_9,
            KeyCode::Numpad0 => KEY_KP_0,
            KeyCode::NumpadDecimal => KEY_KP_PERIOD,
            KeyCode::Meta => KEY_APPLICATION,
            KeyCode::Power => KEY_POWER,
            _ => usize::MAX,
        }
    }

    pub(crate) fn mouse_from_winit(mouse_button: MouseButton) -> usize {
        match mouse_button {
            MouseButton::Left => MOUSE_LEFT,
            MouseButton::Right => MOUSE_RIGHT,
            MouseButton::Middle => MOUSE_MIDDLE,
            MouseButton::Other(2) => MOUSE_3,
            MouseButton::Other(3) => MOUSE_4,
            MouseButton::Other(4) => MOUSE_5,
            MouseButton::Other(5) => MOUSE_6,
            MouseButton::Other(6) => MOUSE_7,
            _ => usize::MAX,
        }
    }
}
