pub mod child;
pub mod lmao;

use crate::render::color::RgbColor;
use crate::render::draw2d::DrawContext2D;
use crate::resolve;
use crate::ui::attributes::Attributes;
use crate::ui::elements::child::Child;
use crate::ui::styles::{
    ChildAlign, Dimension, Direction, Origin, Point, Position, ResCon, TextFit, UiStyle, UiValue,
};
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::{Recover, RwArc, TetrahedronOp};
use std::sync::{Arc, RwLock};

pub trait UiElementCallbacks {
    fn init(&mut self);

    fn draw(&mut self, ctx: &mut DrawContext2D);
}

pub trait UiElement: UiElementCallbacks {
    fn new(attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized;

    fn state(&self) -> &UiElementState;

    fn state_mut(&mut self) -> &mut UiElementState;

    fn style(&self) -> &UiStyle;

    fn style_mut(&mut self) -> &mut UiStyle;

    fn add_child(&mut self, child: Child);

    fn children(&self) -> impl Iterator<Item = &Child>;

    fn children_mut(&mut self) -> impl Iterator<Item = &mut Child>;

    fn get_size(&self, s: &str) -> Dimension<i32>;
}

pub(crate) struct UiElementState {
    x: i32,
    y: i32,
    content_x: i32,
    content_y: i32,
    bounding_x: i32,
    bounding_y: i32,
    width: i32,
    height: i32,
    content_width: i32,
    content_height: i32,
    bounding_width: i32,
    bounding_height: i32,

