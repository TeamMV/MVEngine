use crate::render::backend::buffer::MemoryProperties;
use crate::render::backend::command_buffer::{
    CommandBuffer, CommandBufferLevel, MVCommandBufferCreateInfo,
};
use crate::render::backend::device::Device;
use crate::render::backend::framebuffer::Framebuffer;
use crate::render::backend::image::{
    AccessFlags, Image, ImageAspect, ImageFormat, ImageLayout, ImageTiling, ImageType, ImageUsage,
    MVImageCreateInfo,
};
use crate::render::backend::shader::{MVShaderCreateInfo, Shader};
use crate::render::backend::swapchain::{MVSwapchainCreateInfo, Swapchain, SwapchainError};
use crate::render::backend::Extent2D;
use crate::render::window::Window;
use mvutils::remake::Remake;
use shaderc::{OptimizationLevel, ShaderKind, TargetEnv};

pub struct Renderer {
    device: Device,
    command_buffers: Vec<CommandBuffer>,
    current_frame: u32,
    current_image_index: u32,
    swapchain: Remake<Swapchain>,
    empty_texture: Image,
    missing_texture: Image,
    vsync: bool,
    max_frames_in_flight: u32,
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new(window: &Window, device: Device) -> Self {
        let swapchain = Remake::new(Swapchain::new(
            device.clone(),
            MVSwapchainCreateInfo {
                extent: Extent2D {
                    width: window.get_extent().width,
                    height: window.get_extent().height,
                },
                previous: None,
                vsync: window.info.vsync,
                max_frames_in_flight: window.info.max_frames_in_flight,
            },
        ));

        let mut command_buffers = Vec::new();

        for index in 0..swapchain.get_image_count() {
            command_buffers.push(CommandBuffer::new(
                device.clone(),
                MVCommandBufferCreateInfo {
                    level: CommandBufferLevel::Primary,
                    pool: device.get_graphics_command_pool(),
                    label: Some("Main Frame Command Buffer".to_string()),
                },
            ));
        }

        let missing_texture = Image::new(
            device.clone(),
            MVImageCreateInfo {
                size: Extent2D {
                    width: 2,
                    height: 2,
                },
                format: ImageFormat::R8G8B8A8,
                usage: ImageUsage::SAMPLED | ImageUsage::STORAGE,
                memory_properties: MemoryProperties::DEVICE_LOCAL,
                aspect: ImageAspect::COLOR,
                tiling: ImageTiling::Optimal,
                layer_count: 1,
                image_type: ImageType::Image2D,
                cubemap: false,
                memory_usage_flags: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                data: Some(vec![
                    255u8, 0, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 255u8, 0, 255, 255,
                ]),
                label: Some("Default image".to_string()),
            },
        );
        missing_texture.transition_layout(
            ImageLayout::ShaderReadOnlyOptimal,
            None,
            AccessFlags::empty(),
            AccessFlags::empty(),
        );

        let empty_texture = Image::new(
            device.clone(),
            MVImageCreateInfo {
                size: Extent2D {
                    width: 1,
                    height: 1,
                },
                format: ImageFormat::R8G8B8A8,
                usage: ImageUsage::SAMPLED | ImageUsage::STORAGE,
                memory_properties: MemoryProperties::DEVICE_LOCAL,
                aspect: ImageAspect::COLOR,
                tiling: ImageTiling::Optimal,
                layer_count: 1,
                image_type: ImageType::Image2D,
                cubemap: false,
                memory_usage_flags: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
                data: Some(vec![0, 0, 0, 0]),
                label: Some("Default image".to_string()),
            },
        );
        empty_texture.transition_layout(
            ImageLayout::ShaderReadOnlyOptimal,
            None,
            AccessFlags::empty(),
            AccessFlags::empty(),
        );

        Self {
            device,
            command_buffers,
            swapchain,
            empty_texture,
            missing_texture,
            vsync: window.info.vsync,
            current_frame: 0,
            current_image_index: 0,
            max_frames_in_flight: window.info.max_frames_in_flight,
            width: window.get_extent().width,
            height: window.get_extent().height,
        }
    }

    pub fn begin_frame(&mut self) -> Result<u32, SwapchainError> {
        self.current_image_index = self.swapchain.acquire_next_image()?;
        self.get_current_command_buffer().begin();

        Ok(self.current_image_index)
    }

    pub fn end_frame(&mut self) -> Result<(), SwapchainError> {
        let cmd = &self.command_buffers[self.get_current_image_index() as usize];
        let index = self.get_current_image_index();

        cmd.end();
        self.swapchain.submit_command_buffer(cmd, index)?;

        self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;
        Ok(())
    }

