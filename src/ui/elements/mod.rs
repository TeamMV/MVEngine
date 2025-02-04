pub mod blank;
pub mod child;
pub mod events;
pub mod implementations;
pub mod components;

pub use implementations::*;

use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::elements::blank::Blank;
use crate::ui::elements::button::Button;
use crate::ui::elements::child::{Child, ToChild};
use crate::ui::elements::div::Div;
use crate::ui::elements::events::UiEvents;
use crate::ui::geometry::Rect;
use crate::ui::styles::{ChildAlign, Dimension, Direction, Interpolator, Origin, Position, ResCon, UiStyle, DEFAULT_STYLE};
use crate::ui::uix::UiCompoundElement;
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use mvutils::utils::{Recover, TetrahedronOp};
use parking_lot::RwLock;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;
use itertools::Itertools;
use crate::resolve;
use crate::ui::rendering::ctx::DrawContext2D;
use crate::ui::styles::ResolveResult;

pub trait UiElementCallbacks {
    fn draw(&mut self, ctx: &mut DrawContext2D);
}

pub trait UiElementStub: UiElementCallbacks {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Self
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

    fn compute_styles(&mut self) where Self: Sized {
        let this = unsafe { (self as *mut dyn UiElementStub).as_mut().unwrap() };
        let (_, style, state) = this.components_mut();
        state.ctx.dpi = 20.0; //TODO: change to renderer dpi

        let mut style = style.clone();
        style.merge_unset(&DEFAULT_STYLE);


        //TODO: Implement ResolveResult::Percent here and sidestyle and everywhere by using its compute_percent() method
        let padding = style.padding.get(self, |s| &s.padding, |s| &s.paddings); //t, b, l, r
        let margin = style.margin.get(self, |s| &s.margin, |s| &s.margins);    //0, 1, 2, 3

        let direction = resolve!(self, direction);
        let direction = if !direction.is_set() {
            Direction::Horizontal
        } else {
            direction.unwrap()
        };

        let maybe_parent = state.parent.clone();

        let mut computed_size = Self::compute_children_size(state, &direction);

        let width = resolve!(self, width);
        let width = if width.is_set() {
            width.unwrap()
        } else if width.is_percent() {
            state.is_width_percent = true;
            width.resolve_percent(maybe_parent.clone(), |s| s.rect.width())
        } else {
            computed_size.0 + padding[2] + padding[3]
        };
        let height = resolve!(self, height);
        let height = if height.is_set() {
            height.unwrap()
        } else if height.is_percent() {
            state.is_height_percent = true;
            height.resolve_percent(maybe_parent.clone(), |s| s.rect.height())
        } else {
            computed_size.1 + padding[0] + padding[1]
        };

        state.rect.set_width(width);
        state.rect.set_height(height);
        state.bounding_rect.set_width(width + margin[2] + margin[3]);
        state.bounding_rect.set_height(height + margin[0] + margin[1]);
        state.content_rect.set_width(width - padding[2] - padding[3]);
        state.content_rect.set_height(height - padding[0] - padding[1]);

        let position = resolve!(self, position);
        let position = if !position.is_set() {
            Position::Relative
        } else {
            position.unwrap()
        };

        let origin = resolve!(self, origin);
        let origin = if !origin.is_set() {
            Origin::BottomLeft
        } else {
            origin.unwrap()
        };

        if let Position::Absolute = position {
            let x = resolve!(self, x);
            let x = if x.is_set() {
                x.unwrap()
            } else if x.is_percent() {
                x.resolve_percent(maybe_parent.clone(), |s| s.rect.width())
            } else {
                0
            };

            let y = resolve!(self, y);
            let y = if !y.is_set() {
                y.unwrap()
            } else if y.is_percent() {
                y.resolve_percent(maybe_parent.clone(), |s| s.rect.height())
            } else {
                0
            };

            state.bounding_rect.set_x(origin.get_actual_x(x, width, state));
            state.bounding_rect.set_y(origin.get_actual_y(y, height, state));
        }

        state.rect.set_x(state.bounding_rect.x() + margin[2]);
        state.rect.set_y(state.bounding_rect.y() + margin[1]);
        state.content_rect.set_x(state.rect.x() + padding[2]);
        state.content_rect.set_y(state.rect.y() + padding[1]);

        let child_align = resolve!(self, child_align);
        let child_align = if !child_align.is_set() {
            ChildAlign::Start
        } else {
            child_align.unwrap()
        };

        let (mut used_width, mut used_height) = (0, 0);
        for child_elem in state.children.iter().filter_map(|e| {
            match e {
                Child::Element(c) => Some(c.clone()),
                _ => None
            }
        }) {
            let mut child_guard = child_elem.get_mut();
            let child_binding = unsafe { Unsafe::cast_mut_static(child_guard.deref_mut()) };
            let (_, child_style, child_state) = child_binding.components_mut();

            let child_pos = resolve!(child_guard, position);
            let child_pos = if !child_pos.is_set() {
                Position::Relative
            } else {
                child_pos.unwrap()
            };
            let child_origin = resolve!(child_guard, origin);
            let child_origin = if !child_origin.is_set() {
                Origin::BottomLeft
            } else {
                child_origin.unwrap()
            };

            if let Position::Relative = child_pos {
                let (x, y) = match direction {
                    Direction::Vertical => {
                        let cx = match child_align {
                            ChildAlign::Start => { state.content_rect.x() }
                            ChildAlign::End => { state.content_rect.x() + state.content_rect.width() - child_state.bounding_rect.bounding.width }
                            ChildAlign::Middle => { state.content_rect.x() + state.content_rect.width() / 2 - child_state.bounding_rect.bounding.width / 2 }
                            ChildAlign::OffsetStart(o) => { state.content_rect.x() + o }
                            ChildAlign::OffsetEnd(o) => { state.content_rect.x() + state.content_rect.width() - child_state.bounding_rect.bounding.width + o }
                            ChildAlign::OffsetMiddle(o) => { state.content_rect.x() + state.content_rect.width() / 2 - child_state.bounding_rect.bounding.width / 2 + o }
                        };

                        let cy = state.content_rect.y() + state.content_rect.height() - child_state.bounding_rect.bounding.height - used_height;

                        (
                            child_origin.get_actual_x(cx, child_state.bounding_rect.bounding.width, child_state),
                            child_origin.get_actual_y(cy, child_state.bounding_rect.bounding.height, child_state)
                        )
                    }
                    Direction::Horizontal => {
                        let cx = state.content_rect.x() + used_width;
                        let cy = match child_align {
                            ChildAlign::Start => { state.content_rect.y() }
                            ChildAlign::End => { state.content_rect.y() + state.content_rect.height() - child_state.bounding_rect.bounding.height }
                            ChildAlign::Middle => { state.content_rect.y() + state.content_rect.height() / 2 - child_state.bounding_rect.bounding.height / 2 }
                            ChildAlign::OffsetStart(o) => { state.content_rect.y() + o }
                            ChildAlign::OffsetEnd(o) => { state.content_rect.y() + state.content_rect.height() - child_state.bounding_rect.bounding.height + o }
                            ChildAlign::OffsetMiddle(o) => { state.content_rect.y() + state.content_rect.height() / 2 - child_state.bounding_rect.bounding.height / 2 + o }
                        };

                        (
                            child_origin.get_actual_x(cx, child_state.bounding_rect.bounding.width, child_state),
                            child_origin.get_actual_y(cy, child_state.bounding_rect.bounding.height, child_state)
                        )
                    }
                };

                let child_padding = child_style.padding.get(child_guard.deref(), |s| &s.padding, |s| &s.paddings);
                let child_margin = child_style.margin.get(child_guard.deref(), |s| &s.margin, |s| &s.margins);

                child_state.bounding_rect.set_x(x);
                child_state.bounding_rect.set_y(y);
                child_state.rect.set_x(x + child_padding[2]);
                child_state.rect.set_y(y + child_padding[1]);
                child_state.content_rect.set_x(child_state.rect.x() + child_margin[2]);
                child_state.content_rect.set_y(child_state.rect.y() + child_margin[1]);

                used_width += child_state.bounding_rect.bounding.width;
                used_height += child_state.bounding_rect.bounding.height;
            }
        }
    }