    transforms: UiTransformations,
}

#[derive(Clone)]
pub(crate) struct UiTransformations {
    translation: Dimension<i32>,
    rotation: f32,
    scale: Dimension<f32>,
    origin: Origin,
}

impl UiElementState {
    pub(crate) fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            content_x: 0,
            content_y: 0,
            bounding_x: 0,
            bounding_y: 0,
            width: 0,
            height: 0,
            content_width: 0,
            content_height: 0,
            bounding_width: 0,
            bounding_height: 0,
            transforms: UiTransformations {
                translation: Dimension::new(0, 0),
                rotation: 0.0,
                scale: Dimension::new(1.0, 1.0),
                origin: Origin::Center,
            },
        }
    }

    pub fn compute(&mut self, elem: &mut impl UiElement) {
        let style = elem.style();

        let direction = if style.direction.is_set() {
            resolve!(elem, direction)
        } else {
            Direction::default()
        };

        let origin = if style.origin.is_set() {
            resolve!(elem, origin)
        } else {
            Origin::BottomLeft
        };

        let child_align = if style.child_align.is_set() {
            resolve!(elem, child_align)
        } else {
            ChildAlign::Start
        };

        let width_auto = matches!(style.width, UiValue::Auto);
        let height_auto = matches!(style.height, UiValue::Auto);

        let mut width = if style.width.is_set() {
            resolve!(elem, width)
        } else {
            0
        };
        let mut height = if style.height.is_set() {
            resolve!(elem, height)
        } else {
            0
        };

        if width_auto || height_auto {
            let mut occupied_width = 0;
            let mut occupied_height = 0;

            for child in elem.children_mut() {
                if let Child::Element(mut e) = child {
                    let stat = e.state_mut();

                    if matches!(direction, Direction::Horizontal) {
                        stat.x = occupied_width;
                    } else {
                        stat.y = occupied_height;
                    }

                    stat.compute(&mut e);

                    if matches!(direction, Direction::Horizontal) {
                        occupied_width += e.state().bounding_width;
                        occupied_height = occupied_height.max(e.state().bounding_height);
                    } else {
                        occupied_width = occupied_width.max(e.state().bounding_width);
                        occupied_height += e.state().bounding_height;
                    }
                } else {
                    let s = child.as_string();

                    let text_fit = if style.text.fit.is_set() {
                        resolve!(elem, text.fit)
                    } else {
                        if matches!(style.text.fit, UiValue::None) {
                            TextFit::ExpandParent
                        } else {
                            TextFit::CropText
                        }
                    };

                    if matches!(text_fit, TextFit::ExpandParent) {
                        let size = elem.get_size(&s);
                        if matches!(direction, Direction::Horizontal) {
                            occupied_width += size.width;
                            occupied_height = occupied_height.max(size.height);
                        } else {
                            occupied_width = occupied_width.max(size.width);
                            occupied_height += size.height;
                        }
                    }
                }
            }

            if width_auto && occupied_width > width {
                width = occupied_width;
            }

            if height_auto && occupied_height > height {
                height = occupied_height;
            }
        }

        let padding = style.padding.get(elem); //t,b,l,r
        let margin = style.margin.get(elem);

        self.content_width = width;
        self.content_height = height;
        self.width = width + padding[2] + padding[3];
        self.height = height + padding[0] + padding[1];
        self.bounding_width = self.width + margin[2] + margin[3];
        self.bounding_height = self.height + margin[0] + margin[1];

        let position = if style.position.is_set() {
            resolve!(elem, position)
        } else {
            Position::Relative
        };
        if matches!(position, Position::Absolute) {
            if !matches!(style.x, UiValue::Auto) {
                let x = if style.x.is_set() {
                    resolve!(elem, x)
                } else {
                    0
                };
                self.content_x = origin.get_actual_x(x, self.content_width);
            }
            if !matches!(style.y, UiValue::Auto) {
                let y = if style.y.is_set() {
                    resolve!(elem, y)
                } else {
                    0
                };
                self.content_y = origin.get_actual_y(y, self.content_height);
            }
        }

        self.x = self.content_x - padding[2];
        self.y = self.content_y - padding[1];
        self.bounding_x = self.x - margin[2];
        self.bounding_y = self.y - margin[1];

        for e in elem
            .children_mut()
            .filter(Child::is_element)
            .map(|c| match c {
                Child::Element(ref mut e) => e,
                _ => {
                    unreachable!()
                }
            })
        {
            //set xy of child to rel coords if pos=rel

            let child_position = if e.style().position.is_set() {
                resolve!(e, position)
            } else {
                Position::Relative
            };
            if matches!(child_position, Position::Relative) {
                let stat = e.state_mut();
                let x_off = stat.x;
                let y_off = stat.y;

                let child_origin = if e.style().origin.is_set() {
                    resolve!(e, origin)
                } else {
                    Origin::BottomLeft
                };

                match direction {
                    Direction::Vertical => {
                        stat.content_y = child_origin.get_actual_y(y_off, stat.content_height);
                        stat.content_x = child_origin.get_actual_x(
                            match child_align {
                                ChildAlign::Start => self.content_x,
                                ChildAlign::End => {
                                    self.content_x + self.content_width - stat.content_width
                                }
                                ChildAlign::Middle => {
                                    self.content_x + self.content_width / 2 - stat.content_width / 2
                                }
                                ChildAlign::OffsetStart(o) => self.content_x + o,
                                ChildAlign::OffsetEnd(o) => {
                                    self.content_x + self.content_width - stat.content_width - o
                                }
                                ChildAlign::OffsetMiddle(o) => {
                                    self.content_x + self.content_width / 2 - stat.content_width / 2
                                        + o
                                }
                            },
                            stat.content_width,
                        );
                    }
                    Direction::Horizontal => {
                        stat.content_x = child_origin.get_actual_x(x_off, stat.content_width);
                        stat.content_y = child_origin.get_actual_y(
                            match child_align {
                                ChildAlign::Start => self.content_y,
                                ChildAlign::End => {
                                    self.content_y + self.content_height - stat.content_height
                                }
                                ChildAlign::Middle => {
                                    self.content_y + self.content_height / 2
                                        - stat.content_height / 2
                                }
                                ChildAlign::OffsetStart(o) => self.content_y + o,
                                ChildAlign::OffsetEnd(o) => {
                                    self.content_y + self.content_height - stat.content_height - o
                                }
                                ChildAlign::OffsetMiddle(o) => {
                                    self.content_y + self.content_height / 2
                                        - stat.content_height / 2
                                        + o
                                }
                            },
                            stat.content_height,
                        );
                    }
                }

                let padding = e.style().padding.get(elem); //t,b,l,r
                let margin = e.style().margin.get(elem);

                stat.x = stat.content_x - padding[2];
                stat.y = stat.content_y - padding[1];
                stat.bounding_x = stat.x - margin[2];
                stat.bounding_y = stat.y - margin[1];
            }
        }
    }
}
