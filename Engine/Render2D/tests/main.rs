use std::sync::Arc;
use log::LevelFilter;
use mvutils::unsafe_utils::DangerousCell;
use mvcore::math::vec::{Vec2, Vec3, Vec4};
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::backend::{Backend, Extent2D};
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mvengine_render2d::renderer2d::{Renderer2D, Transform};
use mvutils::version::Version;
use mvcore::asset::asset::AssetType;
use mvcore::asset::manager::{AssetHandle, AssetManager};
use mvcore::render::backend::image::{AccessFlags, ImageLayout};
use mvcore::render::backend::sampler::{Filter, MipmapMode, MVSamplerCreateInfo, Sampler, SamplerAddressMode};
use mvcore::render::renderer::Renderer;

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    let window = Window::new(WindowCreateInfo {
        width: 600,
        height: 600,
        title: "Demo".to_string(),
        fullscreen: false,
        decorated: true,
        resizable: true,
        transparent: false,
        theme: None,
        vsync: true,
        max_frames_in_flight: 2,
        fps: 9999,
        ups: 20,
    });

    window.run::<AppLoop>();
}

struct AppLoop {
    device: Device,
    core_renderer: Arc<DangerousCell<Renderer>>,
    renderer2d: Arc<DangerousCell<Renderer2D>>,

    quad_rotation: f32,
    quad_position: Vec2,
    timer: f32,

    manager: Arc<AssetManager>,
    handle: AssetHandle,
    sampler: Sampler,
}

impl ApplicationLoopCallbacks for AppLoop {
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

        let renderer2d = Arc::new(DangerousCell::new(Renderer2D::new(device.clone(), core_renderer.clone(), core_renderer.get().get_swapchain().get_extent())));

        let manager = AssetManager::new(device.clone(), 1, 1);

        let handle = manager.create_asset("texture.png", AssetType::Texture);

        let sampler = Sampler::new(device.clone(), MVSamplerCreateInfo {
            address_mode: SamplerAddressMode::ClampToEdge,
            filter_mode: Filter::Nearest,
            mipmap_mode: MipmapMode::Nearest,
            anisotropy: false,
            label: None,
        });

        Self { sampler, device, renderer2d, core_renderer, quad_rotation: 0.0, quad_position: Vec2::splat(0.0), timer: 0.0, manager, handle }
    }

    fn post_init(&mut self, _window: &mut Window) {
        let device = self.device.clone();
        let renderer2d = self.renderer2d.clone();
        let sampler = self.sampler.clone();
        self.handle.load(move |handle, _| {
            let asset = handle.get();
            if asset.failed() {
                println!("Failed!");
            } else {
                let Some(texture) = asset.as_texture() else { unreachable!() };

                device.wait_idle();
                for set in renderer2d.get_mut().get_atlas_sets() {
                    set.update_image(0, &texture.image(), &sampler, ImageLayout::ShaderReadOnlyOptimal);
                }
            }
        });
    }

    fn update(&mut self, _window: &mut Window, _delta_t: f64) {

    }

    fn draw(&mut self, _window: &mut Window, delta_t: f64) {
        self.manager.poll_queue(0);

        self.timer += delta_t as f32;
        self.quad_rotation += delta_t as f32 * 2.0;

        self.quad_position.x = self.timer.sin() * 100.0;
        self.quad_position.y = self.timer.cos() * 100.0;

        let renderer2d = self.renderer2d.get_mut();

        renderer2d.add_quad(Transform {
            position: Vec3::new(self.quad_position.x + 300.0, -self.quad_position.y + 400.0, 1.0),
            rotation: Vec3::new(0.0, 0.0, self.quad_rotation),
            scale: Vec2::splat(50.0),
            tex_coord: Vec4::new(0.0, 0.0, 1.0, 1.0),
            color: Vec4::new(1.0, 0.0, 0.0, 0.5),
        });

        renderer2d.add_quad(Transform {
            position: Vec3::new(self.quad_position.x + 300.0, -self.quad_position.y + 200.0, 1.0),
            rotation: Vec3::new(0.0, 0.0, -self.quad_rotation),
            scale: Vec2::splat(50.0),
            tex_coord: Vec4::new(0.0, 0.0, 1.0, 1.0),
            color: Vec4::splat(0.0),
        });

        renderer2d.add_quad(Transform {
            position: Vec3::new(self.quad_position.x + 200.0, -self.quad_position.y + 300.0, 1.0),
            rotation: Vec3::new(0.0, 0.0, self.quad_rotation),
            scale: Vec2::splat(50.0),
            tex_coord: Vec4::new(0.0, 0.0, 1.0, 1.0),
            color: Vec4::splat(0.0),
        });

        renderer2d.add_quad(Transform {
            position: Vec3::new(self.quad_position.x + 400.0, -self.quad_position.y + 300.0, 1.0),
            rotation: Vec3::new(0.0, 0.0, -self.quad_rotation),
            scale: Vec2::splat(50.0),
            tex_coord: Vec4::new(0.0, 0.0, 1.0, 1.0),
            color: Vec4::splat(0.0),
        });

        let image_index = self.core_renderer.get_mut().begin_frame().unwrap();
        let cmd = self.core_renderer.get_mut().get_current_command_buffer();
        let frame_index = self.core_renderer.get().get_current_frame_index();

        renderer2d.draw();

        cmd.blit_image(renderer2d.get_geometry_image(frame_index as usize), self.core_renderer.get_mut().get_swapchain().get_framebuffer(image_index as usize).get_image(0));

        self.core_renderer.get_mut().get_swapchain().get_framebuffer(image_index as usize).get_image(0).transition_layout(ImageLayout::PresentSrc, Some(cmd), AccessFlags::empty(), AccessFlags::empty());

        self.core_renderer.get_mut().end_frame().unwrap();
    }

    fn exiting(&mut self, _window: &mut Window)
    {
        self.device.wait_idle();
    }

    fn resize(&mut self, _window: &mut Window, width: u32, height: u32) {
        self.core_renderer.get_mut().recreate_swapchain(width, height, true, 2); // TODO

        self.renderer2d.get_mut().resize(Extent2D{ width, height });
    }
}
