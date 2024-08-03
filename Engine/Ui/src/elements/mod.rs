pub mod child;
pub mod lmao;
mod events;

use crate::render::color::RgbColor;
use crate::render::draw2d::DrawContext2D;
use crate::resolve;
use crate::ui::attributes::Attributes;
use crate::ui::ease::Easing;
use crate::ui::elements::child::Child;
use crate::ui::styles::{
    ChildAlign, Dimension, Direction, Origin, Point, Position, ResCon, TextFit, UiStyle, UiValue,
};
use crate::ui::timing::{AnimationState, DurationTask, TIMING_MANAGER};
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use mvutils::utils::{Recover, RwArc, TetrahedronOp};
use parking_lot::RwLock;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use mvutils::once::CreateOnce;
use crate::ui::background::Background;
//use crate::ui::elements::events::UiEvents;

pub trait UiElementCallbacks {
    fn init(&mut self);

    fn draw(&mut self, ctx: &mut DrawContext2D);
}

pub trait UiElement: UiElementCallbacks {
    fn new(attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized;

    fn attributes(&self) -> &Attributes;

    fn attributes_mut(&mut self) -> &Attributes;

    fn state(&self) -> &UiElementState;

    fn state_mut(&mut self) -> &mut UiElementState;

    fn style(&self) -> &UiStyle;

    fn style_mut(&mut self) -> &mut UiStyle;

    fn components(&self) -> (&Attributes, &UiStyle, &UiElementState);

    fn components_mut(&mut self) -> (&mut Attributes, &mut UiStyle, &mut UiElementState);

    fn add_child(&mut self, child: Child) {
        self.state_mut().children.push(child);
    }

    fn children(&self) -> &[Child] {
        &self.state().children
    }

    fn children_mut(&mut self) -> &mut [Child] {
        &mut self.state_mut().children
    }

    fn get_size(&self, s: &str) -> Dimension<i32>;

    fn inside(&self, x: i32, y: i32) -> bool {
        let state = self.state();
        if x > state.x && x < state.x + state.width {
            if y > state.y && y < state.y + state.height {
                return true;
            }
        }
        false
    }
}

pub struct UiElementState {
    pub ctx: ResCon,
    pub parent: Option<Arc<RwLock<dyn UiElement>>>,

