use std::ops::Add;
use std::time::{Duration, Instant, SystemTime};
use crate::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use crate::render::backend::swapchain::{MVSwapchainCreateInfo, Swapchain, SwapchainError};
use crate::render::backend::{Backend, Extent2D};
use log::LevelFilter;
use mvutils::version::Version;
use shaderc::{EnvVersion, OptimizationLevel, ShaderKind, SpirvVersion, TargetEnv};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::render::backend::buffer::{Buffer, BufferUsage, MemoryProperties, MVBufferCreateInfo};
use crate::render::backend::command_buffer::{CommandBuffer, CommandBufferLevel, MVCommandBufferCreateInfo};
use crate::render::backend::framebuffer::ClearColor;
use crate::render::backend::pipeline::{Compute, CullMode, Graphics, MVComputePipelineCreateInfo, MVGraphicsPipelineCreateInfo, Pipeline, Topology};
use crate::render::backend::shader::{MVShaderCreateInfo, Shader, ShaderStage};

pub fn run() {
    mvlogger::init(std::io::stdout(), LevelFilter::Debug);

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_visible(true)
        .with_inner_size(Size::Physical(PhysicalSize {
            width: 800,
            height: 600,
        }))
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
            max_frames_in_flight: 3,
        },
    );

    let cmd_buffers = [(); 3].map(|_| CommandBuffer::new(device.clone(), MVCommandBufferCreateInfo {
        level: CommandBufferLevel::Primary,
        pool: device.get_graphics_command_pool(),
        label: Some("Graphics command buffer".to_string()),
    }));

    fn compile(str: &str, kind: ShaderKind) -> Vec<u32> {
        let compiler = shaderc::Compiler::new().expect("Failed to initialize shader compiler");
        let mut options = shaderc::CompileOptions::new().expect("Failed to initialize shader compiler");
        options.set_target_env(TargetEnv::Vulkan, ash::vk::API_VERSION_1_2);
        options.set_optimization_level(OptimizationLevel::Zero);
        let binary_result = compiler
            .compile_into_spirv(
                str,
                kind,
                "shader.vert",
                "main",
                Some(&options),
            )
            .unwrap();
        binary_result.as_binary().to_vec()
    }

    let vertex_shader = include_str!("shader.vert");
    let v_bytes = compile(vertex_shader, ShaderKind::Vertex);

    let vertex_shader = Shader::new(device.clone(), MVShaderCreateInfo {
        stage: ShaderStage::Vertex,
        code: v_bytes,
        label: Some("Vertex shader".to_string()),
    });

    let fragment_shader = include_str!("shader.frag");
    let f_bytes = compile(fragment_shader, ShaderKind::Fragment);

    let fragment_shader = Shader::new(device.clone(), MVShaderCreateInfo {
        stage: ShaderStage::Fragment,
        code: f_bytes,
        label: Some("Fragment shader".to_string()),
    });

    let pipeline = Pipeline::<Graphics>::new(device.clone(), MVGraphicsPipelineCreateInfo {
        shaders: vec![vertex_shader, fragment_shader],
        bindings: vec![],
        attributes: vec![],
        extent: Extent2D { width: 800, height: 600 },
        topology: Topology::Triangle,
        cull_mode: CullMode::None,
        enable_depth_test: true,
        depth_clamp: true,
        blending_enable: true,
        descriptor_sets: vec![],
        push_constants: vec![],
        framebuffer: swapchain.get_current_framebuffer(),
        color_attachments_count: 1,
        label: Some("Debug pipeline".to_string()),
    });

    let mut frames = 0;
    let mut delta_f = 0.0;
    let time_f = 1000000000.0 / 1000000.0;
    let mut now = SystemTime::now();
    let mut timer = SystemTime::now();

    event_loop
        .run(|event, target| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        //do rendering in here

                        let image_index = swapchain.acquire_next_image().unwrap_or_else(|_| {
                            loop {}
                        });

                        let cmd = &cmd_buffers[swapchain.get_current_frame() as usize];
                        let framebuffer = swapchain.get_current_framebuffer();

                        cmd.begin();

                        pipeline.bind(cmd);

                        framebuffer.begin_render_pass(cmd, &[ClearColor::Color([0.0, 0.0, 0.0, 1.0])], Extent2D {
                            width: 800,
                            height: 600,
                        });

                        cmd.draw(3, 0);

                        framebuffer.end_render_pass(cmd);

                        cmd.end();

                        swapchain.submit_command_buffer(cmd, image_index).unwrap_or_else(|_| {
                            loop {}
                        });
                    }
                    _ => {}
                }
            }
            else if let Event::AboutToWait = event {
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
                    println!("{}", frames);
                    frames = 0;
                    timer = SystemTime::now();
                }
            }
        })
        .unwrap();
}