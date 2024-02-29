use ash::vk::Handle;
use std::cmp::min;
use std::num::NonZeroU32;
use std::sync::Arc;

use itertools::Itertools;
use mvsync::block::AwaitSync;
use mvutils::utils::TetrahedronOp;
use openxr::{
    EnvironmentBlendMode, FrameStream, FrameWaiter, OpenGL, Session, SystemId,
    ViewConfigurationType, Vulkan,
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Adapter, AddressMode, Backend, Backends, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState,
    Buffer, BufferDescriptor, BufferUsages, ColorWrites, CompositeAlphaMode, Device,
    DeviceDescriptor, Extent3d, Face, FilterMode, FragmentState, FrontFace, IndexFormat,
    InstanceDescriptor, PolygonMode, PowerPreference, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPipeline, RequestAdapterOptions, SamplerDescriptor,
    ShaderModule, ShaderStages, Surface, SurfaceConfiguration, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    VertexBufferLayout, VertexState,
};
use wgpu::{Instance, InstanceFlags};
use winit::dpi::PhysicalSize;

use crate::render::common::Texture;
use crate::render::consts::{
    BIND_GROUPS, BIND_GROUP_2D, BIND_GROUP_3D, BIND_GROUP_EFFECT, BIND_GROUP_EFFECT_CUSTOM,
    BIND_GROUP_GEOMETRY_3D, BIND_GROUP_GEOMETRY_BATCH_3D, BIND_GROUP_LAYOUT_2D,
    BIND_GROUP_LAYOUT_3D, BIND_GROUP_LAYOUT_BATCH_3D, BIND_GROUP_LAYOUT_EFFECT,
    BIND_GROUP_LAYOUT_EFFECT_CUSTOM, BIND_GROUP_LAYOUT_GEOMETRY_3D,
    BIND_GROUP_LAYOUT_GEOMETRY_BATCH_3D, BIND_GROUP_LAYOUT_LIGHTING_3D,
    BIND_GROUP_LAYOUT_MODEL_MATRIX, BIND_GROUP_LIGHTING_3D, BIND_GROUP_MODEL_MATRIX,
    BIND_GROUP_TEXTURES, BIND_GROUP_TEXTURES_3D, DEFAULT_SAMPLER, DUMMY_TEXTURE, INDEX_LIMIT,
    LIGHT_LIMIT, MAX_LIGHTS, MAX_TEXTURES, TEXTURE_LIMIT, VERTEX_LAYOUT_2D, VERTEX_LAYOUT_3D,
    VERTEX_LAYOUT_MODEL_3D, VERTEX_LAYOUT_NONE, VERT_LIMIT_2D_BYTES,
};
use crate::render::window::WindowSpecs;

pub(crate) struct VRState {
    //pub(crate) surface: Surface,
    //pub(crate) device: Device,
    //pub(crate) queue: Queue,
    //pub(crate) config: SurfaceConfiguration,
    //pub(crate) backend: Backend,
}

impl VRState {
    pub(crate) fn new(specs: &WindowSpecs) -> Self {
        Self::init(specs).await_sync()
    }

