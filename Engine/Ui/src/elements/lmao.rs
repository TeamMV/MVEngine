use std::sync::Arc;
use parking_lot::RwLock;
use crate::render::color::RgbColor;
use crate::render::draw2d::DrawContext2D;
use crate::render::text::FontLoader;
use crate::{resolve, ui};
use crate::resources::resources::R;
use crate::ui::attributes::Attributes;
use crate::ui::elements::child::Child;
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementState};
use crate::ui::styles::{Dimension, UiStyle};

pub struct LmaoElement {
    state: UiElementState,
    style: UiStyle,
    col: RgbColor,
    attributes: Attributes,
}

impl UiElementCallbacks for LmaoElement {
    fn init(&mut self) {
        todo!()
    }

    fn draw(&mut self, ctx: &mut DrawContext2D) {
        //draw margins
        ctx.color(RgbColor::new(1.0, 0.0, 0.0, 0.2));
        //left
        ctx.rectangle(
            self.state.bounding_x,
            self.state.bounding_y,
            self.state.margins[2],
            self.state.bounding_height,
        );

        //right
        ctx.rectangle(
            self.state.x + self.state.width,
            self.state.bounding_y,
            self.state.margins[3],
            self.state.bounding_height,
        );

        //top
        ctx.rectangle(
            self.state.bounding_x,
            self.state.bounding_y + self.state.bounding_height - self.state.margins[0],
            self.state.width,
            self.state.margins[0],
        );

        //bottom
        ctx.rectangle(
            self.state.bounding_x + self.state.margins[2],
            self.state.bounding_y,
            self.state.width,
            self.state.margins[1],
        );

        //draw paddings
        ctx.color(RgbColor::new(0.0, 1.0, 0.0, 0.2));

        //left
        ctx.rectangle(
            self.state.x,
            self.state.y,
            self.state.paddings[2],
            self.state.height,
        );

        //right
        ctx.rectangle(
            self.state.content_x + self.state.bounding_width,
            self.state.y,
            self.state.paddings[3],
            self.state.height,
        );

        //top
        ctx.rectangle(
            self.state.content_x,
            self.state.content_y + self.state.content_height,
            self.state.content_width,
            self.state.paddings[0],
        );

        //bottom
        ctx.rectangle(
            self.state.content_x,
            self.state.y,
            self.state.content_width,
            self.state.paddings[1],
        );

        /*ctx.color(self.col.clone());
        ctx.void_rectangle(
            self.state.x,
            self.state.y,
            self.state.width,
            self.state.height,
            3,
        );*/

        if self.state.background.is_some() {
            let bg = self.state.background.clone().unwrap();
            let bg_guard = bg.write();
            bg_guard.draw(ctx, self.state(), self.style());
        }

        let x = self.state.content_x;
        let y = self.state.content_y;
        let height = resolve!(self, text.size) as i32;

        //println!("{x}, {y}, {height}");

        let text_color = ui::utils::resolve_color(&self.style.text.color, RgbColor::black(), self.state(), |s| &s.text.color);

        for child in self.children_mut() {
            if child.is_element() {
                let mut elem = match child {
                    Child::Element(ref mut e) => e,
                    _ => {
                        unreachable!()
                    }
                }
                .write();
                elem.draw(ctx);
            } else {
                let s = child.as_string();
                ctx.color(text_color);
                ctx.text(false, x, y, height, s.as_str());
            }
        }
    }
}

impl UiElement for LmaoElement {
    fn new(attributes: Attributes, style: UiStyle) -> Self
    where
        Self: Sized,
    {
        Self {
            state: UiElementState::new(),
            style,
            col: RgbColor::blue(),
            attributes,
        }
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    fn attributes_mut(&mut self) -> &Attributes {
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

    fn style_mut(&mut self) -> &mut UiStyle {
        &mut self.style
    }

    fn components(&self) -> (&Attributes, &UiStyle, &UiElementState) {
        (&self.attributes, &self.style, &self.state)
    }

    fn components_mut(&mut self) -> (&mut Attributes, &mut UiStyle, &mut UiElementState) {
        (&mut self.attributes, &mut self.style, &mut self.state)
    }

    fn get_size(&self, s: &str) -> Dimension<i32> {
        let font = R::fonts().get_core("default");
        let height = resolve!(self, text.size);
        let width = font.get_metrics(s).width(height as i32);
        let dim = Dimension::new(width, height as i32);
        //println!("{:?}", dim);
        dim
    }
}
