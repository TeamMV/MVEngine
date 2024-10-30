pub mod blank;
pub mod child;
pub mod events;
pub mod implementations;
pub mod lmao;

pub use implementations::*;

use crate::attributes::Attributes;
use crate::ease::Easing;
use crate::elements::blank::Blank;
use crate::elements::button::Button;
use crate::elements::child::Child;
use crate::elements::div::Div;
use crate::elements::events::UiEvents;
use crate::elements::lmao::LmaoElement;
use crate::resolve;
use crate::styles::{
    ChildAlign, Dimension, Direction, Interpolator, Origin, Point, Position, ResCon, Resolve,
    TextFit, UiStyle, UiValue,
};
use crate::timing::{AnimationState, DurationTask, TIMING_MANAGER};
use crate::uix::{DynamicUi, UiCompoundElement};
use mve2d::renderer2d::GameRenderer2D;
use mvutils::once::CreateOnce;
use mvutils::unsafe_utils::{DangerousCell, Unsafe, UnsafeRef};
use mvutils::utils::{Recover, RwArc, TetrahedronOp};
use parking_lot::RwLock;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use mvutils::ordefault::OrDefault;
use mvcore::input;
use mvcore::input::MouseAction;
use mvcore::input::raw::Input;
use crate::graphics::Rect;
//use crate::elements::events::UiEvents;

pub trait UiElementCallbacks {
    fn draw(&mut self, renderer: &mut GameRenderer2D);
}

pub trait UiElementStub: UiElementCallbacks {
    fn new(attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized;

    fn wrap(self) -> UiElement;

    fn attributes(&self) -> &Attributes;

    fn attributes_mut(&mut self) -> &mut Attributes;

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
        state.rect.inside(x, y)
    }
}

pub enum UiElement {
    Blank(Blank),
    Lmao(LmaoElement),
    Div(Div),
    Button(Button),
}

macro_rules! ui_element_fn {
    ($this:ident, $fn_name:ident()) => {
        match $this {
            UiElement::Blank(e) => e.$fn_name(),
            UiElement::Lmao(e) => e.$fn_name(),
            UiElement::Div(e) => e.$fn_name(),
            _ => todo!(),
        }
    };
    ($this:ident, $fn_name:ident($($args:ident),*)) => {
        match $this {
            UiElement::Blank(e) => e.$fn_name($($args),*),
            UiElement::Lmao(e) => e.$fn_name($($args),*),
            UiElement::Div(e) => e.$fn_name($($args),*),
            _ => todo!(),
        }
    };
}

impl UiElementCallbacks for UiElement {
    fn draw(&mut self, renderer: &mut GameRenderer2D) {
        ui_element_fn!(self, draw(renderer))
    }
}

impl UiElementStub for UiElement {
    fn new(attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized,
    {
        unimplemented!("To instantiate an UiElement, use the struct's constructor!")
    }

    fn wrap(self) -> UiElement {
        self
    }

    fn attributes(&self) -> &Attributes {
        ui_element_fn!(self, attributes())
    }

    fn attributes_mut(&mut self) -> &mut Attributes {
        ui_element_fn!(self, attributes_mut())
    }

    fn state(&self) -> &UiElementState {
        ui_element_fn!(self, state())
    }

    fn state_mut(&mut self) -> &mut UiElementState {
        ui_element_fn!(self, state_mut())
    }

    fn style(&self) -> &UiStyle {
        ui_element_fn!(self, style())
    }

    fn style_mut(&mut self) -> &mut UiStyle {
        ui_element_fn!(self, style_mut())
    }

    fn components(&self) -> (&Attributes, &UiStyle, &UiElementState) {
        ui_element_fn!(self, components())
    }

    fn components_mut(&mut self) -> (&mut Attributes, &mut UiStyle, &mut UiElementState) {
        ui_element_fn!(self, components_mut())
    }

    fn get_size(&self, s: &str) -> Dimension<i32> {
        ui_element_fn!(self, get_size(s))
    }
}

pub struct UiElementState {
    pub ctx: ResCon,
    pub parent: Option<Arc<RwLock<UiElement>>>,

    pub children: Vec<Child>,

    pub rect: Rect,
    pub content_rect: Rect,
    pub bounding_rect: Rect,

    pub margins: [i32; 4], //t,d,l,r
    pub paddings: [i32; 4],

    pub events: UiEvents,
    pub is_animating: bool,
    pub last_animation: u64,
    pub last_style: Option<UiStyle>,

    pub transforms: UiTransformations,

