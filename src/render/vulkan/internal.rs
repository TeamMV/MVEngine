use std::sync::Arc;
use glfw::ffi::glfwVulkanSupported;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::{Version, VulkanLibrary};
use crate::ApplicationInfo;

pub(crate) struct Vulkan {
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl Vulkan {

    pub(crate) unsafe fn init(info: &ApplicationInfo) -> Result<Vulkan, ()> {
        if glfwVulkanSupported() == glfw::ffi::FALSE {
            return Err(());
        }
        let library = VulkanLibrary::new().map_err(|_| ())?;
        let mut instance_info = InstanceCreateInfo::default();
        instance_info.application_name = Some(info.name.clone());
        instance_info.application_version = Version::from(info.version.as_vulkan_version());
        instance_info.engine_name = Some("MVCore".to_string());
        instance_info.engine_version = Version::from(mvutils::version::Version::new(1, 0, 0).as_vulkan_version());
        let instance = Instance::new(library, instance_info).map_err(|_| ())?;
        let mut devices = instance.enumerate_physical_devices().map_err(|_| ())?.collect::<Vec<_>>();
        if devices.len() == 0 {
            return Err(());
        }
        let physical_device = Self::choose_physical_device(devices);
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_, props)| { props.queue_flags.contains(QueueFlags::GRAPHICS) }).ok_or(())? as u32;

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ).map_err(|_| ())?;

        let queue = queues.next().ok_or(())?;

        Ok(Vulkan {
            instance,
            physical_device,
            device,
            queue,
        })
    }

    fn choose_physical_device(devices: Vec<Arc<PhysicalDevice>>) -> Arc<PhysicalDevice> {
        devices.into_iter().max_by_key(|device| {
            let mut score = match device.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 32000,
                PhysicalDeviceType::IntegratedGpu => 16000,
                PhysicalDeviceType::VirtualGpu => 8000,
                _ => 0
            };

            score += device.properties().max_image_dimension2_d;
            score += device.properties().max_image_dimension3_d;

            score
        }).unwrap()
    }

}