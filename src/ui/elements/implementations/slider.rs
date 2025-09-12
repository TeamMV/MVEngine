use crate::input::consts::MouseButton;
use crate::input::{Input, RawInputEvent};
use crate::rendering::pipeline::RenderingPipeline;
use crate::rendering::OpenGLRenderer;
use crate::ui::attributes::{Attributes, ToRope};
use crate::ui::context::UiContext;
use crate::ui::elements::components::drag::DragAssistant;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{create_style_obs, Element, UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use crate::ui::geometry::{geom, SimpleRect};
use crate::ui::styles::{UiStyle, UiStyleWriteObserver, DEFAULT_STYLE};
use mvutils::unsafe_utils::DangerousCell;
use std::rc::{Rc, Weak};
use std::str::FromStr;
use mvutils::state::State;
use mvutils::utils::PClamp;
use crate::resolve2;
use crate::ui::elements::components::boring::BoringText;
use crate::ui::elements::components::text::TextBody;
use crate::ui::geometry::shape::utils;
use crate::ui::styles::enums::Direction;

#[derive(Clone)]
pub struct Slider {
    weak: Weak<DangerousCell<UiElement>>,
    drag_assistant: DragAssistant,
    enumeration: SliderEnumeration,
    attributes: Attributes,
    state: UiElementState,
    style: UiStyle,
    body: ElementBody,
    text: BoringText,
    context: UiContext,
    scroll_offset: i32,
    value: State<f32>,
    dragged: bool
}

impl UiElementStub for Slider {
    fn new(context: UiContext, attributes: Attributes, style: UiStyle) -> Element
    where
        Self: Sized
    {
        let enumeration = if let Some(val) = attributes.attribs.get("range") {
            let s = val.as_rope().to_string();
            SliderEnumeration::from_str(&s).unwrap() //the errmsg is already in there
        } else {
            SliderEnumeration::Range(SliderRange::from_range(0.0, 10.0))
        };

        let value = if let Some(val) = attributes.attribs.get("value") {
            val.as_float_state()
        } else {
            State::new(enumeration.first())
        };

        Rc::new_cyclic(|weak| {
            let this = Self {
                weak: weak.clone(),
                drag_assistant: DragAssistant::new(MouseButton::Left),
                enumeration,
                attributes,
                state: UiElementState::new(context.clone()),
                style,
                body: ElementBody::new(),
                text: BoringText,
                context,
                scroll_offset: 0,
                value,
                dragged: false,
            };
            DangerousCell::new(this.wrap())
        })
    }

    fn wrap(self) -> UiElement {
        UiElement::Slider(self)
    }

    fn wrapped(&self) -> Element {
        self.weak.upgrade().expect("Weak to itself")
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }

    fn state(&self) -> &UiElementState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut UiElementState {
        &mut self.state
    }

    fn style(&self) -> &UiStyle {
        &self.style
    }

    fn style_mut(&mut self) -> UiStyleWriteObserver {
        create_style_obs(&mut self.style, &mut self.state)
    }

    fn context(&self) -> &UiContext {
        &self.context
    }

    fn body(&self) -> &ElementBody {
        &self.body
    }

    fn body_mut(&mut self) -> &mut ElementBody {
        &mut self.body
    }
}

impl UiElementCallbacks for Slider {
    fn draw(&mut self, ctx: &mut RenderingPipeline<OpenGLRenderer>, crop_area: &SimpleRect, debug: bool) {
        self.body.draw(&self.style, &self.state, ctx, &self.context, crop_area);
        let state = &self.state;
        let style = &self.style;
        //horizontal will always be default for slider so hardcoding it is perfectly fine
        let direction = resolve2!(state, style.direction).unwrap_or(Direction::Horizontal);
        let rect = &self.state.content_rect;

        let ratio = resolve2!(state, style.detail.adaptive_ratio).unwrap_or_default(&DEFAULT_STYLE.detail.adaptive_ratio);
        let (knob_w, knob_h) = utils::shape_size(&style.detail, state, &self.context, &rect.bounding, direction.clone(), ratio, |s| &s.detail);

        let first = self.enumeration.first();
        let last = self.enumeration.last();
        if self.value.is_outdated() {
            let value = *self.value.read();
            self.value.update();
            self.scroll_offset = match direction {
                Direction::Vertical => geom::lerp_num(value, first, last, 0.0, rect.height() as f32 - knob_h as f32) as i32,
                Direction::Horizontal => geom::lerp_num(value, first, last, 0.0, rect.width() as f32 - knob_w as f32) as i32,
            };
        }

        self.scroll_offset = match direction {
            Direction::Vertical => self.scroll_offset.p_clamp(0, rect.height() - knob_h),
            Direction::Horizontal => self.scroll_offset.p_clamp(0, rect.width() - knob_w),
        };

        let knob = match direction {
            Direction::Vertical => SimpleRect::new(rect.x(), rect.y() + self.scroll_offset, knob_w, knob_h),
            Direction::Horizontal => SimpleRect::new(rect.x() + self.scroll_offset, rect.y(), knob_w, knob_h),
        };

        utils::draw_shape_style_at(ctx, &self.context, &knob, &style.detail, state, |s| &s.detail, Some(crop_area.clone()));

        self.drag_assistant.reference = (rect.x(), rect.y());
        if !self.drag_assistant.in_drag {
            self.drag_assistant.target = knob;
        } else {
            self.scroll_offset = match direction {
                Direction::Vertical => self.drag_assistant.global_offset.1,
                Direction::Horizontal => self.drag_assistant.global_offset.0,
            };
        }

        if let Some(steps) = self.enumeration.steps() {
            let track_len = match direction {
                Direction::Vertical => rect.height() - knob_h,
                Direction::Horizontal => rect.width() - knob_w,
            };

            if steps > 1 {
                let step_size = (track_len as f32 / steps as f32).ceil();
                let snapped = ((self.scroll_offset as f32 / step_size).round() * step_size).round() as i32;
                self.scroll_offset = snapped.clamp(0, track_len);
            }
        }

        //recalculate value based on the scroll offset yaaaay i definitely want to do this rn
        if self.dragged {
            self.dragged = false;

            match direction {
                Direction::Vertical => {
                    let min_off = 0.0;
                    let max_off = (rect.height() - knob_h) as f32;
                    let val = geom::lerp_num(
                        self.scroll_offset as f32,
                        min_off,
                        max_off,
                        first,
                        last,
                    );
                    *self.value.write() = val;
                }
                Direction::Horizontal => {
                    let min_off = 0.0;
                    let max_off = (rect.width() - knob_w) as f32;
                    let val = geom::lerp_num(
                        self.scroll_offset as f32,
                        min_off,
                        max_off,
                        first,
                        last,
                    );
                    *self.value.write() = val;
                }
            };
        }

        //draw the current value
        let mut val = *self.value.read();
        if self.enumeration.is_step_size_gte(1.0) {
            val = val.round();
        }
        let text = val.to_rope();
        self.text.draw(0, 0, &text, state, style, ctx, &self.context, crop_area);
    }

    fn raw_input_callback(&mut self, action: RawInputEvent, input: &Input) -> bool {
        let mut used = self.super_input(action.clone(), input);
        let dragged = self.drag_assistant.on_input(action, input);
        self.dragged |= dragged;
        used |= dragged;
        used
    }
}

#[derive(Clone)]
pub enum SliderStep {
    Number(f32),
    Continuous
}

#[derive(Clone)]
pub struct SliderRange {
    low: f32,
    high: f32,
    step: SliderStep,
}

impl SliderRange {
    pub fn from_range(lo: f32, hi: f32) -> Self {
        Self {
            low: lo,
            high: hi,
            step: SliderStep::Continuous,
        }
    }

    pub fn from_range_with_step(lo: f32, hi: f32, step: f32) -> Self {
        Self {
            low: lo,
            high: hi,
            step: SliderStep::Number(step),
        }
    }
}

#[derive(Clone)]
pub enum SliderEnumeration {
    List(Vec<f32>),
    Range(SliderRange)
}

impl FromStr for SliderEnumeration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // List form: [2, 4, 6, 8]
        if s.starts_with('[') && s.ends_with(']') {
            let inner = &s[1..s.len() - 1];
            if inner.trim().is_empty() {
                return Ok(SliderEnumeration::List(Vec::new()));
            }

            let values: Result<Vec<f32>, _> = inner
                .split(',')
                .map(|x| x.trim().parse::<f32>().map_err(|e| e.to_string()))
                .collect();

            return Ok(SliderEnumeration::List(values?));
        }

        // Range form: 1..5 or 1..5@0.5
        let (range_part, step_part) = match s.split_once('@') {
            Some((range_str, step_str)) => (range_str.trim(), Some(step_str.trim())),
            None => (s, None),
        };

        if let Some(idx) = range_part.find("..") {
            let start = &range_part[..idx];
            let end = if range_part[idx..].starts_with("..=") {
                &range_part[idx + 3..]
            } else {
                &range_part[idx + 2..]
            };

            let start_val = start.trim().parse::<f32>().map_err(|e| e.to_string())?;
            let end_val = end.trim().parse::<f32>().map_err(|e| e.to_string())?;

            return if let Some(step_str) = step_part {
                let step_val = step_str.parse::<f32>().map_err(|e| e.to_string())?;
                Ok(SliderEnumeration::Range(SliderRange::from_range_with_step(
                    start_val,
                    end_val,
                    step_val,
                )))
            } else {
                Ok(SliderEnumeration::Range(SliderRange::from_range(
                    start_val,
                    end_val,
                )))
            }
        }


        Err(format!("Invalid slider enumeration format: {}", s))
    }
}

impl SliderEnumeration {
    pub fn first(&self) -> f32 {
        match self {
            SliderEnumeration::List(v) => v.first().copied().unwrap_or_default(),
            SliderEnumeration::Range(r) => r.low
        }
    }

    pub fn last(&self) -> f32 {
        match self {
            SliderEnumeration::List(v) => v.last().copied().unwrap_or_default(),
            SliderEnumeration::Range(r) => r.high
        }
    }

    pub fn steps(&self) -> Option<i32> {
        match self {
            SliderEnumeration::List(v) => Some(v.len() as i32),
            SliderEnumeration::Range(r) => {
                match r.step {
                    SliderStep::Number(n) => Some(((r.high - r.low) / n) as i32),
                    SliderStep::Continuous => None
                }
            }
        }
    }

    pub fn is_step_size_gte(&self, gte: f32) -> bool {
        match self {
            SliderEnumeration::List(_) => 1.0 >= gte,
            SliderEnumeration::Range(r) => {
                match r.step {
                    SliderStep::Number(n) => n >= gte,
                    SliderStep::Continuous => false
                }
            }
        }
    }
}