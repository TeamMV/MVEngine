use crate::render::backend::device::{Extensions, MVDeviceCreateInfo};
use hashbrown::HashSet;
use mvutils::version::Version;
use shaderc::EnvVersion::Vulkan1_2;
use std::error::Error;
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::Arc;
use winit::raw_window_handle;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub(crate) struct VkDevice {
    entry: ash::Entry,
    instance: ash::Instance,
    physical_device: ash::vk::PhysicalDevice,
    surface_extension: ash::extensions::khr::Surface,
    swapchain_extension: ash::extensions::khr::Swapchain,
    surface: ash::vk::SurfaceKHR,
    properties: ash::vk::PhysicalDeviceProperties2,
    device: ash::Device,
    command_pools: CommandPools,
    queues: Queues,

    available_present_modes: Vec<ash::vk::PresentModeKHR>,

    vsync_present_mode: ash::vk::PresentModeKHR,
    no_vsync_present_mode: ash::vk::PresentModeKHR,

    #[cfg(debug_assertions)]
    debug_messenger: ash::vk::DebugUtilsMessengerEXT,
    #[cfg(debug_assertions)]
    debug_utils: ash::extensions::ext::DebugUtils,
}

pub(crate) struct CreateInfo {
    // Instance Info
    app_version: Version,
    app_name: CString,
    engine_name: CString,
    engine_version: Version,

    // Extensions
    device_extensions: Extensions,
}

impl From<MVDeviceCreateInfo> for CreateInfo {
    fn from(value: MVDeviceCreateInfo) -> Self {
        fn to_ascii_cstring(input: String) -> CString {
            let ascii = input.chars().filter(|c| c.is_ascii()).collect::<String>();

            // Create a CString from the ASCII bytes
            CString::new(ascii.as_bytes()).expect("CString::new failed")
        }

        CreateInfo {
            app_version: value.app_version,
            app_name: to_ascii_cstring(value.app_name),
            engine_name: to_ascii_cstring(value.engine_name),
            engine_version: value.engine_version,
            device_extensions: value.device_extensions,
        }
    }
}

struct Queues {
    graphics_queue: ash::vk::Queue,
    compute_queue: ash::vk::Queue,
    present_queue: ash::vk::Queue,
}

struct CommandPools {
    graphics_command_pool: ash::vk::CommandPool,
    compute_command_pool: ash::vk::CommandPool,
}

pub(crate) struct QueueIndices {
    pub graphics_queue_index: Option<u32>,
    pub compute_queue_index: Option<u32>,
    pub present_queue_index: Option<u32>,
}

impl QueueIndices {
    fn is_complete(&self) -> bool {
        self.graphics_queue_index.is_some()
            && self.compute_queue_index.is_some()
            && self.present_queue_index.is_some()
    }

    fn create() -> Self {
        QueueIndices {
            graphics_queue_index: None,
            compute_queue_index: None,
            present_queue_index: None,
        }
    }
}

impl VkDevice {
    pub(crate) fn new(create_info: CreateInfo, window: &winit::window::Window) -> Self {
        let entry: ash::Entry = unsafe { ash::Entry::load() }.unwrap();

        let instance = Self::create_instance(&entry, &create_info);

        #[cfg(debug_assertions)]
        let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance);
        #[cfg(debug_assertions)]
        let debug_messenger = Self::create_debug_messenger(&debug_utils, &instance);

        let surface = unsafe {
            #[allow(deprecated)]
            use winit::raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

            Self::create_surface(
                #[allow(deprecated)]
                window.raw_display_handle().unwrap(),
                #[allow(deprecated)]
                window.raw_window_handle().unwrap(),
                &Self::instance_extensions(&entry),
                &entry,
                &instance,
            )
        };

        let surface_khr = ash::extensions::khr::Surface::new(&entry, &instance);
        let extensions = Self::get_required_extensions(&create_info.device_extensions);
        let physical_device =
            Self::pick_physical_device(&surface, &surface_khr, &instance, true, &extensions);
        let properties = Self::get_physical_device_properties(&instance, &physical_device);
        let (device, queues) = Self::create_logical_device(
            &surface_khr,
            &surface,
            &instance,
            &physical_device,
            &create_info.device_extensions,
        );
        let command_pools = Self::create_command_pools(
            &surface_khr,
            &surface,
            &instance,
            &physical_device,
            &device,
        );