    fn compute_children_size(state: &UiElementState, direction: &Direction) -> (i32, i32) where Self: Sized {
        let (mut w, mut h) = (0, 0);
        for child in &state.children {
            match child {
                Child::String(_) => {}
                Child::Element(e) => {
                    let mut guard = e.get_mut();
                    guard.compute_styles();
                    let bounding = &guard.state().bounding_rect;
                    match direction {
                        Direction::Vertical => {
                            if !guard.state().is_height_percent {
                                h += bounding.bounding.height;
                            }
                            if !guard.state().is_width_percent {
                                w = w.max(bounding.bounding.width);
                            }
                        }
                        Direction::Horizontal => {
                            if !guard.state().is_width_percent {
                                w += bounding.bounding.width;
                            }
                            if !guard.state().is_height_percent {
                                h = h.max(bounding.bounding.height);
                            }
                        }
                    }
                }
                Child::State(_) => {}
            }
        }
        (w, h)
    }

    fn find_element_by_id(&self, id: &str) -> Option<Rc<DangerousCell<UiElement>>> {
        for child in &self.state().children {
            if let Child::Element(e) = child {
                let guard = e.get();
                if guard.attributes().id.as_ref().is_some_and(|i| i.as_str() == id) {
                    drop(guard);
                    return Some(e.clone());
                }
                if let Some(e2) = guard.find_element_by_id(id) {
                    return Some(e2);
                }
            }
        }

        None
    }

