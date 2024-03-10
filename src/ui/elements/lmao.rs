use crate::render::color::RgbColor;
use crate::render::draw2d::DrawContext2D;
use crate::render::text::FontLoader;
use crate::resources::resources::R;
use crate::ui::attributes::Attributes;
use crate::ui::elements::{UiElement, UiElementCallbacks, UiElementState};
use crate::ui::elements::child::Child;
use crate::ui::styles::{Dimension, UiStyle};

pub struct LmaoElement {
    state: UiElementState,
    style: UiStyle,
    col: RgbColor,
    children: Vec<Child>
}

impl UiElementCallbacks for LmaoElement {
    fn init(&mut self) {
        todo!()
    }

    fn draw(&mut self, ctx: &mut DrawContext2D) {
        ctx.color(self.col.clone());
        ctx.rectangle(self.state.content_x, self.state.content_y, self.state.content_width, self.state.content_height);

        for child in self.children {
            if child.is_element() {
                let elem = match child { Child::Element(ref mut e) => { e } _ => { unreachable!() } };
                elem.draw(ctx);
            } else {
                let s = child.as_string();
                ctx.color(RgbColor::white());
                ctx.text(false, self.state.content_x, self.state.content_y, self.state.content_height, s.as_str());
            }
        }
    }
}

impl UiElement for LmaoElement {
    fn new(attributes: Attributes, style: UiStyle) -> Self where Self: Sized {
        Self {
            state: UiElementState::new(),
            style,
            col: RgbColor::blue(),
            children: vec![]
        }
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

    fn add_child(&mut self, child: Child) {
        self.children.push(child);
    }

    fn children(&self) -> impl Iterator<Item=&Child> {
        self.children.iter()
    }

    fn children_mut(&mut self) -> impl Iterator<Item=&mut Child> {
        self.children.iter_mut()
    }

    fn get_size(&self, s: &str) -> Dimension<i32> {
        let font = R::fonts().get_core("default");
        let width = font.get_metrics(s).width(self.state.content_height);
        Dimension::new(width, self.state.content_height)
    }
}