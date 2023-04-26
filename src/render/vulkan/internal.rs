use std::default::Default;
use std::ffi::c_void;
use std::io::Read;
use std::ptr::null_mut;
use std::sync::Arc;
use glam::Mat4;
use glfw::ffi::{glfwCreateWindowSurface, glfwVulkanSupported, GLFWwindow};
#[cfg(all(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"), not(feature = "wayland")))]
use glfw::ffi::{glfwGetX11Window, glfwGetX11Display};
#[cfg(all(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"), feature = "wayland"))]
use glfw::ffi::{glfwGetWaylandWindow, glfwGetWaylandDisplay};
#[cfg(target_family = "windows")]
use glfw::ffi::glfwGetWin32Window;
#[cfg(target_os = "macos")]
use glfw::ffi::glfwGetCocoaWindow;
use glsl_to_spirv::ShaderType;
use regex::internal::Input;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::{Version, VulkanLibrary, VulkanObject};
use vulkano::buffer::{Buffer, BufferContents, BufferContentsLayout, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents, PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract};
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::image::sys::Image;
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, GenericMemoryAllocatorCreateInfo, MemoryAllocator, MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle, RawDisplayHandle};
use ash::vk;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use crate::ApplicationInfo;
use crate::render::{VK_EFFECT_VERT, VK_EMPTY_EFFECT_FRAG};
use crate::render::vulkan::vulkan::VulkanShader;

const DEVICE_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    khr_swapchain: true,
    ..DeviceExtensions::empty()
};

fn raw_window_handle(win: *mut GLFWwindow) -> RawWindowHandle {
    #[cfg(target_family = "windows")]
    {
        use raw_window_handle::Win32WindowHandle;
        let (hwnd, hinstance) = unsafe {
            let hwnd = glfwGetWin32Window(win);
            let hinstance = winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null());
            (hwnd, hinstance as _)
        };
        let mut handle = Win32WindowHandle::empty();
        handle.hwnd = hwnd;
        handle.hinstance = hinstance;
        RawWindowHandle::Win32(handle)
    }
    #[cfg(all(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"), not(feature = "wayland")))]
    {
        use raw_window_handle::XlibWindowHandle;
        let mut handle = XlibWindowHandle::empty();
        handle.window = unsafe { glfwGetX11Window(win) as std::os::raw::c_ulong };
        RawWindowHandle::Xlib(handle)
    }
    #[cfg(all(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"), feature = "wayland"))]
    {
        use raw_window_handle::WaylandWindowHandle;
        let mut handle = WaylandWindowHandle::empty();
        handle.surface = unsafe { glfwGetWaylandWindow(win) };
        RawWindowHandle::Wayland(handle)
    }
    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::AppKitWindowHandle;
        let (ns_window, ns_view) = unsafe {
            let ns_window: *mut objc::runtime::Object =
                glfwGetCocoaWindow(win) as *mut _;
            let ns_view: *mut objc::runtime::Object = objc::msg_send![ns_window, contentView];
            assert_ne!(ns_view, std::ptr::null_mut());
            (
                ns_window as *mut std::ffi::c_void,
                ns_view as *mut std::ffi::c_void,
            )
        };
        let mut handle = AppKitWindowHandle::empty();
        handle.ns_window = ns_window;
        handle.ns_view = ns_view;
        RawWindowHandle::AppKit(handle)
    }
    #[cfg(target_os = "emscripten")]
    {
        let _ = context;
        let mut wh = raw_window_handle::WebWindowHandle::empty();
        wh.id = 1;
        RawWindowHandle::Web(wh)
    }
}

fn raw_display_handle() -> RawDisplayHandle {
    #[cfg(target_family = "windows")]
    {
        use raw_window_handle::WindowsDisplayHandle;
        RawDisplayHandle::Windows(WindowsDisplayHandle::empty())
    }
    #[cfg(all(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"), not(feature = "wayland")))]
    {
        use raw_window_handle::XlibDisplayHandle;
        let mut handle = XlibDisplayHandle::empty();
        handle.display = unsafe { glfwGetX11Display() };
        RawDisplayHandle::Xlib(handle)
    }
    #[cfg(all(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"), feature = "wayland"))]
    {
        use raw_window_handle::WaylandDisplayHandle;
        let mut handle = WaylandDisplayHandle::empty();
        handle.display = unsafe { glfwGetWaylandDisplay() };
        RawDisplayHandle::Wayland(handle)
    }
    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::AppKitDisplayHandle;
        RawDisplayHandle::AppKit(AppKitDisplayHandle::empty())
    }
    #[cfg(target_os = "emscripten")]
    {
        RawDisplayHandle::Web(raw_window_handle::WebDisplayHandle::empty())
    }
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub(crate) struct InternalVertex2D {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32_SFLOAT)]
    rotation: f32,
    #[format(R32G32_SFLOAT)]
    rotation_origin: [f32; 2],
    #[format(R32G32B32A32_SFLOAT)]
    color: [f32; 4],
    #[format(R32G32_SFLOAT)]
    uv:[f32; 2],
    #[format(R32_SFLOAT)]
    tex: f32,
    #[format(R32G32B32A32_SFLOAT)]
    canvas_coords: [f32; 4],
    #[format(R32G32_SFLOAT)]
    canvas_data: [f32; 2],
    #[format(R32_SFLOAT)]
    use_camera: f32
}

