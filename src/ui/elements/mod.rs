pub mod child;
pub mod components;
pub mod events;
pub mod implementations;
pub mod prelude;

pub use implementations::*;
use std::fmt::Pointer;

use crate::input::consts::MouseButton;
use crate::input::{Input, MouseAction, RawInputEvent};
use crate::math::vec::Vec2;
use crate::rendering::text::Font;
use crate::rendering::{OpenGLRenderer, RenderContext, Transform};
use crate::resolve;
use crate::ui::anim::ElementAnimator;
use crate::ui::attributes::Attributes;
use crate::ui::context::UiContext;
use crate::ui::elements::button::Button;
use crate::ui::elements::child::{Child, ToChild};
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::components::drag::DragAssistant;
use crate::ui::elements::components::scroll::ScrollBars;
use crate::ui::elements::div::Div;
use crate::ui::elements::events::UiEvents;
use crate::ui::elements::text::Text;
use crate::ui::elements::textbox::TextBox;
use crate::ui::geometry::{Rect, SimpleRect};
use crate::ui::rendering::{UiRenderer, WideRenderContext};
use crate::ui::res::MVR;
use crate::ui::styles::enums::{ChildAlign, Direction, Origin, Overflow, Position};
use crate::ui::styles::types::Dimension;
use crate::ui::styles::{DEFAULT_STYLE, ResCon, UiStyle, UiStyleWriteObserver};
use crate::ui::styles::{InheritSupplier, ResolveResult};
use mvutils::unsafe_utils::{DangerousCell, Unsafe};
use mvutils::utils::PClamp;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use crate::rendering::pipeline::RenderingPipeline;
use crate::ui::elements::checkbox::CheckBox;

pub trait UiElementCallbacks {
    /// Do not call this function manually! instead, call frame_callback()
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect);

    fn raw_input(&mut self, action: RawInputEvent, input: &Input) -> bool {
        false
    }
}

pub fn create_style_obs<'a>(style: &'a mut UiStyle, state: &'a mut UiElementState) -> UiStyleWriteObserver<'a> {
    UiStyleWriteObserver::new(style, &mut state.invalid)
}

pub trait UiElementStub: UiElementCallbacks {
    fn end_frame(&mut self) {
        self.state_mut().events.after_frame();
        for child in &self.state_mut().children {
            if let Child::Element(e) = child {
                e.get_mut().end_frame();
            }
        }
    }

    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Element
    where
        Self: Sized;

    fn wrap(self) -> UiElement;

    /// Return a valid reference counter (Rc<>) to this element. Should be implemented by storing a Weak<> inside the struct and upgrading it
    fn wrapped(&self) -> Element;

    fn attributes(&self) -> &Attributes;

    fn attributes_mut(&mut self) -> &mut Attributes;

    fn state(&self) -> &UiElementState;

    fn state_mut(&mut self) -> &mut UiElementState;

    fn style(&self) -> &UiStyle;

    fn style_mut(&mut self) -> UiStyleWriteObserver;

    fn context(&self) -> &UiContext;

    /// Adds a child to the element and also sets the childs parent to self
    fn add_child(&mut self, child: Child) {
        if let Child::Element(e) = &child {
            e.get_mut().state_mut().parent = Some(self.wrapped());
        }
        if let Child::Iterator(children) = child {
            for c in children {
                self.add_child(c);
            }
        } else {
            self.state_mut().children.push(child);
        }
    }

    fn remove_child_by_id(&mut self, id: &str) {
        self.state_mut().children.retain(|c| {
            if let Child::Element(e) = c {
                let r = e.get().attributes().id.as_ref().is_some_and(|a| a == id);
                if r {
                    e.get_mut().state_mut().parent = None;
                }
                !r
            } else {
                true
            }
        });
    }

    fn remove_child_by_class(&mut self, class: &str) {
        self.state_mut().children.retain(|c| {
            if let Child::Element(e) = c {
                let r = e.get().attributes().classes.iter().any(|s| s == class);
                if r {
                    e.get_mut().state_mut().parent = None;
                }
                !r
            } else {
                true
            }
        });
    }

