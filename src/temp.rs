use crate::render::backend::buffer::{Buffer, BufferUsage, MVBufferCreateInfo, MemoryProperties};
use crate::render::backend::command_buffer::{
    CommandBuffer, CommandBufferLevel, MVCommandBufferCreateInfo,
};
use crate::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use crate::render::backend::framebuffer::{ClearColor, Framebuffer, MVFramebufferCreateInfo};
use crate::render::backend::pipeline::{
    AttributeType, Compute, CullMode, Graphics, MVComputePipelineCreateInfo,
    MVGraphicsPipelineCreateInfo, Pipeline, Topology,
};
use crate::render::backend::shader::{MVShaderCreateInfo, Shader, ShaderStage};
use crate::render::backend::swapchain::{MVSwapchainCreateInfo, Swapchain, SwapchainError};
use crate::render::backend::{Backend, Extent2D, Extent3D};
use log::LevelFilter;
use mvutils::version::Version;
use shaderc::{EnvVersion, OptimizationLevel, ShaderKind, SpirvVersion, TargetEnv};
use std::time::{Duration, Instant, SystemTime};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::render::backend::descriptor_set::{DescriptorPool, DescriptorPoolFlags, DescriptorPoolSize, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, MVDescriptorPoolCreateInfo, MVDescriptorSetFromLayoutCreateInfo, MVDescriptorSetLayoutCreateInfo};
use crate::render::backend::image::{ImageFormat, ImageLayout, ImageUsage};
use crate::render::backend::sampler::{Filter, MipmapMode, MVSamplerCreateInfo, Sampler, SamplerAddressMode};