    fn find_elements_by_class(&self, class: &str) -> Vec<Rc<DangerousCell<UiElement>>> {
        let mut res = vec![];

        for child in &self.state().children {
            if let Child::Element(e) = child {
                let guard = e.get();
                if guard.attributes().classes.contains(&class.to_string()) {
                    res.push(e.clone())
                }
                let res2 = guard.find_elements_by_class(class);
                res.extend(res2);
            }
        }

        res
    }
}

#[derive(Clone)]
pub enum UiElement {
    Blank(Blank),
    Div(Div),
    Button(Button),
}

impl ToChild for UiElement {
    fn to_child(self) -> Child {
        Child::Element(Rc::new(DangerousCell::new(self)))
    }
}

macro_rules! ui_element_fn {
    ($this:ident, $fn_name:ident()) => {
        match $this {
            UiElement::Blank(e) => e.$fn_name(),
            UiElement::Div(e) => e.$fn_name(),
            UiElement::Button(e) => e.$fn_name(),
            _ => todo!(),
        }
    };
    ($this:ident, $fn_name:ident($($args:ident),*)) => {
        match $this {
            UiElement::Blank(e) => e.$fn_name($($args),*),
            UiElement::Div(e) => e.$fn_name($($args),*),
            UiElement::Button(e) => e.$fn_name($($args),*),
            _ => todo!(),
        }
    };
}

impl UiElementCallbacks for UiElement {
    fn draw(&mut self, ctx: &mut DrawContext2D) {
        ui_element_fn!(self, draw(ctx))
    }
}

impl UiElementStub for UiElement {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Self
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
    pub parent: Option<Rc<DangerousCell<UiElement>>>,

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

    pub(crate) base_style: UiStyle,
    pub(crate) is_width_percent: bool,
    pub(crate) is_height_percent: bool,
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
            base_style: crate::ui::styles::EMPTY_STYLE.clone(),
            is_width_percent: false,
            is_height_percent: false,
        }
    }
}

impl Clone for UiElementState {
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx.clone(),
            parent: self.parent.clone(),
            children: self.children.clone(),
            rect: self.rect.clone(),
            content_rect: self.content_rect.clone(),
            bounding_rect: self.bounding_rect.clone(),
            margins: self.margins.clone(),
            paddings: self.paddings.clone(),
            events: UiEvents::create(),
            is_animating: self.is_animating,
            last_animation: self.last_animation,
            last_style: self.last_style.clone(),
            transforms: self.transforms.clone(),
            base_style: self.base_style.clone(),
            is_width_percent: self.is_width_percent.clone(),
            is_height_percent: self.is_height_percent.clone(),
        }
    }
}