    fn children(&self) -> &[Child] {
        &self.state().children
    }

    fn children_mut(&mut self) -> &mut [Child] {
        &mut self.state_mut().children
    }

    fn body(&self) -> &ElementBody;

    fn body_mut(&mut self) -> &mut ElementBody;

    /// Checks whether a given point is inside the element. In retrospective, this function is basic af and idk why its on the trait lol
    fn inside(&self, x: i32, y: i32) -> bool {
        let state = self.state();
        state.rect.inside(x, y)
    }

    /// This function should be called every frame instead of draw()
    fn frame_callback(&mut self, renderer: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect) where Self: Sized + 'static {
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.ui_draw(|t| t.pause());
        }
        let this = unsafe { (self as *mut dyn UiElementStub).as_mut().unwrap() };
        this.state_mut().animator.tick(self);
        let state = self.state();
        if !state.is_valid() {
            self.compute_styles(renderer);
        }
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.ui_draw(|t| t.resume());
        }
        self.draw(renderer, crop_area);
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.ui_draw(|t| t.pause());
        }
    }

    /// Computes all the styles and sets up the state. Should be called before draw()
    fn compute_styles(&mut self, ctx: &impl WideRenderContext)
    where
        Self: Sized + 'static,
    {
        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.ui_compute(|t| t.resume());
        }

        let this = unsafe { (self as *mut dyn UiElementStub).as_mut().unwrap() };
        this.state_mut().animator.tick(self);

        {
            let mut style = self.style_mut();
            style.disable_invalidation();
            style.merge_unset(&DEFAULT_STYLE);
        }
        let style = self.style();


        let state = this.state_mut();
        state.ctx.dpi = ctx.dpi() as f32;

        state.invalid = state.invalid.saturating_sub(1);

        let maybe_parent = state.parent.clone();

        let transform_translate_x = resolve!(self, transform.translate.x)
            .unwrap_or_default_or_percentage(
                &DEFAULT_STYLE.transform.translate.x,
                maybe_parent.clone(),
                |s| s.width(),
                ctx,
            );
        let transform_translate_y = resolve!(self, transform.translate.y)
            .unwrap_or_default_or_percentage(
                &DEFAULT_STYLE.transform.translate.y,
                maybe_parent.clone(),
                |s| s.height(),
                ctx,
            );
        let transform_scale_x = resolve!(self, transform.scale.x).unwrap_or_default_or_percentage(
            &DEFAULT_STYLE.transform.scale.x,
            maybe_parent.clone(),
            |s| s.width() as f32,
            state,
        );
        let transform_scale_y = resolve!(self, transform.scale.y).unwrap_or_default_or_percentage(
            &DEFAULT_STYLE.transform.scale.y,
            maybe_parent.clone(),
            |s| s.height() as f32,
            state,
        );
        let transform_rotation = resolve!(self, transform.rotate).unwrap_or_default_or_percentage(
            &DEFAULT_STYLE.transform.rotate,
            maybe_parent.clone(),
            |s| s.rotation(),
            state,
        );
        let transform_origin =
            resolve!(self, transform.origin).unwrap_or_default(&DEFAULT_STYLE.origin);

        state.transforms.translation.width += transform_translate_x;
        state.transforms.translation.height += transform_translate_y;
        state.transforms.scale.width = transform_scale_x;
        state.transforms.scale.height = transform_scale_y;
        state.transforms.rotation += transform_rotation;
        state.transforms.origin = transform_origin;

        let padding = style
            .padding
            .get(self, |s| &s.padding, |s| s.paddings(), ctx); //t, b, l, r
        let margin = style.margin.get(self, |s| &s.margin, |s| s.margins(), ctx); //0, 1, 2, 3

        let direction = resolve!(self, direction);
        let direction = if !direction.is_set() {
            Direction::Horizontal
        } else {
            direction.unwrap()
        };

        let font = resolve!(self, text.font);
        let font = font.unwrap_or(MVR.font.default);
        let font = self.context().resources.resolve_font(font);

        let size = resolve!(self, text.size).unwrap_or_default_or_percentage(
            &DEFAULT_STYLE.text.size,
            maybe_parent.clone(),
            |s| s.height() as f32,
            state,
        );
        let kerning = resolve!(self, text.kerning).unwrap_or_default(&DEFAULT_STYLE.text.kerning);
        let stretch = resolve!(self, text.stretch).unwrap_or_default(&DEFAULT_STYLE.text.stretch);
        let skew = resolve!(self, text.skew).unwrap_or_default(&DEFAULT_STYLE.text.skew);

        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.ui_compute(|t| t.pause());
        }
        let computed_size =
            Self::compute_children_size(state, &direction, font, size, stretch, skew, kerning, ctx);

        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.ui_compute(|t| t.resume());
        }

        let width = resolve!(self, width);
        let width = if width.is_set() {
            width.unwrap()
        } else if width.is_percent() {
            state.is_width_percent = true;
            width.resolve_percent(maybe_parent.clone(), |s| s.width(), ctx)
        } else {
            computed_size.0 + padding[2] + padding[3]
        };
        let height = resolve!(self, height);
        let height = if height.is_set() {
            height.unwrap()
        } else if height.is_percent() {
            state.is_height_percent = true;
            height.resolve_percent(maybe_parent.clone(), |s| s.height(), ctx)
        } else {
            computed_size.1 + padding[0] + padding[1]
        };

        let overflow_x = resolve!(self, overflow_x).unwrap_or_default(&DEFAULT_STYLE.overflow_x);
        let overflow_y = resolve!(self, overflow_y).unwrap_or_default(&DEFAULT_STYLE.overflow_y);

        let always_x = if let Overflow::Always = overflow_x {
            true
        } else {
            false
        };
        let always_y = if let Overflow::Always = overflow_y {
            true
        } else {
            false
        };

        if let Overflow::Never = overflow_x {
        } else {
            if computed_size.0 > width || always_x {
                //content overflow
                state.scroll_x.available = true;
                state.scroll_x.whole = computed_size.0;
            } else {
                state.scroll_x.available = false;
            }
        }

        if let Overflow::Never = overflow_y {
        } else {
            if computed_size.1 > height || always_y {
                //content overflow
                state.scroll_y.available = true;
                state.scroll_y.whole = computed_size.1;
            } else {
                state.scroll_y.available = false;
            }
        }

        state.rect.set_width(width);
        state.rect.set_height(height);
        state.bounding_rect.set_width(width + margin[2] + margin[3]);
        state
            .bounding_rect
            .set_height(height + margin[0] + margin[1]);
        state
            .content_rect
            .set_width(width - padding[2] - padding[3]);
        state
            .content_rect
            .set_height(height - padding[0] - padding[1]);

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
                x.resolve_percent(maybe_parent.clone(), |s| s.width(), ctx)
            } else {
                0
            };

            let y = resolve!(self, y);
            let y = if y.is_set() {
                y.unwrap()
            } else if y.is_percent() {
                y.resolve_percent(maybe_parent.clone(), |s| s.height(), ctx)
            } else {
                0
            };

            state
                .bounding_rect
                .set_x(origin.get_actual_x(x, width, state));
            state
                .bounding_rect
                .set_y(origin.get_actual_y(y, height, state));
        }

        state.rect.set_x(state.bounding_rect.x() + margin[2]);
        state.rect.set_y(state.bounding_rect.y() + margin[1]);
        state.content_rect.set_x(state.rect.x() + padding[2]);
        state.content_rect.set_y(state.rect.y() + padding[1]);

        let child_align_x = resolve!(self, child_align_x).unwrap_or(ChildAlign::Start);
        let child_align_y = resolve!(self, child_align_y).unwrap_or(ChildAlign::Start);

        let (mut used_width, mut used_height) = (0, 0);
        for child_elem in state.children.iter().filter_map(|e| match e {
            Child::Element(c) => Some(c.clone()),
            _ => None,
        }) {
            let mut child_guard = child_elem.get_mut();
            let child_binding = unsafe { Unsafe::cast_lifetime_mut(child_guard.deref_mut()) };
            let child_style = child_guard.style();
            let child_state = child_binding.state_mut();

            child_state.transforms.translation.width += state.transforms.translation.width;
            child_state.transforms.translation.height += state.transforms.translation.height;
            child_state.transforms.rotation += state.transforms.rotation;
            child_state.transforms.scale.width = state.transforms.scale.width;
            child_state.transforms.scale.height = state.transforms.scale.height;
            child_state.transforms.origin = state.transforms.origin.clone();

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
                        let cx = match child_align_x {
                            ChildAlign::Start => state.content_rect.x(),
                            ChildAlign::End => {
                                state.content_rect.x() + state.content_rect.width()
                                    - child_state.bounding_rect.bounding.width
                            }
                            ChildAlign::Middle => {
                                state.content_rect.x() + state.content_rect.width() / 2
                                    - child_state.bounding_rect.bounding.width / 2
                            }
                            ChildAlign::OffsetStart(o) => state.content_rect.x() + o,
                            ChildAlign::OffsetEnd(o) => {
                                state.content_rect.x() + state.content_rect.width()
                                    - child_state.bounding_rect.bounding.width
                                    + o
                            }
                            ChildAlign::OffsetMiddle(o) => {
                                state.content_rect.x() + state.content_rect.width() / 2
                                    - child_state.bounding_rect.bounding.width / 2
                                    + o
                            }
                        };

                        let cy = match child_align_y {
                            ChildAlign::Start => {
                                state.content_rect.y() + state.content_rect.height()
                                    - child_state.bounding_rect.bounding.height
                                    - used_height
                            }
                            ChildAlign::End => state.content_rect.y() + used_height,
                            ChildAlign::Middle => {
                                state.content_rect.y()
                                    + state.content_rect.height() / 2
                                    + computed_size.1 / 2
                                    - used_height
                                    - child_state.bounding_rect.bounding.height
                            }
                            ChildAlign::OffsetStart(o) => {
                                state.content_rect.y() + state.content_rect.height()
                                    - child_state.bounding_rect.bounding.height
                                    - used_height
                                    + o
                            }
                            ChildAlign::OffsetEnd(o) => state.content_rect.y() + used_height + o,
                            ChildAlign::OffsetMiddle(o) => {
                                state.content_rect.y() + state.content_rect.height() / 2
                                    - computed_size.1 / 2
                                    + used_height
                                    + o
                            }
                        };

                        (
                            child_origin.get_actual_x(
                                cx,
                                child_state.bounding_rect.bounding.width,
                                child_state,
                            ),
                            child_origin.get_actual_y(
                                cy,
                                child_state.bounding_rect.bounding.height,
                                child_state,
                            ),
                        )
                    }
                    Direction::Horizontal => {
                        let cx = match child_align_x {
                            ChildAlign::Start => state.content_rect.x() + used_width,
                            ChildAlign::End => {
                                state.content_rect.x() + state.content_rect.width()
                                    - used_width
                                    - child_state.bounding_rect.bounding.width
                            }
                            ChildAlign::Middle => {
                                state.content_rect.x() + state.content_rect.width() / 2
                                    - computed_size.0 / 2
                                    + used_width
                            }
                            ChildAlign::OffsetStart(o) => state.content_rect.x() + used_width + o,
                            ChildAlign::OffsetEnd(o) => {
                                state.content_rect.x() + state.content_rect.width()
                                    - used_width
                                    - child_state.bounding_rect.bounding.width
                                    + o
                            }
                            ChildAlign::OffsetMiddle(o) => {
                                state.content_rect.x() + state.content_rect.width() / 2
                                    - computed_size.0 / 2
                                    + used_width
                                    + o
                            }
                        };
                        let cy = match child_align_y {
                            ChildAlign::Start => {
                                state.content_rect.y() + state.content_rect.height()
                                    - child_state.bounding_rect.bounding.height
                            }
                            ChildAlign::End => state.content_rect.y(),
                            ChildAlign::Middle => {
                                state.content_rect.y() + state.content_rect.height() / 2
                                    - child_state.bounding_rect.bounding.height / 2
                            }
                            ChildAlign::OffsetStart(o) => {
                                state.content_rect.y() + state.content_rect.height()
                                    - child_state.bounding_rect.bounding.height
                                    + o
                            }
                            ChildAlign::OffsetEnd(o) => state.content_rect.y() + o,
                            ChildAlign::OffsetMiddle(o) => {
                                state.content_rect.y() + state.content_rect.height() / 2
                                    - child_state.bounding_rect.bounding.height / 2
                                    + o
                            }
                        };

                        (
                            child_origin.get_actual_x(
                                cx,
                                child_state.bounding_rect.bounding.width,
                                child_state,
                            ),
                            child_origin.get_actual_y(
                                cy,
                                child_state.bounding_rect.bounding.height,
                                child_state,
                            ),
                        )
                    }
                };

                let child_padding = child_style.padding.get(
                    child_guard.deref(),
                    |s| &s.padding,
                    |s| s.paddings(),
                    ctx,
                );
                let child_margin = child_style.margin.get(
                    child_guard.deref(),
                    |s| &s.margin,
                    |s| s.margins(),
                    ctx,
                );

                let x = if state.scroll_x.available {
                    let screen_offset = (state.scroll_x.offset as f32
                        / state.content_rect.width() as f32)
                        * state.scroll_x.whole as f32;
                    x - screen_offset as i32
                } else {
                    x
                };
                let y = if state.scroll_y.available {
                    let screen_offset = (state.scroll_y.offset as f32
                        / state.content_rect.height() as f32)
                        * state.scroll_y.whole as f32;
                    y + screen_offset as i32
                } else {
                    y
                };

                child_state.bounding_rect.set_x(x);
                child_state.bounding_rect.set_y(y);
                child_state.rect.set_x(x + child_margin[2]);
                child_state.rect.set_y(y + child_margin[1]);
                child_state
                    .content_rect
                    .set_x(child_state.rect.x() + child_padding[2]);
                child_state
                    .content_rect
                    .set_y(child_state.rect.y() + child_padding[1]);

                used_width += child_state.bounding_rect.bounding.width;
                used_height += child_state.bounding_rect.bounding.height;
            }
        }

        state.inner_transforms = state.transforms.clone();
        state.transforms = UiTransformations::new();

        #[cfg(feature = "timed")] {
            crate::debug::PROFILER.ui_compute(|t| t.pause());
        }
    }

    fn compute_children_size(
        state: &mut UiElementState,
        direction: &Direction,
        font: Option<&Font>,
        font_size: f32,
        font_stretch: Dimension<f32>,
        font_skew: f32,
        font_kerning: f32,
        ctx: &impl WideRenderContext,
    ) -> (i32, i32)
    where
        Self: Sized,
    {
        let (mut w, mut h) = (0, 0);
        let font_size = font_size * font_stretch.height;
        for child in &state.children {
            match child {
                Child::String(s) => {
                    if state.requested_width.is_some() && state.requested_height.is_some() {
                        w += state.requested_width.unwrap();
                        h += state.requested_height.unwrap();
                        continue;
                    }

                    if let Some(font) = font {
                        let width = font.get_width(s, font_size);
                        let l = s.len() as f32 - 1f32;
                        let width =
                            width * font_stretch.width + font_skew * 2f32 + font_kerning * l;

                        if let Some(rw) = state.requested_width {
                            w += rw;
                        } else {
                            w += width as i32;
                        }
                        if let Some(rh) = state.requested_height {
                            h += rh;
                        } else {
                            h = h.max(font_size as i32);
                        }
                    }
                }
                Child::Element(e) => {
                    let guard = e.get_mut();
                    guard.compute_styles(ctx);
                    let bounding = &guard.state().bounding_rect;

                    let cw = guard
                        .state()
                        .requested_width
                        .unwrap_or(bounding.bounding.width);
                    let ch = guard
                        .state()
                        .requested_height
                        .unwrap_or(bounding.bounding.height);

                    // Please keep the || true, i am sure the compiler can optimise this away. but the story goes like this:
                    // I put this check in when i made this like 4 months ago and rn it breaks something and stuff still works when i remove the check
                    // But this might turn on me in the future and i want to be able to easily undo this
                    match direction {
                        Direction::Vertical => {
                            if !guard.state().is_height_percent || true {
                                h += ch;
                            }
                            if !guard.state().is_width_percent || true {
                                w = w.max(cw);
                            }
                        }
                        Direction::Horizontal => {
                            if !guard.state().is_width_percent || true {
                                w += cw;
                            }
                            if !guard.state().is_height_percent || true {
                                h = h.max(ch);
                            }
                        }
                    }
                }
                Child::State(s) => {
                    if state.requested_width.is_some() && state.requested_height.is_some() {
                        w += state.requested_width.unwrap();
                        h += state.requested_height.unwrap();
                        continue;
                    }

                    let guard = s.read();
                    let s = guard.deref();
                    if let Some(font) = font {
                        let width = font.get_width(s, font_size);
                        let l = s.len() as f32 - 1f32;
                        let width =
                            width * font_stretch.width + font_skew * 2f32 + font_kerning * l;

                        if let Some(rw) = state.requested_width {
                            w += rw;
                        } else {
                            w += width as i32;
                        }
                        if let Some(rh) = state.requested_height {
                            h += rh;
                        } else {
                            h = h.max(font_size as i32);
                        }
                    }
                }
                _ => {}
            }

            state.requested_width = None;
            state.requested_height = None;
        }
        (w, h)
    }

    fn find_element_by_id(&self, id: &str) -> Option<Rc<DangerousCell<UiElement>>> {
        for child in &self.state().children {
            if let Child::Element(e) = child {
                let guard = e.get();
                if guard
                    .attributes()
                    .id
                    .as_ref()
                    .is_some_and(|i| i.as_str() == id)
                {
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

    /// This is an internal function and should be called inside raw_input() on the implementation
    fn super_input(&mut self, action: RawInputEvent, input: &Input) -> bool {
        let mut used = false;
        let mut in_scroll = false;
        //Scroll bars

        let state = self.state();

        let bar_extent = resolve!(self, scrollbar.size).unwrap_or_default_or_percentage(
            &DEFAULT_STYLE.scrollbar.size,
            state.parent.clone(),
            |s| s.width(),
            state,
        );

        let state = self.state_mut();
        if state.scroll_x.available {
            let knob = ScrollBars::x_knob(state, bar_extent);
            let knob_w = knob.width;

            let assistant = &mut state.scroll_x.assistant;
            assistant.reference = (state.content_rect.x(), state.content_rect.y());
            if !assistant.in_drag {
                assistant.target = knob;
            }
            in_scroll |= assistant.on_input(action.clone(), input);
            if assistant.in_drag {
                state.scroll_x.offset = assistant.global_offset.0;
            }

            if let RawInputEvent::Mouse(MouseAction::Wheel(dx, _)) = action {
                if state.rect.inside(input.mouse_x, input.mouse_y) {
                    state.scroll_x.offset += dx as i32 * 5;
                    used = true;
                }
            }

            state.scroll_x.offset = state
                .scroll_x
                .offset
                .p_clamp(0, state.content_rect.width() - knob_w);
        }

        if state.scroll_y.available {
            let knob = ScrollBars::y_knob(state, bar_extent);
            let knob_h = knob.height;

            let assistant = &mut state.scroll_y.assistant;
            assistant.reference = (state.content_rect.x(), state.content_rect.y());
            if !assistant.in_drag {
                assistant.target = knob;
            }
            in_scroll |= assistant.on_input(action.clone(), input);

            let max_offset = state.content_rect.height() - knob_h;
            if assistant.in_drag {
                state.scroll_y.offset =
                    (state.content_rect.height() - knob_h) - assistant.global_offset.1;
            }

            if let RawInputEvent::Mouse(MouseAction::Wheel(_, dy)) = action {
                if state.rect.inside(input.mouse_x, input.mouse_y) {
                    state.scroll_y.offset -= dy as i32 * 5;
                    used = true;
                }
            }

            state.scroll_y.offset = state.scroll_y.offset.p_clamp(0, max_offset);
        }

        if used || in_scroll {
            state.invalidate();
        }

        used
    }
}

pub type Element = Rc<DangerousCell<UiElement>>;

#[derive(Clone)]
pub enum UiElement {
    Div(Div),
    Button(Button),
    TextBox(TextBox),
    Text(Text),
    CheckBox(CheckBox)
}

impl ToChild for UiElement {
    fn to_child(self) -> Child {
        Child::Element(Rc::new(DangerousCell::new(self)))
    }
}

macro_rules! ui_element_fn {
    ($this:ident, $fn_name:ident()) => {
        match $this {
            UiElement::Div(e) => e.$fn_name(),
            UiElement::Button(e) => e.$fn_name(),
            UiElement::TextBox(e) => e.$fn_name(),
            UiElement::Text(e) => e.$fn_name(),
            UiElement::CheckBox(e) => e.$fn_name(),
        }
    };
    ($this:ident, $fn_name:ident($($args:ident),*)) => {
        match $this {
            UiElement::Div(e) => e.$fn_name($($args),*),
            UiElement::Button(e) => e.$fn_name($($args),*),
            UiElement::TextBox(e) => e.$fn_name($($args),*),
            UiElement::Text(e) => e.$fn_name($($args),*),
            UiElement::CheckBox(e) => e.$fn_name($($args),*),
        }
    };
}

impl UiElementCallbacks for UiElement {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect) {
        ui_element_fn!(self, draw(ctx, crop_area));
    }

    fn raw_input(&mut self, action: RawInputEvent, input: &Input) -> bool {
        ui_element_fn!(self, raw_input(action, input))
    }
}

impl UiElementStub for UiElement {
    fn new(_context: UiContext, _attributes: Attributes, _style: UiStyle) -> Element
    where
        Self: Sized,
    {
        unimplemented!("To instantiate an UiElement, use the struct's constructor!")
    }

    fn wrap(self) -> UiElement {
        self
    }

    fn wrapped(&self) -> Element {
        ui_element_fn!(self, wrapped())
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

    fn style_mut(&mut self) -> UiStyleWriteObserver {
        ui_element_fn!(self, style_mut())
    }

    fn context(&self) -> &UiContext {
        ui_element_fn!(self, context())
    }

    fn body(&self) -> &ElementBody {
        ui_element_fn!(self, body())
    }

    fn body_mut(&mut self) -> &mut ElementBody {
        ui_element_fn!(self, body_mut())
    }
}

#[derive(Clone, Debug)]
pub struct UiScrollState {
    pub available: bool,
    pub offset: i32,
    pub whole: i32,
    assistant: DragAssistant,
}

impl UiScrollState {
    pub fn new() -> Self {
        Self {
            available: false,
            offset: 0,
            whole: 0,
            assistant: DragAssistant::new(MouseButton::Left),
        }
    }
}

pub struct UiElementState {
    pub invalid: u8,
    pub ctx: ResCon,
    pub parent: Option<Rc<DangerousCell<UiElement>>>,

    pub children: Vec<Child>,

    pub rect: Rect,
    pub content_rect: Rect,
    pub bounding_rect: Rect,

    pub margins: [i32; 4], //t,d,l,r
    pub paddings: [i32; 4],

    pub events: UiEvents,

    pub transforms: UiTransformations,
    pub inner_transforms: UiTransformations,

    pub(crate) is_width_percent: bool,
    pub(crate) is_height_percent: bool,

    pub scroll_x: UiScrollState,
    pub scroll_y: UiScrollState,

    pub animator: ElementAnimator,

    pub requested_width: Option<i32>,
    pub requested_height: Option<i32>,
}

#[derive(Clone)]
pub struct UiTransformations {
    pub(crate) translation: Dimension<i32>,
    pub(crate) rotation: f32,
    pub(crate) scale: Dimension<f32>,
    pub(crate) origin: Origin,
}

impl UiTransformations {
    pub fn new() -> Self {
        Self {
            translation: Dimension::new(0, 0),
            rotation: 0.0,
            scale: Dimension::new(1.0, 1.0),
            origin: Default::default(),
        }
    }

    pub fn merge_transform(&mut self, transform: Transform) {
        self.translation.width += transform.translation.x as i32;
        self.translation.height += transform.translation.y as i32;
        self.rotation += transform.rotation;
        self.scale.width = transform.scale.x;
        self.scale.height = transform.scale.y;
        if transform.origin.x != 0.0 || transform.origin.y != 0.0 {
            let ox = transform.origin.x as i32;
            let oy = transform.origin.y as i32;

            self.origin = Origin::Custom(ox, oy);
        }
    }

    pub fn as_render_transform(&self, state: &UiElementState) -> Transform {
        let ox = self
            .origin
            .get_actual_x(state.rect.x(), state.rect.width(), state);
        let oy = self
            .origin
            .get_actual_y(state.rect.y(), state.rect.height(), state);
        Transform {
            translation: Vec2::new(
                self.translation.width as f32,
                self.translation.height as f32,
            ),
            origin: Vec2::new(ox as f32, oy as f32),
            scale: Vec2::new(self.scale.width, self.scale.height),
            rotation: self.rotation,
        }
    }
}

impl UiElementState {
    pub(crate) fn new(context: UiContext) -> Self {
        Self {
            invalid: 2,
            ctx: ResCon { dpi: 0.0 },
            parent: None,
            children: vec![],
            rect: Rect::default(),
            content_rect: Rect::default(),
            bounding_rect: Rect::default(),
            margins: [0; 4],
            paddings: [0; 4],
            events: UiEvents::create(),
            transforms: UiTransformations::new(),
            inner_transforms: UiTransformations::new(),
            is_width_percent: false,
            is_height_percent: false,
            scroll_x: UiScrollState::new(),
            scroll_y: UiScrollState::new(),
            animator: ElementAnimator::new(context),
            requested_width: None,
            requested_height: None,
        }
    }

    pub(crate) const FRAMES_TO_BE_INVALID: u8 = 5;

    pub fn invalidate(&mut self) {
        self.invalid = Self::FRAMES_TO_BE_INVALID;
    }

    pub fn is_valid(&self) -> bool {
        self.invalid == 0
    }
}

impl Clone for UiElementState {
    fn clone(&self) -> Self {
        Self {
            invalid: self.invalid.clone(),
            ctx: self.ctx.clone(),
            parent: self.parent.clone(),
            children: self.children.clone(),
            rect: self.rect.clone(),
            content_rect: self.content_rect.clone(),
            bounding_rect: self.bounding_rect.clone(),
            margins: self.margins.clone(),
            paddings: self.paddings.clone(),
            events: UiEvents::create(),
            transforms: self.transforms.clone(),
            inner_transforms: self.inner_transforms.clone(),
            is_width_percent: self.is_width_percent.clone(),
            is_height_percent: self.is_height_percent.clone(),
            scroll_x: self.scroll_x.clone(),
            scroll_y: self.scroll_y.clone(),
            animator: self.animator.clone(),
            requested_width: self.requested_width.clone(),
            requested_height: self.requested_height.clone(),
        }
    }
}

impl InheritSupplier for UiElementState {
    fn x(&self) -> i32 {
        self.content_rect.x()
    }

    fn y(&self) -> i32 {
        self.content_rect.y()
    }

    fn width(&self) -> i32 {
        self.content_rect.width()
    }

    fn height(&self) -> i32 {
        self.content_rect.height()
    }

    fn paddings(&self) -> [i32; 4] {
        self.paddings
    }

    fn margins(&self) -> [i32; 4] {
        self.margins
    }

    fn rotation(&self) -> f32 {
        self.inner_transforms.rotation
    }
}