pub fn run() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_visible(true)
        .with_inner_size(Size::Physical(PhysicalSize {
            width: 800,
            height: 600,
        }))
        .with_title("HelloTriangle Application")
        .build(&event_loop)
        .unwrap();

    let device = Device::new(
        Backend::Vulkan,
        MVDeviceCreateInfo {
            app_name: "Test app".to_string(),
            app_version: Version::new(0, 1, 0),
            engine_name: "MVEngine".to_string(),
            engine_version: Version::new(0, 1, 0),
            device_extensions: Extensions::empty(),
        },
        &window,
    );

    let mut swapchain = Swapchain::new(
        device.clone(),
        MVSwapchainCreateInfo {
            extent: Extent2D {
                width: 800,
                height: 600,
            },
            previous: None,
            vsync: false,
            max_frames_in_flight: 1,
        },
    );

    let cmd_buffers = [(); 3].map(|_| {
        CommandBuffer::new(
            device.clone(),
            MVCommandBufferCreateInfo {
                level: CommandBufferLevel::Primary,
                pool: device.get_graphics_command_pool(),
                label: Some("Graphics command buffer".to_string()),
            },
        )
    });

    fn compile(str: &str, kind: ShaderKind) -> Vec<u32> {
        let compiler = shaderc::Compiler::new().expect("Failed to initialize shader compiler");
        let mut options =
            shaderc::CompileOptions::new().expect("Failed to initialize shader compiler");
        options.set_target_env(TargetEnv::Vulkan, ash::vk::API_VERSION_1_2);
        options.set_optimization_level(OptimizationLevel::Zero);
        let binary_result = compiler
            .compile_into_spirv(str, kind, "shader.vert", "main", Some(&options))
            .unwrap();
        binary_result.as_binary().to_vec()
    }

    let vertex_shader = include_str!("shader.vert");
    let v_bytes = compile(vertex_shader, ShaderKind::Vertex);

    let vertex_shader = Shader::new(
        device.clone(),
        MVShaderCreateInfo {
            stage: ShaderStage::Vertex,
            code: v_bytes,
            label: Some("Vertex shader".to_string()),
        },
    );

    let fragment_shader = include_str!("shader.frag");
    let f_bytes = compile(fragment_shader, ShaderKind::Fragment);

    let fragment_shader = Shader::new(
        device.clone(),
        MVShaderCreateInfo {
            stage: ShaderStage::Fragment,
            code: f_bytes,
            label: Some("Fragment shader".to_string()),
        },
    );
    
    let effect_shader = include_str!("pixelate.comp");
    let e_bytes = compile(effect_shader, ShaderKind::Compute);
    
    let effect_shader = Shader::new(device.clone(), MVShaderCreateInfo {
        stage: ShaderStage::Compute,
        code: e_bytes,
        label: Some("Pixelate shader".to_string()),
    });

    let vertex_data = [
        0.0f32, -0.5,
        0.5, 0.5,
        -0.5, 0.5
    ];

    let index_data = [0u32, 1, 2];

    let mut vertex_buffer = Buffer::new(
        device.clone(),
        MVBufferCreateInfo {
            instance_size: (2 + 3) * 4,
            instance_count: 3,
            buffer_usage: BufferUsage::VERTEX_BUFFER,
            memory_properties: MemoryProperties::DEVICE_LOCAL,
            minimum_alignment: 1,
            memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
            label: Some("Vertex buffer".to_string()),
        },
    );

    let byte_data_vertex = unsafe {
        std::slice::from_raw_parts(vertex_data.as_ptr() as *const u8, vertex_data.len() * 4)
    };

    vertex_buffer.write(byte_data_vertex, 0, None);

    let mut index_buffer = Buffer::new(
        device.clone(),
        MVBufferCreateInfo {
            instance_size: 4,
            instance_count: 3,
            buffer_usage: BufferUsage::INDEX_BUFFER,
            memory_properties: MemoryProperties::DEVICE_LOCAL,
            minimum_alignment: 1,
            memory_usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
            label: Some("Index buffer".to_string()),
        },
    );

    let byte_data_index = unsafe {
        std::slice::from_raw_parts(index_data.as_ptr() as *const u8, index_data.len() * 4)
    };

    index_buffer.write(byte_data_index, 0, None);

    let descriptor_set_layout = DescriptorSetLayout::new(device.clone(), MVDescriptorSetLayoutCreateInfo {
        bindings: vec![
            DescriptorSetLayoutBinding {
                index: 0,
                stages: ShaderStage::Vertex,
                ty: DescriptorType::UniformBuffer,
                count: 1,
            }
        ],
        label: Some("Color descriptor set layout".to_string()),
    });

    let mut uniform_buffer = Buffer::new(device.clone(), MVBufferCreateInfo {
        instance_size: 16 * 3,
        instance_count: 1,
        buffer_usage: BufferUsage::UNIFORM_BUFFER,
        memory_properties: MemoryProperties::HOST_VISIBLE | MemoryProperties::HOST_COHERENT,
        minimum_alignment: 1,
        memory_usage: gpu_alloc::UsageFlags::HOST_ACCESS,
        label: Some("Uniform Buffer".to_string()),
    });

    let color = [
        1.0f32, 0.0, 0.0, 1.0,
        0.0, 1.0, 0.0, 1.0,
        0.0, 0.0, 1.0, 1.0
    ];

    let bytes = unsafe { std::slice::from_raw_parts(color.as_ptr() as *const u8, color.len() * 4) };

    uniform_buffer.write(bytes, 0, None);

    let descriptor_pool = DescriptorPool::new(device.clone(), MVDescriptorPoolCreateInfo {
        sizes: vec![DescriptorPoolSize {
            ty: DescriptorType::UniformBuffer,
            count: 1,
        },
                    DescriptorPoolSize {
                        ty: DescriptorType::CombinedImageSampler,
                        count: 1,
                    },
                    DescriptorPoolSize {
                        ty: DescriptorType::StorageImage,
                        count: 1,
                    }
        ],
        max_sets: 1,
        flags: DescriptorPoolFlags::FREE_DESCRIPTOR,
        label: Some("Descriptor pool".to_string()),
    });

    let mut descriptor_set = DescriptorSet::from_layout(device.clone(), MVDescriptorSetFromLayoutCreateInfo {
        pool: descriptor_pool.clone(),
        layout: descriptor_set_layout.clone(),
        label: Some("Color descriptor set".to_string()),
    });

    descriptor_set.add_buffer(0, &uniform_buffer, 0, 16 * 3);

    descriptor_set.build();

    let framebuffer = Framebuffer::new(device.clone(), MVFramebufferCreateInfo {
        attachment_formats: vec![ImageFormat::R32B32G32A32],
        extent: Extent2D { width: 800, height: 600 },
        image_usage_flags: ImageUsage::SAMPLED,
        render_pass_info: None,
        label: Some("Effect framebuffer".to_string()),
    });

    let render_pipeline = Pipeline::<Graphics>::new(
        device.clone(),
        MVGraphicsPipelineCreateInfo {
            shaders: vec![vertex_shader, fragment_shader],
            attributes: vec![AttributeType::Float32x2],
            extent: Extent2D {
                width: 800,
                height: 600,
            },
            topology: Topology::Triangle,
            cull_mode: CullMode::None,
            enable_depth_test: true,
            depth_clamp: false, // feature only
            blending_enable: true,
            descriptor_sets: vec![descriptor_set_layout],
            push_constants: vec![],
            framebuffer: framebuffer.clone(),
            color_attachments_count: 1,
            label: Some("render_pipeline".to_string()),
        },
    );

    let effect_set_layout = DescriptorSetLayout::new(device.clone(), MVDescriptorSetLayoutCreateInfo {
        bindings: vec![
            DescriptorSetLayoutBinding {
                index: 0,
                stages: ShaderStage::Compute,
                ty: DescriptorType::CombinedImageSampler,
                count: 1,
            },
            DescriptorSetLayoutBinding {
                index: 1,
                stages: ShaderStage::Compute,
                ty: DescriptorType::StorageImage,
                count: 1,
            }
        ],
        label: Some("Effect descriptor set layout".to_string()),
    });

    let mut effect_set = vec![DescriptorSet::from_layout(device.clone(), MVDescriptorSetFromLayoutCreateInfo {
        pool: descriptor_pool.clone(),
        layout: effect_set_layout.clone(),
        label: Some("Effect descriptor set".to_string()),
    }),
                              DescriptorSet::from_layout(device.clone(), MVDescriptorSetFromLayoutCreateInfo {
                                  pool: descriptor_pool.clone(),
                                  layout: effect_set_layout.clone(),
                                  label: Some("Effect descriptor set".to_string()),
                              })];

    let sampler = Sampler::new(device.clone(), MVSamplerCreateInfo {
        address_mode: SamplerAddressMode::ClampToEdge,
        filter_mode: Filter::Linear,
        mipmap_mode: MipmapMode::Linear,

        label: Some("Effect sampler".to_string()),
    });

    effect_set[0].add_image(0, framebuffer.get_image(0), &sampler, ImageLayout::ShaderReadOnlyOptimal);
    effect_set[0].add_image(1, swapchain.get_framebuffer(0).get_image(0), &sampler, ImageLayout::General);
    effect_set[0].build();

    effect_set[1].add_image(0, framebuffer.get_image(0), &sampler, ImageLayout::ShaderReadOnlyOptimal);
    effect_set[1].add_image(1, swapchain.get_framebuffer(1).get_image(0), &sampler, ImageLayout::General);
    effect_set[1].build();

    let effect_pipeline = Pipeline::<Compute>::new(
        device.clone(),
        MVComputePipelineCreateInfo {
            shader: effect_shader,
            descriptor_sets: vec![effect_set_layout],
            push_constants: vec![],
            label: Some("Effect descriptor set".to_string()),
        },
    );

    let mut frames = 0;
    let mut delta_f = 0.0;
    let time_f: f32 = 1000000000.0 / 10000.0;
    let mut now = SystemTime::now();
    let mut timer = SystemTime::now();
    let mut time: f32 = 0.0;

    let mut ms_timer = SystemTime::now();

    event_loop
        .run(|event, target| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        //do rendering in here

                        println!("{}", ms_timer.elapsed().unwrap().as_millis_f32());
                        ms_timer = SystemTime::now();


                        let image_index = swapchain.acquire_next_image().unwrap_or_else(|_| {
                            log::error!("Can't resize swapchain!");
                            panic!();
                        });

                        let cmd = &cmd_buffers[swapchain.get_current_frame() as usize];
                        let swapchain_framebuffer = swapchain.get_current_framebuffer();

                        cmd.begin();

                        let r = time.sin();
                        let g = 1.0f32;
                        let b = time.cos();

                        time += 0.02f32;

                        let color = [
                            r, g, b, 1.0,
                            g, b, r, 1.0,
                            b, r, g, 1.0
                        ];

                        let bytes = unsafe { std::slice::from_raw_parts(color.as_ptr() as *const u8, color.len() * 4) };

                        uniform_buffer.write(bytes, 0, Some(cmd));

                        descriptor_set.bind(&cmd, &render_pipeline, 0);

                        render_pipeline.bind(cmd);

                        framebuffer.begin_render_pass(
                            cmd,
                            &[ClearColor::Color([0.0, 0.0, 0.0, 1.0])],
                            Extent2D {
                                width: 800,
                                height: 600,
                            },
                        );

                        cmd.bind_vertex_buffer(&vertex_buffer);
                        cmd.bind_index_buffer(&index_buffer);

                        cmd.draw_indexed(3, 0);

                        framebuffer.end_render_pass(cmd);

                        framebuffer.get_image(0).transition_layout(ImageLayout::ShaderReadOnlyOptimal, Some(cmd), ash::vk::AccessFlags::empty(), ash::vk::AccessFlags::empty());
                        swapchain.get_current_framebuffer().get_image(0).transition_layout(ImageLayout::General, Some(cmd), ash::vk::AccessFlags::empty(), ash::vk::AccessFlags::empty());

                        effect_set[image_index as usize].bind(&cmd, &effect_pipeline, 0);
                        effect_pipeline.bind(cmd);

                        cmd.dispatch(Extent3D {
                            width: 800 / 8 + 1,
                            height: 600 / 8 + 1,
                            depth: 1
                        });

                        swapchain.get_current_framebuffer().get_image(0).transition_layout(ImageLayout::PresentSrc, Some(cmd), ash::vk::AccessFlags::empty(), ash::vk::AccessFlags::empty());

                        cmd.end();

                        swapchain
                            .submit_command_buffer(cmd, image_index)
                            .unwrap_or_else(|_| {
                                log::error!("Can't resize swapchain!");
                                panic!();
                            });
                    }
                    _ => {}
                }
            } else if let Event::AboutToWait = event {
                // if frames > 2 {
                //     panic!()
                // }
                delta_f += now
                    .elapsed()
                    .unwrap_or_else(|e| {
                        panic!(
                            "System clock error: Time elapsed of -{}ns is not valid!",
                            e.duration().as_nanos()
                        )
                    })
                    .as_nanos() as f32
                    / time_f;
                now = SystemTime::now();
                if delta_f >= 1.0 {
                    window.request_redraw();
                    frames += 1;
                    delta_f -= 1.0;
                } else {
                    //target.set_control_flow(ControlFlow::WaitUntil(Instant::now().add(Duration::from_nanos(time_f as u64))));
                    target.set_control_flow(ControlFlow::Poll);
                }
                if timer
                    .elapsed()
                    .unwrap_or_else(|e| {
                        panic!(
                            "System clock error: Time elapsed of -{}ms is not valid!",
                            e.duration().as_millis()
                        )
                    })
                    .as_millis()
                    >= 1000
                {
                    //println!("{}", frames);
                    frames = 0;
                    timer = SystemTime::now();
                }
            } else if let Event::LoopExiting = event {
                device.wait_idle();
            }
        })
        .unwrap();
}
