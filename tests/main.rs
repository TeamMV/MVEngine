use std::sync::Arc;

use mvutils::version::Version;

use mvcore::render::window::{Window, WindowSpecs};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::{draw_2d, ApplicationInfo, MVCore};

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
    core.get_render().run_window(specs, ApplicationLoop {})
}

struct ApplicationLoop {
    // layout: CreateOnce<GuiLayout>
}

impl ApplicationLoopCallbacks for ApplicationLoop {
    fn start(&self, window: Arc<Window<Self>>) {
        //let mut elem = GuiMarkdown::create();
        //elem.set_text(FormattedString::new("Hello, World!"));

        //let mut pg = GuiElement::Paragraph(elem);

        //setup!(pg.info_mut().style => {
        //    text_size: 0.1 dm
        //});

        //let layout = GuiElement::Layout(GuiLayout::Section(GuiSection::create()));
        //layout.layout().elements().add_element(&mut pg);

        //self.layout.create(layout);

        //let gui = Gui::new(&self.layout);
        //R::guis().register_core(String::from("myGui"), Arc::new(gui));
    }

    fn update(&self, window: Arc<Window<Self>>) {}

    fn draw(&self, window: Arc<Window<Self>>) {
        draw_2d!(window => {
            rectangle 100, 0, 0, 100;
            //gui "myGui";
        });
    }

    fn exit(&self, window: Arc<Window<Self>>) {}
}
