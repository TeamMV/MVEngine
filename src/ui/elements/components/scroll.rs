use crate::color::RgbColor;
use crate::rendering::RenderContext;
use crate::ui::context::UiContext;
use crate::ui::elements::components::ElementBody;
use crate::ui::elements::{UiElementState, UiElementStub};
use crate::ui::geometry::{shape, SimpleRect};
use crate::ui::styles::{UiStyle, DEFAULT_STYLE};
use crate::resolve2;
use mvutils::lazy;

lazy! {
    pub static OUTER_COLOR: RgbColor = RgbColor::new([150, 150, 150, 255]);
    pub static INNER_COLOR: RgbColor = RgbColor::new([87, 87, 87, 255]);
}

#[derive(Clone)]
pub struct ScrollBars;

impl ScrollBars {
    pub fn draw(
        style: &UiStyle,
        state: &UiElementState,
        body: &ElementBody,
        ctx: &mut impl RenderContext,
        context: &UiContext,
        crop_area: &SimpleRect,
    ) {
        let bar_extent = resolve2!(state, body, style.scrollbar.size).unwrap_or_default_or_percentage(
            &DEFAULT_STYLE.scrollbar.size,
            state.parent.clone(),
            |s| s.width(),
            state,
        );

        if state.scroll_x.available {
            let resolved = resolve2!(state, body, style.scrollbar.track.shape);
            let resource = resolve2!(state, body, style.scrollbar.track.resource);
            if resolved.is_set() && !resource.is_none() {
                let mut rect = state.rect.bounding.clone();
                rect.height = bar_extent;
                shape::utils::draw_shape_style_at(
                    ctx,
                    context,
                    &rect,
                    &style.scrollbar.track,
                    state,
                    body,
                    |s| &s.scrollbar.track,
                    Some(crop_area.clone()),
                );
            }

            let knob = Self::x_knob(state, bar_extent);

            let resolved = resolve2!(state, body, style.scrollbar.knob.shape);
            let resource = resolve2!(state, body, style.scrollbar.knob.resource);
            if resolved.is_set() && !resource.is_none() {
                shape::utils::draw_shape_style_at(
                    ctx,
                    context,
                    &knob,
                    &style.scrollbar.knob,
                    state,
                    body,
                    |s| &s.scrollbar.knob,
                    Some(crop_area.clone()),
                );
            }
        }

        if state.scroll_y.available {
            let resolved = resolve2!(state, body, style.scrollbar.track.shape);
            let resource = resolve2!(state, body, style.scrollbar.track.resource);
            if resolved.is_set() && !resource.is_none() {
                let rect = SimpleRect::new(
                    state.rect.x() + state.rect.width() - bar_extent,
                    state.rect.y(),
                    bar_extent,
                    state.rect.height(),
                );

                shape::utils::draw_shape_style_at(
                    ctx,
                    context,
                    &rect,
                    &style.scrollbar.track,
                    state,
                    body,
                    |s| &s.scrollbar.track,
                    Some(crop_area.clone()),
                );
            }

            let knob = Self::y_knob(state, bar_extent);

            let resolved = resolve2!(state, body, style.scrollbar.knob.shape);
            let resource = resolve2!(state, body, style.scrollbar.knob.resource);
            if resolved.is_set() && !resource.is_none() {
                shape::utils::draw_shape_style_at(
                    ctx,
                    context,
                    &knob,
                    &style.scrollbar.knob,
                    state,
                    body,
                    |s| &s.scrollbar.knob,
                    Some(crop_area.clone()),
                );
            }
        }
    }

    pub fn x_knob(state: &UiElementState, bar_extent: i32) -> SimpleRect {
        let knob_width = (state.rect.width() as f32 / state.scroll_x.whole as f32)
            * state.rect.width() as f32;
        let knob_width = knob_width as i32;
        SimpleRect::new(
            state.rect.x() + state.scroll_x.offset,
            state.rect.y(),
            knob_width,
            bar_extent,
        )
    }

    pub fn y_knob(state: &UiElementState, bar_extent: i32) -> SimpleRect {
        let knob_height = (state.rect.height() as f32 / state.scroll_y.whole as f32)
            * state.rect.height() as f32;
        let knob_height = knob_height as i32;
        SimpleRect::new(
            state.rect.x() + state.rect.width() - bar_extent,
            state.rect.y() + state.rect.height()
                - state.scroll_y.offset
                - knob_height,
            bar_extent,
            knob_height,
        )
    }
}
