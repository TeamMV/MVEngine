use std::process::exit;
use log::LevelFilter;
use mvcore::color::RgbColor;
use mvcore::input;
use mvcore::input::raw::Input;
use mvcore::math::vec::{Vec2, Vec3};
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::backend::image::{AccessFlags, ImageLayout};
use mvcore::render::backend::Backend;
use mvcore::render::renderer::Renderer;
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mve2d::renderer2d::{GameRenderer2D, Shape};
use mvengine_ui::anim::complex::KeyframeAnimation;
use mvengine_ui::anim::{AnimationMode, FillMode};
use mvengine_ui::attributes::Attributes;
use mvengine_ui::ease::{EasingGen, EasingMode};
use mvengine_ui::elements::events::UiClickAction;
use mvengine_ui::elements::lmao::LmaoElement;
use mvengine_ui::elements::{UiElement, UiElementCallbacks, UiElementState, UiElementStub};
use mvengine_ui::styles::{Origin, Position, UiStyle, UiValue};
use mvengine_ui::timing::TIMING_MANAGER;
use mvengine_ui::{anim, modify_style, UI};
use mvutils::unsafe_utils::DangerousCell;
use mvutils::version::Version;
use parking_lot::RwLock;
use std::sync::Arc;
use mvutils::state::State;
use uiproc::ui;

use mvengine_ui::elements::Div;

fn main() {
    let xml = r#"<tag1 attr={let a = 1; {}}><tag2 hello="world">hello world</tag2></tag1>"#;

    let s = State::new("hello");

    let mut e = ui!([s] => {
            <Div id="outer" style={mystyle}>
                <Text>{s}</Text>
            </Div>
        }
    );

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
    renderer2d: Arc<DangerousCell<GameRenderer2D>>,
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

        let renderer2d = Arc::new(DangerousCell::new(GameRenderer2D::new(
            device.clone(),
            core_renderer.clone(),
            core_renderer.get().get_swapchain().get_extent(),
            0,
            0,
        )));

        let mut style = UiStyle::default();
        modify_style!(style.position = UiValue::Just(Position::Absolute));
        modify_style!(style.x = UiValue::Just(300));
        modify_style!(style.y = UiValue::Just(300));
        modify_style!(style.width = UiValue::Just(100));
        modify_style!(style.height = UiValue::Just(100));
        modify_style!(style.transform.origin = UiValue::Just(
            Origin::Custom(50, 50)
        ));

        let mut anim_style_from = style.clone();
        let mut anim_style_to = style.clone();
        modify_style!(anim_style_to.transform.scale! = UiValue::Just(2.0));

        let mut lmao = UiElement::Lmao(LmaoElement::new(Attributes::new(), style));

        let animation = KeyframeAnimation::builder(anim_style_from.clone())
            .next_keyframe(|s| {
                modify_style!(s.x = UiValue::Just(400));
            }, Some(anim::easing(EasingGen::sin(), EasingMode::Out)), Some(10.0))
            .next_keyframe(|s| {
                modify_style!(s.y = UiValue::Just(400));
            }, Some(anim::easing(EasingGen::back(), EasingMode::InOut)), Some(60.0))
            .next_keyframe(|s| {
                modify_style!(s.x = UiValue::Just(2));
            }, Some(anim::easing(EasingGen::bounce(), EasingMode::Out)), None)
            .build();

        lmao.state_mut().events.on_click(move |event| {
            let elem = event.base.elem;
            if event.button == input::MOUSE_LEFT {
                if let UiClickAction::Click = event.base.action {
                    anim::animate_self(elem, &anim_style_to, 200, anim::easing(EasingGen::linear(), EasingMode::In), FillMode::Keep, AnimationMode::KeepProgress);
                } else {
                    anim::animate_self(elem, &anim_style_from, 200, anim::easing(EasingGen::linear(), EasingMode::In), FillMode::Keep, AnimationMode::KeepProgress);
                }
            }
        });

        let arc = Arc::new(RwLock::new(lmao));

        unsafe {
            UI.get_mut().init_input(window.get_input());
            UI.get_mut().add_root(arc.clone());
            window.set_input_processor(mvengine_ui::Ui::input_processor);
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

        let inp = window.get_input();
        if inp.get().keys[Input::key_from_str("w")] {
            ren.add_shape(Shape::Rectangle {
                position: Vec3::new(50.0, 50.0, 0.0),
                rotation: Default::default(),
                scale: Vec2::splat(100.0),
                tex_id: None,
                tex_coord: Default::default(),
                color: RgbColor::yellow().as_vec4(),
                blending: 0.0,
            })
        }

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