    pub fn blit_to_swapchain(&self, image: Image, cmd: &CommandBuffer) {
        let swapchain_image = self
            .swapchain
            .get_current_framebuffer()
            .get_image(0)
            .clone();

        swapchain_image.transition_layout(
            ImageLayout::TransferDstOptimal,
            Some(cmd),
            AccessFlags::empty(),
            AccessFlags::empty(),
        );

        cmd.blit_image(image.clone(), swapchain_image.clone());

        swapchain_image.transition_layout(
            ImageLayout::PresentSrc,
            Some(cmd),
            AccessFlags::empty(),
            AccessFlags::empty(),
        );
    }

    pub fn get_current_framebuffer(&self) -> Framebuffer {
        self.swapchain.get_current_framebuffer()
    }

    pub fn get_current_frame_index(&self) -> u32 {
        self.swapchain.get_current_frame()
    }

    pub fn get_current_image_index(&self) -> u32 {
        self.swapchain.get_current_image_index()
    }

    pub fn get_current_command_buffer(&self) -> &CommandBuffer {
        &self.command_buffers[self.get_current_image_index() as usize]
    }

    pub fn get_swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    pub fn get_swapchain_mut(&mut self) -> &mut Swapchain {
        &mut self.swapchain
    }

    pub fn recreate_swapchain(
        &mut self,
        width: u32,
        height: u32,
        vsync: bool,
        mut max_frames_in_flight: u32,
    ) {
        if self.width == width
            && self.height == height
            && self.vsync == vsync
            && self.max_frames_in_flight == max_frames_in_flight
        {
            return;
        }

        if max_frames_in_flight == 0 {
            max_frames_in_flight = 1;
        }

        self.device.wait_idle();

        self.swapchain.replace(|swapchain| {
            Swapchain::new(
                self.device.clone(),
                MVSwapchainCreateInfo {
                    extent: Extent2D { width, height },
                    previous: Some(swapchain),
                    vsync,
                    max_frames_in_flight,
                },
            )
        });

        if self.max_frames_in_flight != max_frames_in_flight {
            self.command_buffers.clear();
            for index in 0..max_frames_in_flight {
                self.command_buffers.push(CommandBuffer::new(
                    self.device.clone(),
                    MVCommandBufferCreateInfo {
                        level: CommandBufferLevel::Primary,
                        pool: self.device.get_graphics_command_pool(),
                        label: Some("Main Frame Command Buffer".to_string()),
                    },
                ));
            }
        }

        self.vsync = vsync;
        self.max_frames_in_flight = max_frames_in_flight;
        self.width = width;
        self.height = height;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.recreate_swapchain(width, height, self.vsync, self.max_frames_in_flight);
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        self.recreate_swapchain(self.width, self.height, vsync, self.max_frames_in_flight);
    }

    pub fn set_max_frames_in_flight(&mut self, max_frames_in_flight: u32) {
        self.recreate_swapchain(self.width, self.height, self.vsync, max_frames_in_flight);
    }

    pub fn get_swapchain_width(&self) -> u32 {
        self.width
    }

    pub fn get_swapchain_height(&self) -> u32 {
        self.height
    }

    pub fn is_vsync(&self) -> bool {
        self.vsync
    }

    pub fn get_max_frames_in_flight(&self) -> u32 {
        self.max_frames_in_flight
    }

    pub fn compile_shader(
        &self,
        data: &str,
        kind: ShaderKind,
        name: Option<String>,
        defines: &[String],
    ) -> Shader {
        let compiler = shaderc::Compiler::new().unwrap();
        let mut options = shaderc::CompileOptions::new().unwrap();
        options.set_optimization_level(OptimizationLevel::Performance);

        for define in defines {
            options.add_macro_definition(define.as_str(), None);
        }
        options.set_target_env(TargetEnv::Vulkan, ash::vk::API_VERSION_1_2);
        let code = compiler
            .compile_into_spirv(
                data,
                kind,
                name.as_ref().unwrap_or(&"".to_string()),
                "main",
                Some(&options),
            )
            .unwrap()
            .as_binary()
            .to_vec();

        Shader::new(
            self.device.clone(),
            MVShaderCreateInfo {
                stage: kind.into(),
                code,
                label: name,
            },
        )
    }

    pub fn get_empty_texture(&self) -> &Image {
        &self.empty_texture
    }

    pub fn get_missing_texture(&self) -> &Image {
        &self.missing_texture
    }
}
