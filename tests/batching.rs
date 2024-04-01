use std::f32::consts::PI;
use glam::Mat4;
use log::LevelFilter;
use mvutils::once::CreateOnce;
use shaderc::ShaderKind;
use mvcore::render::window::{Window, WindowCreateInfo};
use mvcore::render::ApplicationLoopCallbacks;
use mvcore::render::backend::{Backend, Extent2D};
use mvcore::render::backend::buffer::{Buffer, BufferUsage, MemoryProperties, MVBufferCreateInfo};
use mvcore::render::backend::descriptor_set::{DescriptorPool, DescriptorPoolFlags, DescriptorPoolSize, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, MVDescriptorPoolCreateInfo, MVDescriptorSetCreateInfo, MVDescriptorSetLayoutCreateInfo};
use mvcore::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use mvcore::render::backend::framebuffer::ClearColor;
use mvcore::render::backend::pipeline::{AttributeType, CullMode, Graphics, MVGraphicsPipelineCreateInfo, Pipeline, Topology};
use mvcore::render::backend::shader::ShaderStage;
use mvcore::render::camera2d::Camera2D;
use mvcore::render::mesh::Mesh;
use mvcore::render::renderer::Renderer;

fn main() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    let window = Window::new(WindowCreateInfo {
        width: 1600,
        height: 900,
        title: "MVCore".to_string(),
        fullscreen: false,
        decorated: true,
        resizable: true,
        transparent: false,
        theme: None,
        vsync: false,
        max_frames_in_flight: 2,
        fps: 9990,
        ups: 20,
    });

    window.run::<ApplicationLoop>();
}

struct ApplicationLoop {
    device: Device,
    mesh: Mesh,
    pipeline: Pipeline,
    renderer: Renderer,
    camera_set: DescriptorSet,
    pool: DescriptorPool,
    camera_buffer: Buffer,
    vertices: Vec<Vertex>,
    quad_rotation: f32
}

#[repr(C)]
struct Vertex {
    position: glam::Vec3,
    tex_coord: glam::Vec2
}

impl Vertex {
    pub fn to_bytes(vertices: &[Vertex]) -> &[u8] {
        unsafe { std::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * std::mem::size_of::<Vertex>()) }
    }

    pub fn get_attribute_description() -> Vec<AttributeType> {
        vec![AttributeType::Float32x3, AttributeType::Float32x2]
    }
}

pub fn create_quad_vertices(position: glam::Vec3, rotation: f32, size: f32) -> Vec<Vertex> {
    let translation = Mat4::from_translation(position);

    // Rotation (for example, rotating around the Y axis by 90 degrees)
    let rotation = Mat4::from_rotation_z(rotation * PI / 180.0);

    // Scaling (uniform scaling by a factor of 2)
    let scale = Mat4::from_scale(glam::Vec3::splat(size));

    // Combining translation, rotation, and scaling to create the model matrix
    let model_matrix = translation * rotation * scale;

    vec![
        Vertex{position: model_matrix.transform_point3(glam::Vec3{x: -1.0f32, y:  1.0, z: 0.0}), tex_coord: glam::Vec2{x: 0.0f32, y: 0.0}}, // 0
        Vertex{position: model_matrix.transform_point3(glam::Vec3{x: -1.0f32, y: -1.0, z: 0.0}), tex_coord: glam::Vec2{x: 0.0f32, y: 0.0}}, // 1
        Vertex{position: model_matrix.transform_point3(glam::Vec3{x:  1.0f32, y: -1.0, z: 0.0}), tex_coord: glam::Vec2{x: 0.0f32, y: 0.0}}, // 2

        Vertex{position: model_matrix.transform_point3(glam::Vec3{x: -1.0f32, y:  1.0, z: 0.0}), tex_coord: glam::Vec2{x: 0.0f32, y: 0.0}}, // 0
        Vertex{position: model_matrix.transform_point3(glam::Vec3{x:  1.0f32, y: -1.0, z: 0.0}), tex_coord: glam::Vec2{x: 0.0f32, y: 0.0}}, // 2
        Vertex{position: model_matrix.transform_point3(glam::Vec3{x:  1.0f32, y:  1.0, z: 0.0}), tex_coord: glam::Vec2{x: 0.0f32, y: 0.0}}, // 3
    ]
}

