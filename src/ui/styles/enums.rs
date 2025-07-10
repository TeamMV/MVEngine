use crate::ui::elements::UiElementState;
use crate::ui::parse::parse_num;
use crate::ui::res::MVR;
use mvutils::{Savable, TryFromString};
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Default, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, TryFromString)]
pub enum Origin {
    TopLeft,
    #[default]
    BottomLeft,
    TopRight,
    BottomRight,
    Center,
    #[exclude]
    Custom(i32, i32),
    #[exclude]
    Eval(fn(i32, i32, i32, i32, &UiElementState) -> (i32, i32)),
}

impl Origin {
    pub fn is_right(&self) -> bool {
        matches!(self, Origin::BottomRight | Origin::TopRight)
    }

    pub fn is_left(&self) -> bool {
        *self == Origin::BottomLeft || *self == Origin::TopLeft
    }

    pub fn get_custom(&self) -> Option<(i32, i32)> {
        if let Origin::Custom(x, y) = self {
            Some((*x, *y))
        } else {
            None
        }
    }

    pub fn get_actual_x(&self, x: i32, width: i32, state: &UiElementState) -> i32 {
        match self {
            Origin::TopLeft => x,
            Origin::BottomLeft => x,
            Origin::TopRight => x - width,
            Origin::BottomRight => x - width,
            Origin::Center => x + width / 2,
            Origin::Custom(cx, _) => x + cx,
            Origin::Eval(f) => {
                let res = f(
                    state.bounding_rect.x(),
                    state.bounding_rect.y(),
                    state.bounding_rect.width(),
                    state.bounding_rect.height(),
                    state,
                );
                x - res.0
            }
        }
    }

    pub fn get_actual_y(&self, y: i32, height: i32, state: &UiElementState) -> i32 {
        match self {
            Origin::TopLeft => y - height,
            Origin::BottomLeft => y,
            Origin::TopRight => y - height,
            Origin::BottomRight => y,
            Origin::Center => y + height / 2,
            Origin::Custom(_, cy) => y - cy,
            Origin::Eval(f) => {
                let res = f(
                    state.bounding_rect.x(),
                    state.bounding_rect.y(),
                    state.bounding_rect.width(),
                    state.bounding_rect.height(),
                    state,
                );
                y - res.1
            }
        }
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromString)]
pub enum Position {
    Absolute,
    #[default]
    Relative,
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromString)]
pub enum Direction {
    Vertical,
    #[default]
    Horizontal,
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, TryFromString)]
pub enum TextFit {
    ExpandParent,
    #[default]
    CropText,
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, TryFromString)]
pub enum TextAlign {
    Start,
    #[default]
    Middle,
    End,
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromString)]
pub enum Overflow {
    Always,
    Never,
    #[default]
    Normal,
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum ChildAlign {
    #[default]
    Start,
    End,
    Middle,
    OffsetStart(i32),
    OffsetEnd(i32),
    OffsetMiddle(i32),
}

impl FromStr for ChildAlign {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut name = String::new();
        let mut num = String::new();
        let mut in_num = false;
        let mut done = false;
        for c in value.chars() {
            if done {
                continue;
            }
            if in_num {
                if c == ')' {
                    done = true;
                } else {
                    num.push(c);
                }
            } else {
                if c.is_alphabetic() || c == '_' {
                    name.push(c);
                } else {
                    if c == '(' {
                        in_num = true;
                    }
                }
            }
        }

        match name.as_str() {
            "start" => Ok(ChildAlign::Start),
            "end" => Ok(ChildAlign::End),
            "middle" => Ok(ChildAlign::Middle),
            "start_offset" => Ok(ChildAlign::OffsetStart(parse_num::<i32, ParseIntError>(
                num.as_str(),
            )?)),
            "end_offset" => Ok(ChildAlign::OffsetEnd(parse_num::<i32, ParseIntError>(
                num.as_str(),
            )?)),
            "middle_offset" => Ok(ChildAlign::OffsetMiddle(parse_num::<i32, ParseIntError>(
                num.as_str(),
            )?)),
            _ => Err(format!("ChildAlign '{name}' unknown!")),
        }
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromString)]
pub enum BackgroundRes {
    #[default]
    Color,
    Texture,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Savable)]
pub enum Geometry {
    Shape(usize),
    Adaptive(usize),
}

impl Default for Geometry {
    fn default() -> Self {
        Self::Shape(MVR.shape.rect)
    }
}

impl FromStr for Geometry {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut name = String::new();
        let mut num = String::new();
        let mut in_num = false;
        let mut done = false;
        for c in value.chars() {
            if done {
                continue;
            }
            if in_num {
                if c == ')' {
                    done = true;
                } else {
                    num.push(c);
                }
            } else {
                if c.is_alphabetic() || c == '_' {
                    name.push(c);
                } else {
                    if c == '(' {
                        in_num = true;
                    }
                }
            }
        }

        match name.as_str() {
            "shape" => Ok(Geometry::Shape(parse_num::<usize, ParseIntError>(
                num.as_str(),
            )?)),
            "adaptive" => Ok(Geometry::Adaptive(parse_num::<usize, ParseIntError>(
                num.as_str(),
            )?)),
            _ => Err(format!("ChildAlign '{name}' unknown!")),
        }
    }
}
