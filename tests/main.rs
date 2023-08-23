use std::sync::{Arc, RwLock};
use mvutils::once::CreateOnce;
use mvutils::screen::Measurement;
use mvutils::utils::Recover;

use mvutils::version::Version;

use mvcore::render::window::{Window, WindowSpecs};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::{draw_2d, ApplicationInfo, MVCore, style};
use mvcore::gui::components::{GuiComponent, GuiElement, GuiLayout, GuiLayoutComponent, GuiTextComponent, GuiTextElement};
use mvcore::gui::components::layout::GuiSection;
use mvcore::gui::components::text::GuiLabel;
use mvcore::gui::styles::{BorderStyle, GuiValue, Positioning};
use mvcore::render::color::{Color, Gradient, RGB};

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
    specs.fps = 60;
    specs.decorated = true;
    specs.resizable = true;
    specs.width = 800;
    specs.height = 600;
    core.get_render().run_window(specs, ApplicationLoop { elem: CreateOnce::new() })
}

struct ApplicationLoop {
    elem: CreateOnce<Arc<RwLock<GuiElement>>>
}

impl ApplicationLoopCallbacks for ApplicationLoop {
    fn start(&self, window: Arc<Window<Self>>) {
        let mut label = GuiLabel::create();

        label.set_text("Hello".to_string());
        label.info_mut().style.x = GuiValue::Just(500);
        label.info_mut().style.y = GuiValue::Just(400);
        label.info_mut().style.position = GuiValue::Just(Positioning::Absolute);
        label.info_mut().style.padding_left = GuiValue::Just(20);
        label.info_mut().style.padding_right = GuiValue::Just(20);

        label.info_mut().style.text_size = GuiValue::Just(64);
        label.info_mut().style.text_chroma = GuiValue::Just(true);

        label.info_mut().style.background_color = GuiValue::Just(Gradient::new(Color::<RGB, f32>::white()));
        label.info_mut().style.border_style = GuiValue::Just(BorderStyle::Triangle);
        label.info_mut().style.border_color = GuiValue::Just(Gradient::new(Color::<RGB, f32>::cyan()));
        label.info_mut().style.border_radius = GuiValue::Just(20);
        label.info_mut().style.border_width = GuiValue::Just(5);


        let mut section = GuiSection::create();
        section.info_mut().style.x = GuiValue::Just(500);
        section.info_mut().style.y = GuiValue::Just(400);
        section.info_mut().style.position = GuiValue::Just(Positioning::Absolute);

        section.info_mut().style.background_color = GuiValue::Just(Gradient::new(Color::<RGB, f32>::red()));

        let label_elem = Arc::new(RwLock::new(GuiElement::Text(GuiTextElement::Label(label))));

        //section.elements_mut().add_element(label_elem);
//
        //let section_elem = Arc::new(RwLock::new(GuiElement::Layout(GuiLayout::Section(section))));

        self.elem.create(|| label_elem);
    }

    fn update(&self, window: Arc<Window<Self>>) {

    }

    fn draw(&self, window: Arc<Window<Self>>) {
        let mut guard = self.elem.write().recover();

        window.draw_2d_pass(|ctx| {
            ctx.color(Color::<RGB, f32>::red());
            ctx.rectangle(100, 100, 100, 100);
            guard.draw(ctx);
        });
    }

    fn exit(&self, window: Arc<Window<Self>>) {}
}