impl ApplicationLoopCallbacks for ApplicationLoop {
    fn new(window: &mut Window) -> Self {
        let device = Device::new(
            Backend::Vulkan, MVDeviceCreateInfo {
                app_name: "".to_string(),
                app_version: Default::default(),
                engine_name: "".to_string(),
                engine_version: Default::default(),
                device_extensions: Extensions::empty(),
            },
            window.get_handle()
        );

        let renderer = Renderer::new(window, device.clone());

        let descriptor_pool = DescriptorPool::new(device.clone(), MVDescriptorPoolCreateInfo {
            sizes: vec![
                DescriptorPoolSize{ ty: DescriptorType::CombinedImageSampler, count: 1000 },
                DescriptorPoolSize{ ty: DescriptorType::StorageImage, count: 1000 },
                DescriptorPoolSize{ ty: DescriptorType::UniformBuffer, count: 1000 },
                DescriptorPoolSize{ ty: DescriptorType::StorageBuffer, count: 1000 },
            ],
            max_sets: 1000,
            flags: DescriptorPoolFlags::FREE_DESCRIPTOR,
            label: Some("Main Descriptor Set".to_string()),
        });

        let camera = Camera2D::new(renderer.get_swapchain().get_extent().width, renderer.get_swapchain().get_extent().height);

        let mut camera_buffer = Buffer::new(device.clone(), MVBufferCreateInfo {
            instance_size: 128,
            instance_count: 1,
            buffer_usage: BufferUsage::UNIFORM_BUFFER,
            memory_properties: MemoryProperties::HOST_VISIBLE | MemoryProperties::HOST_COHERENT,
            minimum_alignment: 1,
            memory_usage: gpu_alloc::UsageFlags::HOST_ACCESS,
            label: Some("Camera Uniform Buffer".to_string()),
        });

        let matrices = [camera.get_view(), camera.get_projection()];
        let bytes = unsafe { std::slice::from_raw_parts(matrices.as_ptr() as *const u8, 128)};

        camera_buffer.write(bytes, 0, None);

        let camera_set_layout = DescriptorSetLayout::new(device.clone(), MVDescriptorSetLayoutCreateInfo {
            bindings: vec![DescriptorSetLayoutBinding{
                index: 0,
                stages: ShaderStage::Vertex,
                ty: DescriptorType::UniformBuffer,
                count: 1,
            }],
            label: None,
        });

        let mut camera_set = DescriptorSet::new(device.clone(), MVDescriptorSetCreateInfo {
            pool: descriptor_pool.clone(),
            bindings: vec![
                DescriptorSetLayoutBinding{
                    index: 0,
                    stages: ShaderStage::Vertex,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                }
            ],
            label: Some("Camera Set".to_string()),
        });

        camera_set.add_buffer(0, &camera_buffer, 0, 128);
        camera_set.build();

        let vertices = create_quad_vertices(glam::Vec3::new(100.0, -100.0, 0.0), 45.0, 10.0);

        let mesh = Mesh::new(device.clone(), Vertex::to_bytes(&vertices), None, Some("Test mesh".to_string()));

        let vertex_shader = renderer.compile_shader(include_str!("../src/render/shaders/2d/default.vert"), ShaderStage::Vertex, ShaderKind::Vertex);
        let fragment_shader = renderer.compile_shader(include_str!("../src/render/shaders/2d/default.frag"), ShaderStage::Fragment, ShaderKind::Fragment);

        let test_pipeline = Pipeline::<Graphics>::new(device.clone(), MVGraphicsPipelineCreateInfo {
            shaders: vec![vertex_shader, fragment_shader],
            attributes: Vertex::get_attribute_description(),
            extent: renderer.get_swapchain().get_extent(),
            topology: Topology::Triangle,
            cull_mode: CullMode::None,
            enable_depth_test: false,
            depth_clamp: false,
            blending_enable: false,
            descriptor_sets: vec![camera_set_layout.clone()],
            push_constants: vec![],
            framebuffer: renderer.get_swapchain().get_framebuffer(0),
            color_attachments_count: 1,
            label: None,
        });

        ApplicationLoop {
            mesh,
            device,
            pipeline: test_pipeline,
            renderer,
            camera_set,
            pool: descriptor_pool,
            vertices: Vec::new(),
            camera_buffer,
            quad_rotation: 0.0
        }
    }

    fn update(&mut self, window: &mut Window, delta_t: f64) {
    }

    fn draw(&mut self, window: &mut Window, delta_t: f64) {
        println!("ms: {}, fps: {}", delta_t * 1000.0, 1.0 / delta_t);

        self.vertices.clear();

        self.quad_rotation += delta_t as f32 * 50.0;

        for x in 0..100 {
            for y in 0..100 {
                self.vertices.extend(create_quad_vertices(glam::Vec3::new(x as f32 * 20.0 + 20.0, -y as f32 * 20.0 - 20.0, 0.0), self.quad_rotation, 7.5));
            }
        }

        let bytes = Vertex::to_bytes(&self.vertices);

        self.mesh.update_vertex_buffer(bytes);

        let image_index = self.renderer.begin_frame().unwrap();

        let swapchain = self.renderer.get_swapchain_mut();
        let extent = swapchain.get_extent();
        let framebuffer = swapchain.get_current_framebuffer();
        let cmd = self.renderer.get_current_command_buffer();

        framebuffer.begin_render_pass(cmd, &[ClearColor::Color([0.0, 0.0, 0.0, 1.0])], extent);

        self.pipeline.bind(cmd);

        self.camera_set.bind(cmd, &self.pipeline, 0);
        self.mesh.draw(cmd);

        framebuffer.end_render_pass(cmd);

        self.renderer.end_frame().unwrap();
    }

    fn exiting(&mut self, window: &mut Window) {
        println!("exit");
    }

    fn resize(&mut self, window: &mut Window, width: u32, height: u32) {
        println!("resize {width} {height}");
    }
}