    pub(crate) children: Vec<Child>,

    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) content_x: i32,
    pub(crate) content_y: i32,
    pub(crate) bounding_x: i32,
    pub(crate) bounding_y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) content_width: i32,
    pub(crate) content_height: i32,
    pub(crate) bounding_width: i32,
    pub(crate) bounding_height: i32,
    pub(crate) margins: [i32; 4], //t,d,l,r
    pub(crate) paddings: [i32; 4],

    pub(crate) transforms: UiTransformations,

    pub background: Option<Arc<RwLock<dyn Background>>>,

    //pub events: UiEvents
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
            ctx: ResCon { dpi: 0.0 },
            parent: None,
            children: vec![],
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
            margins: [0; 4],
            paddings: [0; 4],
            transforms: UiTransformations {
                translation: Dimension::new(0, 0),
                rotation: 0.0,
                scale: Dimension::new(1.0, 1.0),
                origin: Origin::Center,
            },
            background: None,
            //events: UiEvents::create()
        }
    }

    pub fn compute(elem: Arc<RwLock<dyn UiElement>>, ctx: &mut DrawContext2D) {
        let mut guard = elem.write();
        guard.state_mut().ctx.dpi = ctx.dpi();

        let binding = unsafe { (&*guard as *const dyn UiElement).as_ref().unwrap() };
        let (_, style, state) = guard.components_mut();

        let direction = if style.direction.is_set() {
            resolve!(binding, direction)
        } else {
            Direction::default()
        };

        let origin = if style.origin.is_set() {
            resolve!(binding, origin)
        } else {
            Origin::BottomLeft
        };

        let child_align = if style.child_align.is_set() {
            resolve!(binding, child_align)
        } else {
            ChildAlign::Start
        };

        let width_auto = style.width.is_auto();
        let height_auto = style.height.is_auto();

        let mut width = if style.width.is_set() {
            resolve!(binding, width)
        } else {
            0
        };
        let mut height = if style.height.is_set() {
            resolve!(binding, height)
        } else {
            0
        };

        let mut occupied_width = 0;
        let mut occupied_height = 0;

        for child in state.children.iter_mut().rev() {
            if let Child::Element(e) = child {
                let mut guard = e.write();
                let mut stat = guard.state_mut();

                if matches!(direction, Direction::Horizontal) {
                    stat.bounding_x = occupied_width;
                } else {
                    stat.bounding_y = occupied_height - stat.margins[1];
                }

                drop(guard);

                UiElementState::compute(e.clone(), ctx);

                let mut guard = e.write();
                let mut stat = guard.state_mut();

                if matches!(direction, Direction::Horizontal) {
                    occupied_width += stat.bounding_width;
                    occupied_height = occupied_height.max(stat.bounding_height);
                } else {
                    occupied_width = occupied_width.max(stat.bounding_width);
                    occupied_height += stat.bounding_height;
                }
            } else {
                let s = child.as_string();

                let text_fit = if style.text.fit.is_set() {
                    resolve!(binding, text.fit)
                } else {
                    if matches!(style.text.fit.get_value(), UiValue::None) {
                        TextFit::ExpandParent
                    } else {
                        TextFit::CropText
                    }
                };

                if matches!(text_fit, TextFit::ExpandParent) {
                    let size = binding.get_size(&s);

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
        };

        width = style
            .width
            .get_field()
            .apply(width, binding, |s| &s.width.get_field());
        height = style
            .height
            .get_field()
            .apply(height, binding, |s| &s.height.get_field());

        let padding = style.padding.get(binding, |s| &s.padding); //t,b,l,r
        let margin = style.margin.get(binding, |s| &s.margin);

        state.margins.copy_from_slice(&margin);
        state.paddings.copy_from_slice(&padding);

        state.content_width = width;
        state.content_height = height;
        state.width = width + padding[2] + padding[3];
        state.height = height + padding[0] + padding[1];
        state.bounding_width = state.width + margin[2] + margin[3];
        state.bounding_height = state.height + margin[0] + margin[1];

        //TODO: check if bounding width/height is set correcetly

        let position = if style.position.is_set() {
            resolve!(binding, position)
        } else {
            Position::Relative
        };
        if matches!(position, Position::Absolute) {
            if !style.x.is_auto() {
                let x = if style.x.is_set() {
                    resolve!(binding, x)
                } else {
                    0
                };
                state.bounding_x = origin.get_actual_x(x, state.content_width);
            }
            if !style.y.is_auto() {
                let y = if style.y.is_set() {
                    resolve!(binding, y)
                } else {
                    0
                };
                state.bounding_y = origin.get_actual_y(y, state.content_height);
            }
        }

        state.x = state.bounding_x + margin[2];
        state.y = state.bounding_y + margin[1];
        state.content_x = state.x + padding[2];
        state.content_y = state.y + padding[1];

        let mut height_remaining = state.content_height;

        for e in state
            .children
            .iter()
            .rev()
            .filter(|c| c.is_element())
            .map(|c| match c {
                Child::Element(e) => e.clone(),
                _ => {
                    unreachable!()
                }
            })
        {
            let mut e_guard = e.write();
            let e_binding = unsafe { (&*e_guard as *const dyn UiElement).as_ref().unwrap() };
            let (_, e_style, stat) = e_guard.components_mut();
            //set xy of child to rel coords if pos=rel

            let child_position = if e_style.position.is_set() {
                resolve!(e_binding, position)
            } else {
                Position::Relative
            };
            if matches!(child_position, Position::Relative) {
                let x_off = stat.x;
                let y_off = stat.y;

                let child_origin = if e_style.origin.is_set() {
                    resolve!(e_binding, origin)
                } else {
                    Origin::BottomLeft
                };

                match direction {
                    Direction::Vertical => {
                        stat.bounding_y = child_origin.get_actual_y(y_off, stat.bounding_height)
                            + state.content_y;
                        stat.bounding_x = child_origin.get_actual_x(
                            match child_align {
                                ChildAlign::Start => state.content_x,
                                ChildAlign::End => {
                                    state.content_x + state.content_width - stat.bounding_width
                                }
                                ChildAlign::Middle => {
                                    state.content_x + state.content_width / 2
                                        - stat.bounding_width / 2
                                }
                                ChildAlign::OffsetStart(o) => state.content_x + o,
                                ChildAlign::OffsetEnd(o) => {
                                    state.content_x + state.content_width - stat.bounding_width - o
                                }
                                ChildAlign::OffsetMiddle(o) => {
                                    state.content_x + state.content_width / 2
                                        - stat.bounding_width / 2
                                        + o
                                }
                            },
                            stat.bounding_width,
                        );
                    }
                    Direction::Horizontal => {
                        stat.bounding_x =
                            child_origin.get_actual_x(x_off, stat.bounding_width) + state.content_x;
                        stat.bounding_y = child_origin.get_actual_y(
                            match child_align {
                                ChildAlign::Start => state.content_y,
                                ChildAlign::End => {
                                    state.content_y + state.content_height - stat.bounding_height
                                }
                                ChildAlign::Middle => {
                                    state.content_y + state.content_height / 2
                                        - stat.content_height / 2
                                }
                                ChildAlign::OffsetStart(o) => state.content_y + o,
                                ChildAlign::OffsetEnd(o) => {
                                    state.content_y + state.content_height
                                        - stat.bounding_height
                                        - o
                                }
                                ChildAlign::OffsetMiddle(o) => {
                                    state.content_y + state.content_height / 2
                                        - stat.bounding_height / 2
                                        + o
                                }
                            },
                            stat.bounding_height,
                        );
                    }
                }

                let e_padding = e_style.padding.get(binding, |s| &s.padding); //t,b,l,r
                let e_margin = e_style.margin.get(binding, |s| &s.margin);

                stat.bounding_y = stat.bounding_y.min(state.bounding_y + state.bounding_height - stat.bounding_height);

                stat.x = stat.bounding_x + e_margin[2];
                stat.y = stat.bounding_y + e_margin[1];
                stat.content_x = stat.x + e_padding[2];
                stat.content_y = stat.y + e_padding[1];
            }
        }
    }
}