        let available_present_modes = unsafe {
            surface_khr.get_physical_device_surface_present_modes(physical_device, surface)
        }
        .unwrap_or_else(|e| {
            log::error!("vkGetPhysicalDeviceSurfacePresentModes failed, error: {e}");
            panic!()
        });

        let vsync_present_mode = [ash::vk::PresentModeKHR::MAILBOX]
            .into_iter()
            .find(|mode| available_present_modes.contains(mode))
            .unwrap_or(ash::vk::PresentModeKHR::FIFO);

        let no_vsync_present_mode = [
            ash::vk::PresentModeKHR::IMMEDIATE,
            ash::vk::PresentModeKHR::FIFO_RELAXED,
            ash::vk::PresentModeKHR::MAILBOX,
        ]
        .into_iter()
        .find(|mode| available_present_modes.contains(mode))
        .unwrap_or(ash::vk::PresentModeKHR::FIFO);

        let swapchain_khr = ash::extensions::khr::Swapchain::new(&instance, &device);

        Self {
            entry,
            instance,
            debug_messenger,
            debug_utils,
            surface_extension: surface_khr,
            swapchain_extension: swapchain_khr,
            surface,
            properties,
            command_pools,
            physical_device,
            device,
            queues,
            vsync_present_mode,
            no_vsync_present_mode,
            available_present_modes,
        }
    }

    fn create_command_pools(
        surface_khr: &ash::extensions::khr::Surface,
        surface: &ash::vk::SurfaceKHR,
        instance: &ash::Instance,
        physical_device: &ash::vk::PhysicalDevice,
        device: &ash::Device,
    ) -> CommandPools {
        let indices = Self::get_queue_indices(surface_khr, surface, physical_device, instance);

        let graphics_pool: ash::vk::CommandPool;
        {
            let pool_info = ash::vk::CommandPoolCreateInfo::builder()
                .queue_family_index(indices.graphics_queue_index.unwrap())
                .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .build();

            graphics_pool = unsafe { device.create_command_pool(&pool_info, None) }.unwrap();
        }

        let compute_pool: ash::vk::CommandPool;
        {
            let pool_info = ash::vk::CommandPoolCreateInfo::builder()
                .queue_family_index(indices.graphics_queue_index.unwrap())
                .flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .build();

            compute_pool = unsafe { device.create_command_pool(&pool_info, None) }.unwrap();
        }

        CommandPools {
            graphics_command_pool: graphics_pool,
            compute_command_pool: compute_pool,
        }
    }

    fn instance_extensions(entry: &ash::Entry) -> Vec<&'static CStr> {
        let instance_extensions = {
            log::trace!("vkEnumerateInstanceExtensionProperties");
            entry.enumerate_instance_extension_properties(None)
        };
        let instance_extensions = instance_extensions.unwrap_or_else(|e| {
            log::error!("vkEnumerateInstanceExtensionProperties failed, error: {e}");
            panic!()
        });

        #[cfg(not(target_os = "windows"))]
        let mut extensions = vec![ash::extensions::khr::Surface::name()];
        #[cfg(target_os = "windows")]
        let mut extensions = vec![ash::extensions::khr::Surface::name(), ash::extensions::khr::Win32Surface::name()];

        #[cfg(target_os = "macos")]
        {
            extensions.push(ash::extensions::ext::MetalSurface::name());
            extensions.push(ash::vk::KhrPortabilityEnumerationFn::name());
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            extensions.push(ash::extensions::khr::XlibSurface::name());
            extensions.push(ash::extensions::khr::XcbSurface::name());
            extensions.push(ash::extensions::khr::WaylandSurface::name());
        }

        extensions.push(ash::vk::ExtSwapchainColorspaceFn::name());
        extensions.push(ash::vk::KhrGetPhysicalDeviceProperties2Fn::name());

        #[cfg(debug_assertions)]
        extensions.push(ash::extensions::ext::DebugUtils::name());

        extensions.retain(|&ext| {
            let keep = instance_extensions.iter().any(|inst_ext| {
                inst_ext.extension_name.contains(&0)
                    && unsafe { CStr::from_ptr(inst_ext.extension_name.as_slice().as_ptr()) } == ext
            });
            if !keep {
                log::warn!(
                    "Couldn't find vulkan instance extension '{}'",
                    ext.to_string_lossy()
                )
            }
            keep
        });

        extensions
    }

    fn create_instance(entry: &ash::Entry, create_info: &CreateInfo) -> ash::Instance {
        log::info!("Creating Instance");
        let app_create_info = ash::vk::ApplicationInfo::builder()
            .engine_name(create_info.engine_name.as_c_str())
            .application_name(create_info.app_name.as_c_str())
            .application_version(create_info.app_version.as_vulkan_version())
            .engine_version(create_info.engine_version.as_vulkan_version())
            .api_version(ash::vk::API_VERSION_1_2)
            .build();

        // Instance Extensions
        let extensions_ptr = Self::instance_extensions(entry)
            .into_iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();

        // Layer Names, right now just a debug layer
        // We also need them as *const c_char
        #[cfg(debug_assertions)]
        let layers = vec![b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const c_char];
        #[cfg(not(debug_assertions))]
        let layers = vec![];

        let create_info = ash::vk::InstanceCreateInfo::builder()
            .application_info(&app_create_info)
            .enabled_extension_names(&layers)
            .enabled_extension_names(&extensions_ptr)
            .build();

        log::trace!("vkCreateInstance");
        unsafe { entry.create_instance(&create_info, None) }.unwrap_or_else(|e| {
            log::error!("vkCreateInstance failed, error: {e}");
            panic!()
        })
    }

    #[cfg(debug_assertions)]
    fn create_debug_messenger(
        debug_utils: &ash::extensions::ext::DebugUtils,
        instance: &ash::Instance,
    ) -> ash::vk::DebugUtilsMessengerEXT {
        log::info!("Creating Debug Messenger");
        let create_info = ash::vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback))
            .build();

        unsafe { debug_utils.create_debug_utils_messenger(&create_info, None) }.unwrap_or_else(
            |e| {
                log::error!("Failed to create debug utils messenger, error: {e}");
                panic!()
            },
        )
    }

    unsafe fn create_surface(
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
        extensions: &[&CStr],
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> ash::vk::SurfaceKHR {
        #[cfg(target_os = "linux")]
        unsafe fn xlib(
            dpy: *mut ash::vk::Display,
            window: ash::vk::Window,
            extensions: &[&CStr],
            entry: &ash::Entry,
            instance: &ash::Instance,
        ) -> ash::vk::SurfaceKHR {
            if !extensions.contains(&ash::extensions::khr::XlibSurface::name()) {
                log::error!("Vulkan driver does not support VK_KHR_xlib_surface");
                panic!();
            }

            let xlib_loader = ash::extensions::khr::XlibSurface::new(entry, instance);
            let info = ash::vk::XlibSurfaceCreateInfoKHR::builder()
                .flags(ash::vk::XlibSurfaceCreateFlagsKHR::empty())
                .window(window)
                .dpy(dpy);

            xlib_loader
                .create_xlib_surface(&info, None)
                .unwrap_or_else(|_| {
                    log::error!("XlibSurface::create_xlib_surface() failed");
                    panic!();
                })
        }

        #[cfg(target_os = "linux")]
        unsafe fn xcb(
            connection: *mut ash::vk::xcb_connection_t,
            window: ash::vk::xcb_window_t,
            extensions: &[&CStr],
            entry: &ash::Entry,
            instance: &ash::Instance,
        ) -> ash::vk::SurfaceKHR {
            if !extensions.contains(&ash::extensions::khr::XcbSurface::name()) {
                log::error!("Vulkan driver does not support VK_KHR_xcb_surface");
                panic!();
            }

            let xcb_loader = ash::extensions::khr::XcbSurface::new(entry, instance);
            let info = ash::vk::XcbSurfaceCreateInfoKHR::builder()
                .flags(ash::vk::XcbSurfaceCreateFlagsKHR::empty())
                .window(window)
                .connection(connection);

            xcb_loader
                .create_xcb_surface(&info, None)
                .unwrap_or_else(|_| {
                    log::error!("XcbSurface::create_xcb_surface() failed");
                    panic!();
                })
        }

        #[cfg(target_os = "linux")]
        unsafe fn wayland(
            display: *mut c_void,
            surface: *mut c_void,
            extensions: &[&CStr],
            entry: &ash::Entry,
            instance: &ash::Instance,
        ) -> ash::vk::SurfaceKHR {
            if !extensions.contains(&ash::extensions::khr::WaylandSurface::name()) {
                log::error!("Vulkan driver does not support VK_KHR_wayland_surface");
                panic!();
            }

            let w_loader = ash::extensions::khr::WaylandSurface::new(entry, instance);
            let info = ash::vk::WaylandSurfaceCreateInfoKHR::builder()
                .flags(ash::vk::WaylandSurfaceCreateFlagsKHR::empty())
                .display(display)
                .surface(surface);

            w_loader
                .create_wayland_surface(&info, None)
                .unwrap_or_else(|_| {
                    log::error!("WaylandSurface::create_wayland_surface() failed");
                    panic!();
                })
        }

        #[cfg(target_os = "windows")]
        unsafe fn windows(
            hinstance: *mut c_void,
            hwnd: *mut c_void,
            extensions: &[&CStr],
            entry: &ash::Entry,
            instance: &ash::Instance,
        ) -> ash::vk::SurfaceKHR {
            if !extensions.contains(&ash::extensions::khr::Win32Surface::name()) {
                log::error!("Vulkan driver does not support VK_KHR_win32_surface");
                panic!();
            }

            let info = ash::vk::Win32SurfaceCreateInfoKHR::builder()
                .flags(ash::vk::Win32SurfaceCreateFlagsKHR::empty())
                .hinstance(hinstance)
                .hwnd(hwnd);
            let win32_loader = ash::extensions::khr::Win32Surface::new(entry, instance);
            win32_loader
                .create_win32_surface(&info, None)
                .unwrap_or_else(|_| {
                    log::error!("Unable to create Win32 surface");
                    panic!();
                })
        }

        // #[cfg(target_os = "macos")]
        // fn create_surface_from_view(
        //     view: *mut c_void,
        // ) -> ash::vk::SurfaceKHR {
        //     if !self.shared.extensions.contains(&ext::MetalSurface::name()) {
        //         return Err(crate::InstanceError::new(String::from(
        //             "Vulkan driver does not support VK_EXT_metal_surface",
        //         )));
        //     }
        //
        //     let layer = unsafe {
        //         crate::metal::Surface::get_metal_layer(view as *mut objc::runtime::Object, None)
        //     };
        //
        //     let surface = {
        //         let metal_loader = ext::MetalSurface::new(&self.shared.entry, &self.shared.raw);
        //         let vk_info = vk::MetalSurfaceCreateInfoEXT::builder()
        //             .flags(vk::MetalSurfaceCreateFlagsEXT::empty())
        //             .layer(layer as *mut _)
        //             .build();
        //
        //         unsafe { metal_loader.create_metal_surface(&vk_info, None).unwrap() }
        //     };
        //
        //     Ok(self.create_surface_from_vk_surface_khr(surface))
        // }

        match (window_handle, display_handle) {
            #[cfg(target_os = "linux")]
            (RawWindowHandle::Wayland(handle), RawDisplayHandle::Wayland(display)) => wayland(
                display.display.as_ptr(),
                handle.surface.as_ptr(),
                extensions,
                entry,
                instance,
            ),
            #[cfg(target_os = "linux")]
            (RawWindowHandle::Xlib(handle), RawDisplayHandle::Xlib(display)) => {
                let display = display.display.expect("Display pointer is not set.");
                xlib(
                    display.as_ptr() as *mut *const c_void,
                    handle.window,
                    extensions,
                    entry,
                    instance,
                )
            }
            #[cfg(target_os = "linux")]
            (RawWindowHandle::Xcb(handle), RawDisplayHandle::Xcb(display)) => {
                let connection = display.connection.expect("Pointer to X-Server is not set.");
                xcb(
                    connection.as_ptr(),
                    handle.window.get(),
                    extensions,
                    entry,
                    instance,
                )
            }
            #[cfg(target_os = "windows")]
            (RawWindowHandle::Win32(handle), _) => {
                let hinstance = winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null());
                windows(
                    hinstance as *mut _,
                    handle.hwnd.get() as *mut _,
                    extensions,
                    entry,
                    instance,
                )
            }
            // #[cfg(target_os = "macos")]
            // (RawWindowHandle::AppKit(handle), _)
            // if self.shared.extensions.contains(&ext::MetalSurface::name()) => {
            //     create_surface_from_view(handle.ns_view.as_ptr())
            // }
            (_, _) => {
                log::error!("window handle {window_handle:?} is not a Vulkan-compatible handle");
                unimplemented!();
            }
        }
    }

    fn get_physical_device_properties(
        instance: &ash::Instance,
        physical_device: &ash::vk::PhysicalDevice,
    ) -> ash::vk::PhysicalDeviceProperties2 {
        let mut properties = ash::vk::PhysicalDeviceProperties2::default();
        unsafe { instance.get_physical_device_properties2(*physical_device, &mut properties) };

        properties
    }

    fn check_surface_support(
        surface_extension: &ash::extensions::khr::Surface,
        surface: &ash::vk::SurfaceKHR,
        physical_device: &ash::vk::PhysicalDevice,
    ) -> bool {
        // Get Surface Capabilities
        //let capabilities = unsafe { surface_extension.get_physical_device_surface_capabilities(*physical_device, *surface) }.unwrap();

        // Get Surface Formats
        let formats = unsafe {
            surface_extension.get_physical_device_surface_formats(*physical_device, *surface)
        }
        .unwrap();

        // Get Presentation Modes
        let presentation_modes = unsafe {
            surface_extension.get_physical_device_surface_present_modes(*physical_device, *surface)
        }
        .unwrap();

        // just check whether it's not empty
        !formats.is_empty() && !presentation_modes.is_empty()
    }

    fn pick_physical_device(
        surface: &ash::vk::SurfaceKHR,
        surface_khr: &ash::extensions::khr::Surface,
        instance: &ash::Instance,
        prioritize_discrete: bool,
        extensions: &[&CStr],
    ) -> ash::vk::PhysicalDevice {
        let devices = unsafe { instance.enumerate_physical_devices() }.expect("No Devices Found!");

        let mut physical_device: Option<ash::vk::PhysicalDevice> = None;

        for device in devices {
            if Self::is_device_suitable(&device, instance, surface_khr, surface, extensions) {
                let properties = unsafe { instance.get_physical_device_properties(device) };
                if properties.device_type == ash::vk::PhysicalDeviceType::DISCRETE_GPU
                    && prioritize_discrete
                {
                    physical_device = Some(device);
                    break;
                } else {
                    physical_device = Some(device);
                }
            }
        }

        match physical_device {
            Some(x) => x,
            None => {
                log::error!("Could find any suitable physical device!");
                panic!()
            }
        }
    }

    fn is_device_suitable(
        physical_device: &ash::vk::PhysicalDevice,
        instance: &ash::Instance,
        surface_khr: &ash::extensions::khr::Surface,
        surface: &ash::vk::SurfaceKHR,
        extensions: &[&CStr],
    ) -> bool {
        let indices = Self::get_queue_indices(surface_khr, surface, physical_device, instance);
        let are_extensions_supported =
            Self::are_device_extensions_supported(physical_device, instance, extensions);

        let surface_support = Self::check_surface_support(surface_khr, surface, physical_device);

        indices.is_complete() && are_extensions_supported && surface_support
    }

    fn get_queue_indices(
        surface_khr: &ash::extensions::khr::Surface,
        surface: &ash::vk::SurfaceKHR,
        physical_device: &ash::vk::PhysicalDevice,
        instance: &ash::Instance,
    ) -> QueueIndices {
        let queues =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };
        let queues: Vec<(usize, ash::vk::QueueFamilyProperties)> = queues
            .iter()
            .enumerate()
            .map(|(index, info)| (index, *info))
            .collect();

        let mut queue_indices = QueueIndices::create();

        for queue in queues {
            if queue.1.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS)
                && queue.1.queue_flags.contains(ash::vk::QueueFlags::COMPUTE)
            {
                queue_indices.graphics_queue_index = Some(queue.0 as u32);
            }
            if queue.1.queue_flags.contains(ash::vk::QueueFlags::COMPUTE)
                && !queue.1.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS)
            {
                queue_indices.compute_queue_index = Some(queue.0 as u32);
            }
            if unsafe {
                surface_khr.get_physical_device_surface_support(
                    *physical_device,
                    queue.0 as u32,
                    *surface,
                )
            }
            .unwrap()
            {
                queue_indices.present_queue_index = Some(queue.0 as u32);
            }
        }

        queue_indices
    }

    fn create_logical_device(
        surface_khr: &ash::extensions::khr::Surface,
        surface: &ash::vk::SurfaceKHR,
        instance: &ash::Instance,
        physical_device: &ash::vk::PhysicalDevice,
        extensions: &Extensions,
    ) -> (ash::Device, Queues) {
        let indices = Self::get_queue_indices(surface_khr, surface, physical_device, instance);
        let mut queue_create_infos: Vec<ash::vk::DeviceQueueCreateInfo> = Vec::new();
        let mut unique_queues: Vec<u32> = Vec::new();
        let mut set: HashSet<u32> = HashSet::new();

        if !set.contains(&indices.present_queue_index.unwrap()) {
            set.insert(indices.present_queue_index.unwrap());
            unique_queues.push(indices.present_queue_index.unwrap());
        }
        if !set.contains(&indices.graphics_queue_index.unwrap()) {
            set.insert(indices.graphics_queue_index.unwrap());
            unique_queues.push(indices.graphics_queue_index.unwrap());
        }
        if !set.contains(&indices.compute_queue_index.unwrap()) {
            set.insert(indices.compute_queue_index.unwrap());
            unique_queues.push(indices.compute_queue_index.unwrap());
        }

        for index in unique_queues {
            let info = ash::vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(index)
                .queue_priorities(&[1.0]);
            queue_create_infos.push(info.build());
        }

        let extensions = Self::get_required_extensions(extensions)
            .into_iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();

        let mut features = ash::vk::PhysicalDeviceFeatures2::builder();

        // Telling vulkan to use BufferDeviceAddressFeature
        let mut device_address = ash::vk::PhysicalDeviceBufferDeviceAddressFeaturesKHR::default(); // we need those structs
        features = features.push_next(&mut device_address);

        let mut features = features.build();

        let create_info = ash::vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&extensions)
            .queue_create_infos(&queue_create_infos)
            .push_next(&mut features)
            .build();

        let device = unsafe { instance.create_device(*physical_device, &create_info, None) }
            .expect("Failed to create logical device!");

        let graphics_queue =
            unsafe { device.get_device_queue(indices.graphics_queue_index.unwrap(), 0) };
        let compute_queue =
            unsafe { device.get_device_queue(indices.compute_queue_index.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(indices.present_queue_index.unwrap(), 0) };

        (
            device,
            Queues {
                graphics_queue,
                compute_queue,
                present_queue,
            },
        )
    }

    fn are_device_extensions_supported(
        physical_device: &ash::vk::PhysicalDevice,
        instance: &ash::Instance,
        requested_extensions: &[&CStr],
    ) -> bool {
        let available_extensions =
            unsafe { instance.enumerate_device_extension_properties(*physical_device) }.unwrap();

        log::info!("Requested Extensions: ");
        for (index, extension_requested) in requested_extensions.iter().enumerate() {
            log::info!("\t{}", extension_requested.to_string_lossy());
            let mut extensions_found = false;
            for extension_available in &available_extensions {
                let name = unsafe { CStr::from_ptr(extension_available.extension_name.as_ptr()) };
                if name == *extension_requested {
                    // requested extension found
                    extensions_found = true;
                }
            }

            // false if some we're not found
            if !extensions_found {
                return false;
            }
        }
        // If all are found return true
        true
    }

    fn get_required_extensions(requested: &Extensions) -> Vec<&'static CStr> {
        let mut extensions = vec![
            ash::vk::ExtImageRobustnessFn::name(),
            ash::vk::KhrSwapchainFn::name(),
            ash::vk::KhrSwapchainMutableFormatFn::name(),
            ash::vk::ExtRobustness2Fn::name(),
            ash::vk::KhrBufferDeviceAddressFn::name(),
            #[cfg(target_os = "macos")]
            ash::vk::KhrPortabilitySubsetFn::name(),
        ];

        if requested.contains(Extensions::DRAW_INDIRECT_COUNT) {
            extensions.push(ash::vk::KhrDrawIndirectCountFn::name());
        }

        if requested.contains(Extensions::RAY_TRACING) {
            extensions.push(ash::vk::KhrDeferredHostOperationsFn::name());
            extensions.push(ash::vk::KhrAccelerationStructureFn::name());
            extensions.push(ash::vk::KhrRayQueryFn::name());
            extensions.push(ash::vk::KhrRayTracingPipelineFn::name());
            extensions.push(ash::vk::KhrShaderClockFn::name());
            extensions.push(ash::vk::KhrExternalMemoryFn::name());
            #[cfg(target_os = "windows")]
            extensions.push(ash::vk::KhrExternalMemoryWin32Fn::name());
        }

        if requested.contains(Extensions::TEXTURE_COMPRESSION_ASTC_HDR) {
            extensions.push(ash::vk::ExtTextureCompressionAstcHdrFn::name());
        }

        extensions
    }

    pub fn get_device(&self) -> &ash::Device {
        &self.device
    }

    pub fn get_surface(&self) -> ash::vk::SurfaceKHR {
        self.surface
    }

    pub fn get_surface_khr(&self) -> &ash::extensions::khr::Surface {
        &self.surface_extension
    }

    pub fn get_physical_device(&self) -> ash::vk::PhysicalDevice {
        self.physical_device
    }

    pub fn get_available_present_modes(&self) -> &Vec<ash::vk::PresentModeKHR> {
        &self.available_present_modes
    }

    pub fn get_vsync_present_mode(&self) -> ash::vk::PresentModeKHR {
        self.vsync_present_mode
    }

    pub fn get_no_vsync_present_mode(&self) -> ash::vk::PresentModeKHR {
        self.no_vsync_present_mode
    }

    pub fn get_indices(&self) -> QueueIndices {
        Self::get_queue_indices(
            &self.surface_extension,
            &self.surface,
            &self.physical_device,
            &self.instance,
        )
    }

    pub fn get_swapchain_extension(&self) -> &ash::extensions::khr::Swapchain {
        &self.swapchain_extension
    }

    pub fn get_instance(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn find_supported_formats(
        &self,
        formats: &[ash::vk::Format],
        tiling: ash::vk::ImageTiling,
        features: ash::vk::FormatFeatureFlags,
    ) -> ash::vk::Format {
        for format in formats {
            let properties = unsafe {
                self.instance
                    .get_physical_device_format_properties(self.physical_device, *format)
            };
            if (tiling == ash::vk::ImageTiling::LINEAR
                && (properties.linear_tiling_features & features) == features)
                || (tiling == ash::vk::ImageTiling::OPTIMAL
                    && (properties.optimal_tiling_features & features) == features)
            {
                return *format; // return first format on the list that fulfils requirements
            }
        }

        ash::vk::Format::UNDEFINED // return undefined if none are supported
    }
}

#[cfg(debug_assertions)]
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: ash::vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: ash::vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const ash::vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> ash::vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    let ty = match message_type {
        ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "GENERAL",
        ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "VALIDATION",
        ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "PERFORMANCE",
        _ => "",
    };

    match message_severity {
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!("Vulkan {ty} [{message_id_name} ({message_id_number})] : {message}")
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("Vulkan {ty} [{message_id_name} ({message_id_number})] : {message}")
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("Vulkan {ty} [{message_id_name} ({message_id_number})] : {message}")
        }
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("Vulkan {ty} [{message_id_name} ({message_id_number})] : {message}")
        }
        _ => {}
    }

    ash::vk::FALSE
}