struct VkWindow {
    window: *mut GLFWwindow
}

unsafe impl HasRawWindowHandle for VkWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        raw_window_handle(self.window)
    }
}

unsafe impl HasRawDisplayHandle for VkWindow {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        raw_display_handle()
    }
}

pub(crate) struct Vulkan {
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<SwapchainImage>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    graphics_pipeline_2d: Arc<GraphicsPipeline>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    memory_allocator: StandardMemoryAllocator,
}

impl Vulkan {
    pub(crate) unsafe fn init(info: &ApplicationInfo, window: *mut GLFWwindow, width: u32, height: u32) -> Result<Vulkan, ()> {
        if glfwVulkanSupported() == glfw::ffi::FALSE {
            return Err(());
        }

        let library = VulkanLibrary::new().map_err(|_| ())?;
        let mut instance_info = InstanceCreateInfo::default();

        instance_info.application_name = Some(info.name.clone());
        instance_info.application_version = Version::from(info.version.as_vulkan_version());
        instance_info.engine_name = Some("MVCore".to_string());
        instance_info.engine_version = Version::from(mvutils::version::Version::parse(env!("CARGO_PKG_VERSION")).unwrap().as_vulkan_version());
        instance_info.enabled_extensions = vulkano_win::required_extensions(&library);

        let instance = Instance::new(library, instance_info).map_err(|_| ())?;

        //let raw_surface: *mut vk::SurfaceKHR = null_mut();
        //if glfwCreateWindowSurface(instance.handle(), window, null_mut(), raw_surface) != vk::Result::SUCCESS {
        //    return Err(());
        //}

        let handle = VkWindow { window };

        let surface = vulkano_win::create_surface_from_handle_ref(&handle, instance.clone()).map_err(|_| ())?;

        let mut devices = instance.enumerate_physical_devices().map_err(|_| ())?.collect::<Vec<_>>();
        if devices.len() == 0 {
            return Err(());
        }
        let (physical_device, queue_family_index) = Self::choose_physical_device(devices, surface.clone()).ok_or(())?;

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: DEVICE_EXTENSIONS,
                ..Default::default()
            },
        ).map_err(|_| ())?;

        let queue = queues.next().ok_or(())?;

        let caps = physical_device.surface_capabilities(&surface, Default::default()).map_err(|_| ())?;

        let composite_alpha = caps.supported_composite_alpha.into_iter().next().ok_or(())?;
        let image_format = Some(physical_device.surface_formats(&surface, Default::default()).map_err(|_| ())?[0].0);

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: [width, height],
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            },
        ).map_err(|_| ())?;

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(), // set the format the same as the swapchain
                    samples: 1,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        ).map_err(|_| ())?;

        let framebuffers = images.iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                )
            }).flatten().collect::<Vec<_>>();

        if framebuffers.len() == 0 {
            return Err(());
        }

        let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

        let vs = ShaderModule::from_bytes(
            device.clone(),
            &glsl_to_spirv::compile(VK_EFFECT_VERT, ShaderType::Vertex)
                .expect("Error making default shaders")
                .bytes()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()).map_err(|_| ())?;

        let fs = ShaderModule::from_bytes(
            device.clone(),
            &glsl_to_spirv::compile(VK_EMPTY_EFFECT_FRAG, ShaderType::Fragment)
                .expect("Error making default shaders")
                .bytes()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()).map_err(|_| ())?;

        let graphics_pipeline = Self::generate_graphics_pipeline::<InternalVertex2D>(
            device.clone(),
            width as f32,
            height as f32,
            render_pass.clone(),
            vs.clone(),
            fs.clone()
        ).ok_or(())?;

        let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());

        Ok(Vulkan {
            instance,
            physical_device,
            device,
            queue,
            surface,
            swapchain,
            images,
            render_pass,
            framebuffers,
            graphics_pipeline_2d: graphics_pipeline,
            vs,
            fs,
            memory_allocator,
            command_buffer_allocator,
        })
    }

    fn choose_physical_device(devices: Vec<Arc<PhysicalDevice>>, surface: Arc<Surface>) -> Option<(Arc<PhysicalDevice>, u32)> {
        devices.into_iter()
            .filter_map(|device| {
                if !device.supported_extensions().contains(&DEVICE_EXTENSIONS) {
                    return None;
                }
                return device.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && device.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|q| (device, q as u32));
            })
            .max_by_key(|(device, _)| {
            let mut score = match device.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 32000,
                PhysicalDeviceType::IntegratedGpu => 16000,
                PhysicalDeviceType::VirtualGpu => 8000,
                _ => 0
            };

            score += device.properties().max_image_dimension2_d;
            score += device.properties().max_image_dimension3_d;

            score
        })
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let caps = self.physical_device.surface_capabilities(&self.surface, Default::default()).expect("Error resizing!");

        let composite_alpha = caps.supported_composite_alpha.into_iter().next().expect("Error resizing!");
        let image_format = Some(self.physical_device.surface_formats(&self.surface, Default::default()).expect("Error resizing!")[0].0);

        self.swapchain.recreate(SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1,
            image_format,
            image_extent: [width, height],
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha,
            ..Default::default()
        }).expect("Error recreating swapchain!");

        self.graphics_pipeline_2d = Self::generate_graphics_pipeline::<InternalVertex2D>(
            self.device.clone(),
            width as f32,
            height as f32,
            self.render_pass.clone(),
            self.vs.clone(),
            self.fs.clone()
        ).expect("Error creating graphics pipeline!");
    }

    fn generate_graphics_pipeline<T: Vertex>(device: Arc<Device>, width: f32, height: f32, render_pass: Arc<RenderPass>, vs: Arc<ShaderModule>, fs: Arc<ShaderModule>) -> Option<Arc<GraphicsPipeline>> {
        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [width, height],
            depth_range: 0.0..1.0,
        };

        GraphicsPipeline::start()
            .vertex_input_state(T::per_vertex())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(device).ok()
    }

    fn get_command_buffers<T: Vertex>(allocator: &StandardCommandBufferAllocator, queue: Arc<Queue>, pipeline: Arc<GraphicsPipeline>, framebuffers: Vec<Arc<Framebuffer>>, vertex_buffer: Arc<Buffer>, index_buffer: Arc<Buffer>, draw: usize) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
        framebuffers.iter()
            .map(|framebuffer| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                ).unwrap();

                builder.begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassContents::Inline,
                    )
                    .unwrap()
                    .bind_pipeline_graphics(pipeline.clone())
                    .bind_vertex_buffers(0, Subbuffer::from(vertex_buffer.clone()))
                    .bind_index_buffer(Subbuffer::from(index_buffer.clone()))
                    .draw(draw as u32, 1, 0, 0)
                    .unwrap()
                    .end_render_pass()
                    .unwrap();

                Arc::new(builder.build().unwrap())
            }).collect()
    }

    pub(crate) fn gen_command_buffer_2d(&self, vertices: Arc<Buffer>, indices: Arc<Buffer>, len: usize) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
        Self::get_command_buffers::<InternalVertex2D>(
            &self.command_buffer_allocator,
            self.queue.clone(),
            self.graphics_pipeline_2d.clone(),
            self.framebuffers.clone(),
            vertices,
            indices,
            len
        )
    }

    pub(crate) fn run(&self, commands: Vec<Arc<PrimaryAutoCommandBuffer>>) {
        for cmd in commands.into_iter() {
            cmd.execute(self.queue.clone()).expect("Failed to run draw!");
        }
    }

    pub(crate) fn buffer_vertices(&self, vertices: &[f32]) -> Arc<Buffer> {
        Buffer::from_iter(
            &self.memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            vertices.iter().cloned(),
        ).expect("Failed to create vulkan buffer.").buffer().clone()
    }

    pub(crate) fn buffer_indices(&self, indices: &[u32]) -> Arc<Buffer> {
        Buffer::from_iter(
            &self.memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            indices.iter().cloned(),
        ).expect("Failed to create vulkan buffer.").buffer().clone()
    }

    pub(crate) fn buffer_uniform<T: BufferContents>(&self, data: T) -> Arc<Buffer> {
        Buffer::from_data(
            &self.memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            data
        ).expect("Failed to create vulkan buffer.").buffer().clone()
    }

    pub(crate) fn set_shader_2d(&mut self, shader: &mut VulkanShader, width: u32, height: u32) {
        unsafe {
            shader.vk_make(self.device.clone());

            self.vs = shader.vk_get(0);
            self.fs = shader.vk_get(1);

            self.graphics_pipeline_2d = Self::generate_graphics_pipeline::<InternalVertex2D>(
                self.device.clone(),
                width as f32,
                height as f32,
                self.render_pass.clone(),
                self.vs.clone(),
                self.fs.clone()
            ).expect("Error creating graphics pipeline!");
        }
    }

    pub(crate) fn set_shader_3d(&mut self, shader: &mut VulkanShader, width: u32, height: u32) {

    }
}