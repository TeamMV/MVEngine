use crate::render::backend::command_buffer::{
    CommandBuffer, CommandBufferLevel, MVCommandBufferCreateInfo,
};
use crate::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use crate::render::backend::framebuffer::Framebuffer;
use crate::render::backend::swapchain::{MVSwapchainCreateInfo, Swapchain, SwapchainError};
use crate::render::backend::{Backend, Extent2D};
use crate::render::window::WindowCreateInfo;
use mvutils::version::Version;
use std::sync::Arc;

pub(crate) struct State {
    device: Device,
    swapchain: Option<Swapchain>,
    command_buffers: Vec<CommandBuffer>,
}

impl State {
    pub(crate) fn new(info: &WindowCreateInfo, window: &winit::window::Window) -> Self {
        let device = Device::new(
            Backend::Vulkan,
            MVDeviceCreateInfo {
                app_name: info.title.to_string(),
                app_version: Version::new(0, 1, 0),
                engine_name: "MVEngine".to_string(),
                engine_version: Version::new(0, 1, 0),
                device_extensions: Extensions::empty(),
            },
            window,
        );

        let swapchain = Swapchain::new(
            device.clone(),
            MVSwapchainCreateInfo {
                extent: Extent2D {
                    width: info.width,
                    height: info.height,
                },
                previous: None,
                vsync: false,
                max_frames_in_flight: 2,
            },
        );

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

        State {
            device,
            swapchain: Some(swapchain),
            command_buffers,
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.device.wait_idle();

        self.swapchain = Some(Swapchain::new(
            self.device.clone(),
            MVSwapchainCreateInfo {
                extent: Extent2D { width, height },
                previous: self.swapchain.take(),
                vsync: false,
                max_frames_in_flight: 2,
            },
        ));
    }

    pub(crate) fn get_device(&self) -> Device {
        self.device.clone()
    }

    pub(crate) fn get_current_framebuffer(&self) -> Framebuffer {
        self.swapchain
            .as_ref()
            .expect("Swapchain should never be None")
            .get_current_framebuffer()
    }

    pub(crate) fn get_current_frame_index(&self) -> u32 {
        self.swapchain
            .as_ref()
            .expect("Swapchain should never be None")
            .get_current_frame()
            .clone()
    }

    pub(crate) fn get_current_image_index(&self) -> u32 {
        self.swapchain
            .as_ref()
            .expect("Swapchain should never be None")
            .get_current_image_index()
            .clone()
    }

    pub(crate) fn get_current_command_buffer(&self) -> &CommandBuffer {
        &self.command_buffers[self.get_current_image_index() as usize]
    }

    pub(crate) fn begin_frame(&mut self) -> Result<u32, SwapchainError> {
        let image = self
            .swapchain
            .as_mut()
            .expect("Swapchain should never be None")
            .acquire_next_image()?;
        self.get_current_command_buffer().begin();
        Ok(image)
    }

    pub(crate) fn end_frame(&mut self) -> Result<(), SwapchainError> {
        let cmd = &self.command_buffers[self.get_current_image_index() as usize];
        let index = self.get_current_image_index();

        cmd.end();
        self.swapchain
            .as_mut()
            .expect("Swapchain should never be None")
            .submit_command_buffer(cmd, index)?;
        Ok(())
    }

    pub(crate) fn get_swapchain(&self) -> &Swapchain {
        self.swapchain
            .as_ref()
            .expect("Swapchain should never be None")
    }
}