    async fn init(specs: &WindowSpecs) -> Self {
        unsafe {
            let entry = openxr::Entry::load().expect("Couldn't find OpenXR loader");

            let xr_extensions = entry
                .enumerate_extensions()
                .expect("Couldn't find OpenXR extensions");

            let mut backends = Backends::empty();

            if xr_extensions.khr_vulkan_enable2 {
                backends = backends | Backends::VULKAN;
            }

            if backends.is_empty() {
                panic!("No OpenXR compatible graphics drivers found");
            }

            let layers = entry
                .enumerate_layers()
                .expect("Couldn't find OpenXR layers");

            println!("Layers: {:?}", layers);

            let xr_instance = entry
                .create_instance(
                    &openxr::ApplicationInfo {
                        application_name: "",
                        application_version: 0,
                        engine_name: "MVEngine",
                        engine_version: 0,
                    },
                    &xr_extensions,
                    &[],
                )
                .expect("Couldn't create OpenXR instance");

            let system = xr_instance
                .system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)
                .expect("No VR headset detected");

            let blend_modes = xr_instance
                .enumerate_environment_blend_modes(system, ViewConfigurationType::PRIMARY_STEREO)
                .expect("No blend modes supported by VR headset");

            let blend_mode = blend_modes.contains(&EnvironmentBlendMode::OPAQUE).yn(
                EnvironmentBlendMode::OPAQUE,
                *blend_modes
                    .get(0)
                    .expect("No blend modes supported by VR headset"),
            );

            let instance = Instance::new(InstanceDescriptor {
                backends,
                flags: InstanceFlags::from_build_config(),
                dx12_shader_compiler: Default::default(),
                gles_minor_version: Default::default(),
            });

            let adapter = instance.request_adapter(
                &RequestAdapterOptions {
                    power_preference: specs.green_eco_mode.yn(PowerPreference::LowPower, PowerPreference::HighPerformance),
                    compatible_surface: None,
                    force_fallback_adapter: false,
                }
            ).await.expect("Graphical adapter cannot be found! (This is usually a driver issue, or you are missing hardware)");

            let backend = adapter.get_info().backend;

            if !matches!(backend, Backend::Gl | Backend::Vulkan | Backend::Dx11) {
                panic!("Graphics backend {:?} not supported for VR", backend);
            }

            let textures = adapter.limits().max_sampled_textures_per_shader_stage;

            let _ = MAX_TEXTURES.try_create(|| min(textures as usize - 1, TEXTURE_LIMIT));

            let _ = MAX_LIGHTS.try_create(|| LIGHT_LIMIT);

            let (device, queue) = adapter
                .request_device(
                    &DeviceDescriptor {
                        features: adapter.features(),
                        limits: adapter.limits(),
                        label: Some("GPU"),
                    },
                    None,
                )
                .await
                .expect("Could not create logical device!");

            let vr_state = match backend {
                Backend::Vulkan => Self::init_vulkan(
                    xr_instance,
                    system,
                    blend_mode,
                    &instance,
                    &adapter,
                    &device,
                ),
                _ => unreachable!(),
            };

            //let surface_caps = surface.get_capabilities(&adapter);
            //let surface_format = surface_caps
            //    .formats
            //    .iter()
            //    .copied()
            //    .find(|f| f.is_srgb())
            //    .unwrap_or(surface_caps.formats[0]);

            //let surface_alpha = surface_caps
            //    .alpha_modes
            //    .contains(&CompositeAlphaMode::Opaque)
            //    .yn(CompositeAlphaMode::Opaque, surface_caps.alpha_modes[0]);

            //let config = SurfaceConfiguration {
            //    usage: TextureUsages::RENDER_ATTACHMENT,
            //    format: surface_format,
            //    width: specs.width,
            //    height: specs.height,
            //    present_mode: specs
            //        .vsync
            //        .yn(PresentMode::AutoVsync, PresentMode::AutoNoVsync),
            //    alpha_mode: surface_alpha,
            //    view_formats: vec![],
            //};
            //surface.configure(&device, &config);

            let _ = BIND_GROUPS.try_init(|groups| {
                groups.insert(
                    BIND_GROUP_2D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_2D),
                );
                groups.insert(
                    BIND_GROUP_TEXTURES,
                    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("Bind group layout textures 2D"),
                        entries: &[BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                view_dimension: TextureViewDimension::D2,
                                sample_type: TextureSampleType::Float { filterable: true },
                            },
                            count: Some(NonZeroU32::new_unchecked(*MAX_TEXTURES as u32)),
                        }],
                    }),
                );
                groups.insert(
                    BIND_GROUP_MODEL_MATRIX,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_MODEL_MATRIX),
                );
                groups.insert(
                    BIND_GROUP_3D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_BATCH_3D),
                );
                groups.insert(
                    BIND_GROUP_TEXTURES_3D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_3D),
                );
                groups.insert(
                    BIND_GROUP_GEOMETRY_BATCH_3D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_GEOMETRY_BATCH_3D),
                );
                groups.insert(
                    BIND_GROUP_GEOMETRY_3D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_GEOMETRY_3D),
                );
                groups.insert(
                    BIND_GROUP_LIGHTING_3D,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_LIGHTING_3D),
                );
                groups.insert(
                    BIND_GROUP_EFFECT,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_EFFECT),
                );
                groups.insert(
                    BIND_GROUP_EFFECT_CUSTOM,
                    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_EFFECT_CUSTOM),
                );
            });

            let texture = device.create_texture(&TextureDescriptor {
                label: Some("Dummy texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&TextureViewDescriptor::default());

            let _ = DUMMY_TEXTURE.try_create(|| Arc::new(Texture::premade(texture, view)));

            let _ = DEFAULT_SAMPLER.try_create(|| {
                device.create_sampler(&SamplerDescriptor {
                    label: Some("Texture sampler"),
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    mag_filter: FilterMode::Linear,
                    min_filter: FilterMode::Linear,
                    mipmap_filter: FilterMode::Linear,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: 32.0,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                })
            });

            //Self {
            //    surface,
            //    device,
            //    queue,
            //    config,
            //    backend: adapter.get_info().backend,
            //}
            todo!()
        }
    }

    unsafe fn init_vulkan(
        xr_instance: openxr::Instance,
        system: SystemId,
        blend_mode: EnvironmentBlendMode,
        instance: &Instance,
        adapter: &Adapter,
        device: &Device,
    ) -> Self {
        let vk_instance = instance
            .as_hal::<wgpu::hal::api::Vulkan>()
            .expect("Corrupted Vulkan instance")
            .shared_instance()
            .raw_instance();
        let vk_physical_device = adapter.as_hal(|a: Option<&wgpu::hal::api::Vulkan::Adapter>| {
            a.expect("Corrupted Vulkan instance").raw_physical_device()
        });
        let vk_device = device.as_hal(|a: Option<&wgpu::hal::api::Vulkan::Device>| {
            a.expect("Corrupted Vulkan instance").raw_device()
        });

        let (session, frame_wait, frame_stream) = xr_instance
            .create_session::<Vulkan>(
                system,
                &openxr::vulkan::SessionCreateInfo {
                    instance: vk_instance.handle().as_raw() as _,
                    physical_device: vk_physical_device.as_raw() as _,
                    device: vk_device.handle().as_raw() as _,
                    queue_family_index,
                    queue_index: 0,
                },
            )
            .expect("Failed to create VR session");

        Self::init_internal(xr_instance, session, frame_wait, frame_stream)
    }

    //unsafe fn init_gl(xr_instance: openxr::Instance, system: SystemId, blend_mode: EnvironmentBlendMode, instance: &Instance, adapter: &Adapter, device: &Device) -> Self {
    //    let gl_device = device.as_hal(|a: Option<&wgpu::hal::api::Gles::Device>| a.expect("Corrupted OpenGL instance"));
    //    Self::init_internal(xr_instance, session, frame_wait, frame_stream)
    //}

    fn init_internal<G: openxr::Graphics>(
        xr_instance: openxr::Instance,
        session: Session<G>,
        frame_wait: FrameWaiter,
        frame_stream: FrameStream<Vulkan>,
    ) -> Self {
        let action_set = xr_instance
            .create_action_set("input", "input pose information", 0)
            .expect("Couldn't get VR input");

        let right_action = action_set
            .create_action::<openxr::Posef>("right_hand", "Right Hand Controller", &[])
            .expect("Couldn't get right hand input");
        let left_action = action_set
            .create_action::<openxr::Posef>("left_hand", "Left Hand Controller", &[])
            .expect("Couldn't get left hand input");

        xr_instance
            .suggest_interaction_profile_bindings(
                xr_instance
                    .string_to_path("/interaction_profiles/khr/simple_controller")
                    .expect("Created path inaccessible"),
                &[
                    openxr::Binding::new(
                        &right_action,
                        xr_instance
                            .string_to_path("/user/hand/right/input/grip/pose")
                            .expect("Created path inaccessible"),
                    ),
                    openxr::Binding::new(
                        &left_action,
                        xr_instance
                            .string_to_path("/user/hand/left/input/grip/pose")
                            .expect("Created path inaccessible"),
                    ),
                ],
            )
            .expect("Failed to set bindings for hands");

        session
            .attach_action_sets(&[&action_set])
            .expect("Failed to attach binding sets");

        let right_space = right_action
            .create_space(session.clone(), openxr::Path::NULL, openxr::Posef::IDENTITY)
            .expect("Failed to create space for right hand");
        let left_space = left_action
            .create_space(session.clone(), openxr::Path::NULL, openxr::Posef::IDENTITY)
            .expect("Failed to create space for left hand");

        let stage = session
            .create_reference_space(openxr::ReferenceSpaceType::LOCAL, openxr::Posef::IDENTITY)
            .expect("Failed to create stage");

        todo!()
    }
}
