use std::sync::{Arc, RwLock};
use mvutils::utils::Recover;

use crate::gui::components::GuiLayout;
use crate::render::draw2d::Draw2D;

pub mod archive;
pub mod components;
pub mod ease;
pub mod gui_formats;
pub mod styles;
pub mod animation;

pub struct Gui {
    root: Arc<RwLock<GuiLayout>>,
}

unsafe impl Send for Gui {}

unsafe impl Sync for Gui {}

impl Gui {
    pub fn new(root: Arc<RwLock<GuiLayout>>) -> Self {
        Self { root }
    }

    pub(crate) fn draw(&mut self, draw_2d: &mut Draw2D) {
        self.root.write().recover().draw(draw_2d);
    }
}

pub struct GuiRenderer {
    to_render: Vec<Arc<RwLock<Gui>>>,
}

impl GuiRenderer {
    pub fn new() -> Self {
        Self { to_render: vec![] }
    }

    pub fn request_draw(&mut self, gui: Arc<RwLock<Gui>>) {
        self.to_render.push(gui);
    }

    pub(crate) fn render(&mut self, draw_2d: &mut Draw2D) {
        for gui in self.to_render.iter_mut() {
            gui.write().recover().draw(draw_2d);
        }
    }
}
