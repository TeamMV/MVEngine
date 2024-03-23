use crate::render::backend::device::{Device, Extensions, MVDeviceCreateInfo};
use crate::render::backend::swapchain::{MVSwapchainCreateInfo, Swapchain};
use crate::render::backend::{Backend, Extent2D};
use log::LevelFilter;
use mvutils::version::Version;
use shaderc::{EnvVersion, OptimizationLevel, ShaderKind, SpirvVersion, TargetEnv};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::render::backend::pipeline::{Compute, CullMode, Graphics, MVComputePipelineCreateInfo, MVGraphicsPipelineCreateInfo, Pipeline, Topology};
use crate::render::backend::shader::{MVShaderCreateInfo, Shader, ShaderStage};

pub fn run() {
    mvlogger::init(std::io::stdout(), LevelFilter::Trace);

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

    let swapchain = Swapchain::new(
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

    fn compile(str: &str) -> Vec<u32> {
        let compiler = shaderc::Compiler::new().expect("Failed to initialize shader compiler");
        let mut options = shaderc::CompileOptions::new().expect("Failed to initialize shader compiler");
        options.set_target_env(TargetEnv::Vulkan, ash::vk::API_VERSION_1_2);
        options.set_optimization_level(OptimizationLevel::Zero);
        let binary_result = compiler
            .compile_into_spirv(
                str,
                ShaderKind::Compute,
                "shader.glsl",
                "main",
                Some(&options),
            )
            .unwrap();
        binary_result.as_binary().to_vec()
    }

    let shader = include_str!("shader.glsl");
    let bytes = compile(shader);

    let shader = Shader::new(device.clone(), MVShaderCreateInfo {
        stage: ShaderStage::Compute,
        code: bytes,
        label: Some("Debug shader".to_string()),
    });

    // let pipeline = Pipeline::<Graphics>::new(device.clone(), MVGraphicsPipelineCreateInfo {
    //     shaders: vec![shader],
    //     bindings: vec![],
    //     attributes: vec![],
    //     extent: Extent2D { width: 800, height: 600 },
    //     topology: Topology::Triangle,
    //     cull_mode: CullMode::Back,
    //     enable_depth_test: true,
    //     depth_clamp: true,
    //     blending_enable: true,
    //     descriptor_sets: vec![],
    //     push_constants: vec![],
    //     render_pass: swapchain.get_render_pass(),
    //     color_attachments_count: 1,
    //     label: Some("Debug pipeline".to_string()),
    // });

    let pipeline = Pipeline::<Compute>::new(device.clone(), MVComputePipelineCreateInfo {
        shader,
        descriptor_sets: vec![],
        push_constants: vec![],
        label: Some("Debug pipeline".to_string()),
    });

    event_loop
        .run(|event, target| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    WindowEvent::RedrawRequested => {}
                    _ => {}
                }
            }
        })
        .unwrap();
}