    pub(crate) base_style: UiStyle
}

#[derive(Clone)]
pub(crate) struct UiTransformations {
    pub(crate) translation: Dimension<i32>,
    pub(crate) rotation: f32,
    pub(crate) scale: Dimension<f32>,
    pub(crate) origin: Origin,
}

impl UiElementState {
    pub(crate) fn new() -> Self {
        Self {
            ctx: ResCon { dpi: 0.0 },
            parent: None,
            children: vec![],
            rect: Rect::default(),
            content_rect: Rect::default(),
            bounding_rect: Rect::default(),
            margins: [0; 4],
            paddings: [0; 4],
            events: UiEvents::create(),
            is_animating: false,
            last_animation: 0,
            last_style: None,
            transforms: UiTransformations {
                translation: Dimension::new(0, 0),
                rotation: 0.0,
                scale: Dimension::new(0.0, 0.0),
                origin: Default::default(),
            },
            base_style: crate::styles::EMPTY_STYLE.clone(),
        }
    }

    pub fn compute(elem: Arc<RwLock<UiElement>>, renderer: &mut GameRenderer2D, input: &Input) {
        let mut guard = elem.write();
        guard.state_mut().ctx.dpi = 20.0; //TODO: get dpi from renderer

        let binding = unsafe { (&mut *guard as *mut UiElement).as_mut().unwrap() };
        let (_, style, state) = guard.components_mut();

        state.events.mouse_change(MouseAction::Move(input.positions[0], input.positions[1]), binding, input);

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

                stat.base_style.merge_unset(&state.base_style);

                if matches!(direction, Direction::Horizontal) {
                    stat.bounding_rect.x = occupied_width;
                } else {
                    stat.bounding_rect.y = occupied_height - stat.margins[1];
                }

                drop(guard);

                UiElementState::compute(e.clone(), renderer, input);

                let mut guard = e.write();
                let mut stat = guard.state_mut();

                if matches!(direction, Direction::Horizontal) {
                    occupied_width += stat.bounding_rect.width;
                    occupied_height = occupied_height.max(stat.bounding_rect.height);
                } else {
                    occupied_width = occupied_width.max(stat.bounding_rect.width);
                    occupied_height += stat.bounding_rect.height;
                }
            } else {
                let s = child.as_string();

                let text_fit = if style.text.fit.is_set() {
                    resolve!(binding, text.fit)
                } else {
                    if style.text.fit.is_auto() {
                        TextFit::CropText
                    } else {
                        TextFit::ExpandParent
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

        width = match &style.width {
            Resolve::UiValue(val) => val
                .resolve(state.ctx.dpi, state.parent.clone(), |s| {
                    &s.width.get_value()
                })
                .unwrap_or(width),
            Resolve::LayoutField(lay) => lay.apply(width, binding, |s| &s.width.get_field()),
        };

        height = match &style.height {
            Resolve::UiValue(val) => val
                .resolve(state.ctx.dpi, state.parent.clone(), |s| {
                    &s.height.get_value()
                })
                .unwrap_or(height),
            Resolve::LayoutField(lay) => lay.apply(height, binding, |s| &s.height.get_field()),
        };

        let (scale_x, scale_y) = style.transform.scale.resolve_with_default(
            state.ctx.dpi,
            state.parent.clone(),
            |s| &s.transform.scale,
            (1.0, 1.0),
        );

        let non_trans_width = width;
        let non_trans_height = height;

        width = (width as f32 * scale_x) as i32;
        height = (height as f32 * scale_y) as i32;

        let padding = style.padding.get(binding, |s| &s.padding); //t,b,l,r
        let margin = style.margin.get(binding, |s| &s.margin);

        state.margins.copy_from_slice(&margin);
        state.paddings.copy_from_slice(&padding);

        state.content_rect.width = width;
        state.content_rect.height = height;
        state.rect.width = width + padding[2] + padding[3];
        state.rect.height = height + padding[0] + padding[1];
        state.bounding_rect.width = state.rect.width + margin[2] + margin[3];
        state.bounding_rect.height = state.rect.height + margin[0] + margin[1];

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
                state.bounding_rect.x = origin.get_actual_x(x, state.content_rect.width, state);
            }
            if !style.y.is_auto() {
                let y = if style.y.is_set() {
                    resolve!(binding, y)
                } else {
                    0
                };
                state.bounding_rect.y = origin.get_actual_y(y, state.content_rect.height, state);
            }
        }

        let transform_origin = resolve!(binding, transform.origin);
        match transform_origin {
            Origin::TopLeft => {
                println!("{scale_y}");
                state.bounding_rect.y -= state.bounding_rect.height;
            }
            Origin::BottomLeft => { /*Nothing cuz already right scaling*/ }
            Origin::TopRight => {
                state.bounding_rect.x -= state.bounding_rect.width;
                state.bounding_rect.y -= state.bounding_rect.height;
            }
            Origin::BottomRight => state.bounding_rect.x -= state.bounding_rect.width,
            Origin::Center => {
                state.bounding_rect.x -= (state.bounding_rect.width as f32 * 0.5) as i32;
                state.bounding_rect.y -= (state.bounding_rect.height as f32 * 0.5) as i32;
            }
            Origin::Custom(cx, cy) => {
                //TODO: test this chatgpt code
                let dx = cx - state.bounding_rect.x;
                let dy = cy - state.bounding_rect.y;

                state.bounding_rect.x -= (dx as f32 * (scale_x - 1.0)) as i32;
                state.bounding_rect.y -= (dy as f32 * (scale_y - 1.0)) as i32;
            }
            Origin::Eval(f) => {
                let res = f(
                    state.bounding_rect.x,
                    state.bounding_rect.y,
                    non_trans_width + margin[2] + margin[3],
                    non_trans_height + margin[0] + margin[1],
                );

                let dx = res.0 - state.bounding_rect.x;
                let dy = res.1 - state.bounding_rect.y;

                state.bounding_rect.x -= (dx as f32 * (scale_x - 1.0)) as i32;
                state.bounding_rect.y -= (dy as f32 * (scale_y - 1.0)) as i32;
            }
        }

        let (trans_x, trans_y) = style.transform.translate.resolve_with_default(
            state.ctx.dpi,
            state.parent.clone(),
            |s| &s.transform.translate,
            (0, 0),
        );
        state.bounding_rect.x += trans_x;
        state.bounding_rect.y += trans_y;

        state.rect.x = state.bounding_rect.x + margin[2];
        state.rect.y = state.bounding_rect.y + margin[1];
        state.content_rect.x = state.rect.x + padding[2];
        state.content_rect.y = state.rect.y + padding[1];

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
            let e_binding = unsafe { (&*e_guard as *const dyn UiElementStub).as_ref().unwrap() };
            let (_, e_style, stat) = e_guard.components_mut();
            //set xy of child to rel coords if pos=rel

            let child_position = if e_style.position.is_set() {
                resolve!(e_binding, position)
            } else {
                Position::Relative
            };
            if matches!(child_position, Position::Relative) {
                let x_off = stat.rect.x;
                let y_off = stat.rect.y;

                let child_origin = if e_style.origin.is_set() {
                    resolve!(e_binding, origin)
                } else {
                    Origin::BottomLeft
                };

                match direction {
                    Direction::Vertical => {
                        stat.bounding_rect.y =
                            child_origin.get_actual_y(y_off, stat.bounding_rect.height, state)
                                + state.content_rect.y;
                        stat.bounding_rect.x = child_origin.get_actual_x(
                            match child_align {
                                ChildAlign::Start => state.content_rect.x,
                                ChildAlign::End => {
                                    state.content_rect.x + state.content_rect.width - stat.bounding_rect.width
                                }
                                ChildAlign::Middle => {
                                    state.content_rect.x + state.content_rect.width / 2
                                        - stat.bounding_rect.width / 2
                                }
                                ChildAlign::OffsetStart(o) => state.content_rect.x + o,
                                ChildAlign::OffsetEnd(o) => {
                                    state.content_rect.x + state.content_rect.width - stat.bounding_rect.width - o
                                }
                                ChildAlign::OffsetMiddle(o) => {
                                    state.content_rect.x + state.content_rect.width / 2
                                        - stat.bounding_rect.width / 2
                                        + o
                                }
                            },
                            stat.bounding_rect.width,
                            state,
                        );
                    }
                    Direction::Horizontal => {
                        stat.bounding_rect.x =
                            child_origin.get_actual_x(x_off, stat.bounding_rect.width, state)
                                + state.content_rect.x;
                        stat.bounding_rect.y = child_origin.get_actual_y(
                            match child_align {
                                ChildAlign::Start => state.content_rect.y,
                                ChildAlign::End => {
                                    state.content_rect.y + state.content_rect.height - stat.bounding_rect.height
                                }
                                ChildAlign::Middle => {
                                    state.content_rect.y + state.content_rect.height / 2
                                        - stat.content_rect.height / 2
                                }
                                ChildAlign::OffsetStart(o) => state.content_rect.y + o,
                                ChildAlign::OffsetEnd(o) => {
                                    state.content_rect.y + state.content_rect.height
                                        - stat.bounding_rect.height
                                        - o
                                }
                                ChildAlign::OffsetMiddle(o) => {
                                    state.content_rect.y + state.content_rect.height / 2
                                        - stat.bounding_rect.height / 2
                                        + o
                                }
                            },
                            stat.bounding_rect.height,
                            state,
                        );
                    }
                }

                let e_padding = e_style.padding.get(binding, |s| &s.padding); //t,b,l,r
                let e_margin = e_style.margin.get(binding, |s| &s.margin);

                stat.bounding_rect.y = stat
                    .bounding_rect.y
                    .min(state.bounding_rect.y + state.bounding_rect.height - stat.bounding_rect.height);

                stat.rect.x = stat.bounding_rect.x + e_margin[2];
                stat.rect.y = stat.bounding_rect.y + e_margin[1];
                stat.content_rect.x = stat.rect.x + e_padding[2];
                stat.content_rect.y = stat.rect.y + e_padding[1];
            }
        }
    }
}
