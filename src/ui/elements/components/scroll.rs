use crate::ui::res::err::UiResErr;
use crate::ui::res::err::ResType;
use std::ops::Deref;
use crate::ui::styles::{ResolveResult, DEFAULT_STYLE};
use crate::color::RgbColor;
use crate::ui::context::UiContext;
use crate::ui::elements::{UiElementState, UiElementStub};
use crate::ui::geometry::{Rect, SimpleRect};
use mvutils::lazy;
use crate::{get_adaptive, get_shape, resolve};
use crate::graphics::Drawable;
use crate::rendering::RenderContext;
use crate::ui::geometry::shape::Shape;
use crate::ui::rendering::adaptive::{AdaptiveFill, AdaptiveShape};
use crate::ui::styles::enums::{BackgroundRes, Geometry};

lazy! {
    pub static OUTER_COLOR: RgbColor = RgbColor::new([150, 150, 150, 255]);
    pub static INNER_COLOR: RgbColor = RgbColor::new([87, 87, 87, 255]);
}

#[derive(Clone)]
pub struct ScrollBars {}

impl ScrollBars {
    pub fn draw<E: UiElementStub + 'static>(
        &mut self,
        elem: &E,
        ctx: &mut impl RenderContext,
        context: &UiContext,
        crop_area: &SimpleRect
    ) {
        let res = context.resources;
        let state = elem.state();
        
        let bar_extent = resolve!(elem, scrollbar.size)
            .unwrap_or_default_or_percentage(&DEFAULT_STYLE.scrollbar.size, state.parent.clone(), |s| s.width(), state);
        
        
        if state.scroll_x.available {
            let resolved = resolve!(elem, scrollbar.track.shape);
            let resource = resolve!(elem, scrollbar.track.resource);
            if resolved.is_set() && !resource.is_none() {
                let mut rect = state.content_rect.bounding.clone();
                rect.height = bar_extent;
                todo!()
            }

            let knob = Self::x_knob(state, bar_extent);

            let resolved = resolve!(elem, scrollbar.knob.shape);
            let resource = resolve!(elem, scrollbar.knob.resource);
            if resolved.is_set() && !resource.is_none() {
                todo!()
            }
        }

        if state.scroll_y.available {
            let resolved = resolve!(elem, scrollbar.track.shape);
            let resource = resolve!(elem, scrollbar.track.resource);
            if resolved.is_set() && !resource.is_none() {
                let rect = SimpleRect::new(state.content_rect.x() + state.content_rect.width() - bar_extent,
                                               state.content_rect.y(),
                                               bar_extent,
                                               state.content_rect.height());
                
                todo!()
            }

            let knob = Self::y_knob(state, bar_extent);

            let resolved = resolve!(elem, scrollbar.knob.shape);
            let resource = resolve!(elem, scrollbar.knob.resource);
            if resolved.is_set() && !resource.is_none() {
                todo!()
            }
        }
    }

    pub fn x_knob(state: &UiElementState, bar_extent: i32) -> SimpleRect {        
        let knob_width = (state.content_rect.width() as f32 / state.scroll_x.whole as f32)
            * state.content_rect.width() as f32;
        let knob_width = knob_width as i32;
        SimpleRect::new(
            state.content_rect.x() + state.scroll_x.offset,
            state.content_rect.y(),
            knob_width,
            bar_extent,
        )
    }

    pub fn y_knob(state: &UiElementState, bar_extent: i32) -> SimpleRect {        
        let knob_height = (state.content_rect.height() as f32 / state.scroll_y.whole as f32)
            * state.content_rect.height() as f32;
        let knob_height = knob_height as i32;
        SimpleRect::new(
            state.content_rect.x() + state.content_rect.width() - bar_extent,
            state.content_rect.y() + state.content_rect.height() - state.scroll_y.offset - knob_height,
            bar_extent,
            knob_height,
        )
    }
}
