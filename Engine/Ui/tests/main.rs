use log::LevelFilter;
use mvcore::input;
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::backend::image::{AccessFlags, ImageLayout};
use mvcore::render::backend::Backend;
use mvcore::render::renderer::Renderer;
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mve2d::renderer2d::Renderer2D;
use mvutils::unsafe_utils::DangerousCell;
use mvutils::version::Version;
use parking_lot::RwLock;
use std::sync::Arc;
use Ui::anim::{AnimationMode, FillMode};
use Ui::attributes::Attributes;
use Ui::elements::lmao::LmaoElement;
use Ui::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use Ui::styles::{Origin, Position, UiStyle, UiValue};
use Ui::timing::TIMING_MANAGER;
use Ui::{anim, modify_style, resolve, UI};
use Ui::elements::events::{UiClickAction, UiHoverAction};

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);
    let mut info = WindowCreateInfo::default();
    info.title = "UI test".to_string();
    info.fps = 60;
    info.ups = 20;
    info.vsync = true;

    let window = Window::new(info);
    window.run::<Application>();
}

struct Application {
    device: Device,
    core_renderer: Arc<DangerousCell<Renderer>>,
    renderer2d: Arc<DangerousCell<Renderer2D>>,
    elem: Arc<RwLock<UiElement>>,
}

impl ApplicationLoopCallbacks for Application {
    fn new(window: &mut Window) -> Self {
        let device = Device::new(
            Backend::Vulkan,
            MVDeviceCreateInfo {
                app_name: "Test app".to_string(),
                app_version: Version::new(0, 0, 1, 0),
                engine_name: "MVEngine".to_string(),
                engine_version: Version::new(0, 0, 1, 0),
                device_extensions: Extensions::empty(),
            },
            &window.get_handle(),
        );
        let core_renderer = Arc::new(DangerousCell::new(Renderer::new(&window, device.clone())));

        let renderer2d = Arc::new(DangerousCell::new(Renderer2D::new(
            device.clone(),
            core_renderer.clone(),
            core_renderer.get().get_swapchain().get_extent(),
            0,
            0,
        )));

        let mut style = UiStyle::default();
        modify_style!(style.position = UiValue::Just(Position::Absolute));
        modify_style!(style.x = UiValue::Just(100));
        modify_style!(style.y = UiValue::Just(100));
        modify_style!(style.width = UiValue::Just(100));
        modify_style!(style.height = UiValue::Just(100));
        modify_style!(style.transform.origin = UiValue::Just(Origin::Center));

        let mut anim_style_from = style.clone();
        let mut anim_style_to = style.clone();
        modify_style!(anim_style_to.transform.scale! = UiValue::Just(1.2));

        let mut lmao = UiElement::Lmao(LmaoElement::new(Attributes::new(), style));

        lmao.state_mut().events.on_hover(move |event| {
            let elem = event.base.elem;
            if let UiHoverAction::Enter = event.base.action {
                anim::animate_self(elem, &anim_style_to, 200, anim::EASING_LINEAR, FillMode::Keep, AnimationMode::StartOver);
            } else {
                anim::animate_self(elem, &anim_style_from, 200, anim::EASING_LINEAR, FillMode::Keep, AnimationMode::StartOver);
            }
        });

        let arc = Arc::new(RwLock::new(lmao));

        unsafe {
            UI.get_mut().init_input(window.get_input());
            UI.get_mut().add_root(arc.clone());
            window.set_input_processor(Ui::Ui::input_processor);
        }


        Self {
            device,
            core_renderer,
            renderer2d,
            elem: arc,
        }
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {}

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        let ren = self.renderer2d.get_mut();
        UiElementState::compute(self.elem.clone(), ren);

        let mut guard = self.elem.write();
        guard.draw(ren);
        drop(guard);


        let image_index = self.core_renderer.get_mut().begin_frame().unwrap();
        let cmd = self.core_renderer.get_mut().get_current_command_buffer();
        let frame_index = self.core_renderer.get().get_current_frame_index();

        ren.draw();

        cmd.blit_image(
            ren.get_geometry_image(frame_index as usize),
            self.core_renderer
                .get_mut()
                .get_swapchain()
                .get_framebuffer(image_index as usize)
                .get_image(0),
        );

        self.core_renderer
            .get_mut()
            .get_swapchain()
            .get_framebuffer(image_index as usize)
            .get_image(0)
            .transition_layout(
                ImageLayout::PresentSrc,
                Some(cmd),
                AccessFlags::empty(),
                AccessFlags::empty(),
            );

        self.core_renderer.get_mut().end_frame().unwrap();

        unsafe { TIMING_MANAGER.post_frame(delta_t as f32, 0); }
    }

    fn exiting(&mut self, window: &mut Window) {}

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {}
}