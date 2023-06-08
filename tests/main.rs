use std::sync::{Arc, Mutex, RwLock};
use mvsync::MVSync;
use mvutils::once::{CreateOnce, InitOnce};
use mvutils::utils::Recover;
use mvutils::version::Version;

use mvcore::{ApplicationInfo, draw_2d, MVCore, setup};
use mvcore::gui::components::{GuiComponent, GuiElement, GuiLayout, GuiMarkdown, GuiSection, GuiTextComponent};
use mvcore::gui::gui_formats::FormattedString;
use mvcore::gui::styles::GuiValue;
use mvcore::gui::styles::GuiValue::{Just, Measurement};
use mvcore::render::{ApplicationLoopCallbacks, RenderCore};
use mvcore::render::color::RGB;
use mvcore::render::color::Color;
use mvcore::render::window::{Window, WindowSpecs};

fn main() {
    env_logger::init();
    let core = MVCore::new(ApplicationInfo {
        name: "Test".to_string(),
        version: Version::new(1, 0, 0),
        multithreaded: true,
        extra_threads: 1,
    });
    let mut specs = WindowSpecs::default();
    specs.vsync = false;
    specs.fps = 10000;
    specs.decorated = true;
    specs.resizable = true;
    specs.width = 800;
    specs.height = 600;
    core.get_render().run_window(specs, ApplicationLoop {layout: CreateOnce::new()})
}

struct ApplicationLoop {
    layout: CreateOnce<GuiLayout>
}

impl ApplicationLoopCallbacks for ApplicationLoop {
    fn start(&self, window: Arc<Window<Self>>) {
        let mut elem = GuiMarkdown::create();
        elem.set_text(FormattedString::new("Hello, World!"));

        let mut pg = GuiElement::Paragraph(elem);

        setup!(pg.info_mut().style => {
            text_size: 0.1 dm
        });

        let layout = GuiElement::Layout(GuiLayout::Section(GuiSection::create()));
        layout.layout().elements().add_element(&mut pg);

        self.layout.create(layout);
    }

    fn update(&self, window: Arc<Window<Self>>) {

    }

    fn draw(&mut self, window: Arc<Window<Self>>) {
        draw_2d!(window => {
            rectangle 100, 0, 0, 100;
            gui "myGui";
        });
    }

    fn exit(&self, window: Arc<Window<Self>>) {

    }
}