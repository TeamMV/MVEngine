use crate::gpu::{CameraBuffer, Vertex};
use mvcore::math::vec::Vec2;
use mvcore::render::backend::buffer::{Buffer, BufferUsage, MVBufferCreateInfo, MemoryProperties};
use mvcore::render::backend::descriptor_set::{
    DescriptorPool, DescriptorPoolFlags, DescriptorPoolSize, DescriptorSet,
    DescriptorSetLayoutBinding, DescriptorType, MVDescriptorPoolCreateInfo,
    MVDescriptorSetCreateInfo,
};
use mvcore::render::backend::device::Device;
use mvcore::render::backend::framebuffer::{Framebuffer, MVFramebufferCreateInfo};
use mvcore::render::backend::image::{ImageFormat, ImageUsage};
use mvcore::render::backend::pipeline::{
    AttributeType, CullMode, Graphics, MVGraphicsPipelineCreateInfo, Pipeline, Topology,
};
use mvcore::render::backend::sampler::{
    Filter, MVSamplerCreateInfo, MipmapMode, Sampler, SamplerAddressMode,
};
use mvcore::render::backend::shader::ShaderStage;
use mvcore::render::backend::Extent2D;
use mvcore::render::camera::OrthographicCamera;
use mvcore::render::renderer::Renderer;
use mvutils::unsafe_utils::DangerousCell;
use shaderc::ShaderKind;
use std::sync::Arc;

pub struct Renderer2D {
    device: Device,
    core_renderer: Arc<DangerousCell<Renderer>>,

    descriptor_pool: DescriptorPool,

    camera_buffers: Vec<Buffer>,
    camera_sets: Vec<DescriptorSet>,

    geometry_framebuffers: Vec<Framebuffer>,
    extent: Extent2D,
    nearest_sampler: Sampler,
    linear_sampler: Sampler,
    pipeline: Pipeline<Graphics>,
}

impl Renderer2D {
    pub fn new(device: Device, core_renderer: Arc<DangerousCell<Renderer>>) -> Self {
        let max_textures = 65536;
        let descriptor_pool = DescriptorPool::new(
            device.clone(),
            MVDescriptorPoolCreateInfo {
                sizes: vec![DescriptorPoolSize {
                    ty: DescriptorType::CombinedImageSampler,
                    count: max_textures,
                }],
                max_sets: 1000,
                flags: DescriptorPoolFlags::FREE_DESCRIPTOR,
                label: Some("Renderer2D desc pool".to_string()),
            },
        );

        let extent = core_renderer.get().get_swapchain().get_extent();

        let camera = OrthographicCamera::new(extent.width, extent.height);

        let mut camera_buffers = Vec::new();

        let max_frames = core_renderer.get().get_max_frames_in_flight();
        for _ in 0..max_frames {
            let mut buffer = Buffer::new(
                device.clone(),
                MVBufferCreateInfo {
                    instance_size: size_of::<CameraBuffer>() as u64,
                    instance_count: 1,
                    buffer_usage: BufferUsage::UNIFORM_BUFFER,
                    memory_properties: MemoryProperties::HOST_VISIBLE
                        | MemoryProperties::HOST_COHERENT,
                    minimum_alignment: 1,
                    memory_usage: gpu_alloc::UsageFlags::HOST_ACCESS,
                    label: None,
                },
            );

            let buf = CameraBuffer {
                view_matrix: camera.get_view(),
                proj_matrix: camera.get_projection(),
                screen_size: Vec2::new(extent.width as f32, extent.height as f32),
            };

            let bytes = unsafe {
                std::slice::from_raw_parts(
                    &buf as *const CameraBuffer as *const u8,
                    size_of::<CameraBuffer>(),
                )
            };

            buffer.write(bytes, 0, None);
            camera_buffers.push(buffer);
        }

        let mut camera_sets = Vec::new();

        for index in 0..max_frames {
            let mut camera_set = DescriptorSet::new(
                device.clone(),
                MVDescriptorSetCreateInfo {
                    pool: descriptor_pool.clone(),
                    bindings: vec![DescriptorSetLayoutBinding {
                        index: 0,
                        stages: ShaderStage::Vertex,
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                    }],
                    label: Some("Camera Set".to_string()),
                },
            );

            camera_set.add_buffer(
                0,
                &camera_buffers[index as usize],
                0,
                size_of::<CameraBuffer>() as u64,
            );
            camera_set.build();

            camera_sets.push(camera_set);
        }

        let linear_sampler = Sampler::new(
            device.clone(),
            MVSamplerCreateInfo {
                address_mode: SamplerAddressMode::ClampToEdge,
                filter_mode: Filter::Linear,
                mipmap_mode: MipmapMode::Linear,
                anisotropy: false,
                label: None,
            },
        );

        let nearest_sampler = Sampler::new(
            device.clone(),
            MVSamplerCreateInfo {
                address_mode: SamplerAddressMode::ClampToEdge,
                filter_mode: Filter::Nearest,
                mipmap_mode: MipmapMode::Nearest,
                anisotropy: false,
                label: None,
            },
        );

        let mut geometry_framebuffers = Vec::new();
        for _ in 0..max_frames {
            let framebuffer = Framebuffer::new(
                device.clone(),
                MVFramebufferCreateInfo {
                    attachment_formats: vec![ImageFormat::R16G16B16A16, ImageFormat::D16],
                    extent,
                    image_usage_flags: ImageUsage::TRANSFER_SRC,
                    render_pass_info: None,
                    label: Some("Geometry Framebuffer".to_string()),
                },
            );

            geometry_framebuffers.push(framebuffer);
        }

        let vertex_shader = core_renderer.get().compile_shader(
            include_str!("shaders/default.vert"),
            ShaderKind::Fragment,
            Some("Renderer2D Shared Vertex Shader".to_string()),
            &[],
        );

        let fragment_shader = core_renderer.get().compile_shader(
            include_str!("shaders/default.frag"),
            ShaderKind::Fragment,
            Some("Renderer2D Shared Fragment Shader".to_string()),
            &[],
        );

        let pipeline = Pipeline::<Graphics>::new(
            device.clone(),
            MVGraphicsPipelineCreateInfo {
                shaders: vec![vertex_shader, fragment_shader.clone()],
                attributes: Vertex::get_attrib_desc(),
                extent,
                topology: Topology::Triangle,
                cull_mode: CullMode::None,
                enable_depth_test: true,
                depth_clamp: false,
                blending_enable: true,
                descriptor_sets: vec![camera_sets[0].get_layout()],
                push_constants: vec![],
                framebuffer: geometry_framebuffers[0].clone(),
                color_attachments_count: 1,
                label: Some("Renderer2D Rectangle Pipeline".to_string()),
            },
        );

        Self {
            device,
            core_renderer,
            descriptor_pool,
            camera_sets,
            camera_buffers,
            linear_sampler,
            nearest_sampler,
            geometry_framebuffers,
            extent,
            pipeline,
        }
    }

    pub fn get_extent(&self) -> &Extent2D {
        &self.extent
    }
}
