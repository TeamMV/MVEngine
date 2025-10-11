use crate::rendering::api::{RendererCreateInfo, RendererCreateInfoFlags, ShaderFlavor};
use crate::rendering::backend::buffer::MemoryProperties;
use crate::rendering::backend::command_buffer::{
    CommandBuffer, CommandBufferLevel, MVCommandBufferCreateInfo,
};
use crate::rendering::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use crate::rendering::backend::framebuffer::Framebuffer;
use crate::rendering::backend::image::{
    AccessFlags, Image, ImageAspect, ImageFormat, ImageLayout, ImageTiling, ImageType, ImageUsage,
    MVImageCreateInfo,
};
use crate::rendering::backend::shader::Shader;
use crate::rendering::backend::swapchain::{MVSwapchainCreateInfo, Swapchain, SwapchainError};
use crate::rendering::backend::{Backend, Extent2D};
use crate::window::Window;
use gpu_alloc::UsageFlags;
use mvutils::remake::Remake;
use mvutils::version::Version;
use shaderc::ShaderKind;

pub struct XRendererCore {
    device: Device,
    frames_in_flight: u32,
    frame_index: u32,
    image_index: u32,
    swapchain: Remake<Swapchain>,
    command_buffers: Vec<CommandBuffer>,
    is_vsync: bool,
}

impl XRendererCore {
    /// Create a new instance of our amazing new modern fast and performant XRendererCore!
    pub fn new(window: &Window, create_info: RendererCreateInfo) -> Option<Self> {
        let device = Device::new(
            Backend::Vulkan,
            MVDeviceCreateInfo {
                app_name: create_info.app_name,
                app_version: create_info.version,
                engine_name: "MVEngine".to_string(),
                engine_version: Version::new(0, 1, 0, 0),
                green_eco_mode: create_info
                    .flags
                    .contains(RendererCreateInfoFlags::GREEN_ECO_MODE),
                device_extensions: Extensions::DESCRIPTOR_INDEXING,
            },
            window.get_handle(),
        )?;

        let vsync = create_info.flags.contains(RendererCreateInfoFlags::VSYNC);

        let swapchain = Self::create_swapchain(
            device.clone(),
            None,
            window.info.width,
            window.info.height,
            vsync,
            create_info.frames_in_flight,
        );

        let mut command_buffers = Vec::new();

        for i in 0..swapchain.get_image_count() {
            command_buffers.push(CommandBuffer::new(
                device.clone(),
                MVCommandBufferCreateInfo {
                    level: CommandBufferLevel::Primary,
                    pool: device.get_graphics_command_pool(),
                    label: Some(format!("Graphics Command Buffer {i}")),
                },
            ));
        }

        Some(Self {
            device,
            swapchain: Remake::new(swapchain),
            command_buffers,
            is_vsync: vsync,
            frames_in_flight: create_info.frames_in_flight,
            frame_index: 0,
            image_index: 0,
        })
    }

    /// Compile a new shader using shaderc and return the abstract Shader instance.
    pub fn load_shader(
        &self,
        name: &str,
        flavor: ShaderFlavor,
        source: &str,
    ) -> Result<Shader, shaderc::Error> {
        let kind = match flavor {
            ShaderFlavor::Vertex => ShaderKind::Vertex,
            ShaderFlavor::Fragment => ShaderKind::Fragment,
            ShaderFlavor::Compute => ShaderKind::Compute,
        };
        Shader::compile(self.device.clone(), source, kind, stropt(name))
    }

    pub fn load_texture(
        &self,
        name: &str,
        data: &[u8],
        memory_properties: MemoryProperties,
        usage: ImageUsage,
        memory_usage_flags: UsageFlags,
    ) -> Image {
        // TODO: this error we might wanna handle idk
        let image = image::load_from_memory(data).unwrap().into_rgba8();

        Image::new(
            self.device.clone(),
            MVImageCreateInfo {
                size: Extent2D {
                    width: image.width(),
                    height: image.height(),
                },
                format: ImageFormat::R8G8B8A8,
                usage,
                memory_properties,
                aspect: ImageAspect::COLOR,
                tiling: ImageTiling::Optimal,
                layer_count: 1,
                image_type: ImageType::Image2D,
                cubemap: false,
                memory_usage_flags,
                data: Some(image.into_raw()),
                label: stropt(name),
            },
        )
    }

    pub fn create_texture_manually(&self, create_info: MVImageCreateInfo) -> Image {
        Image::new(self.device.clone(), create_info)
    }

    fn create_swapchain(
        device: Device,
        previous: Option<Swapchain>,
        width: u32,
        height: u32,
        vsync: bool,
        max_frames_in_flight: u32,
    ) -> Swapchain {
        let ci = MVSwapchainCreateInfo {
            extent: Extent2D { width, height },
            previous,
            vsync,
            max_frames_in_flight,
        };
        Swapchain::new(device, ci)
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

    pub fn recreate(&mut self, width: u32, height: u32, vsync: bool, frames_in_flight: u32) {
        self.device.wait_idle();

        self.swapchain.replace(|swapchain| {
            Self::create_swapchain(
                self.device.clone(),
                Some(swapchain),
                width,
                height,
                vsync,
                frames_in_flight,
            )
        });

        self.command_buffers.clear();
        for i in 0..self.swapchain.get_image_count() {
            self.command_buffers.push(CommandBuffer::new(
                self.device.clone(),
                MVCommandBufferCreateInfo {
                    level: CommandBufferLevel::Primary,
                    pool: self.device.get_graphics_command_pool(),
                    label: Some(format!("Graphics Command Buffer {i}")),
                },
            ));
        }

        self.frames_in_flight = frames_in_flight;
        self.is_vsync = vsync;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.recreate(width, height, self.is_vsync, self.frames_in_flight);
    }

    pub fn recreate_in_place(&mut self) {
        let Extent2D { width, height } = self.swapchain.get_extent();
        self.recreate(width, height, self.is_vsync, self.frames_in_flight);
    }

    pub fn begin_draw(&mut self) -> Result<u32, (u32, SwapchainError)> {
        match self.swapchain.acquire_next_image() {
            Ok(i) => {
                self.image_index = i;
                self.get_current_command_buffer().begin();
                Ok(self.image_index)
            }
            Err((i, e)) if matches!(e, SwapchainError::Suboptimal) => {
                self.image_index = i;
                self.get_current_command_buffer().begin();
                Err((i, e))
            }
            Err(e) => Err(e),
        }
    }

    pub fn end_draw(&mut self) -> Result<(), SwapchainError> {
        let cmd = &self.command_buffers[self.frame_index as usize];
        let image_idx = self.swapchain.get_current_image_index();
        cmd.end();
        self.frame_index = (self.frame_index + 1) % self.frames_in_flight;
        self.swapchain.submit_command_buffer(cmd, image_idx)
    }

    pub fn get_device(&self) -> Device {
        self.device.clone()
    }

    pub fn get_swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    pub fn get_swapchain_mut(&mut self) -> &mut Swapchain {
        &mut self.swapchain
    }

    pub fn get_current_command_buffer(&self) -> &CommandBuffer {
        &self.command_buffers[self.swapchain.get_current_image_index() as usize]
    }

    pub fn get_current_image_index(&self) -> u32 {
        self.image_index
    }

    pub fn get_current_framebuffer(&self) -> Framebuffer {
        self.swapchain.get_current_framebuffer()
    }
}

fn stropt(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}
