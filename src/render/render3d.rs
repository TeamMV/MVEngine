use std::ffi::c_void;
use std::ptr::null_mut;
use std::slice;
use std::sync::Arc;

use glam::Mat4;
use mvutils::unsafe_utils::Unsafe;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, IndexFormat,
    RenderPass, TextureView,
};

use crate::render::common::{Bytes, Shader, Texture};
use crate::render::common3d::{Material, DUMMY_MATERIAL, InstancedMaterial};
use crate::render::consts::{
    DEFAULT_SAMPLER, DUMMY_TEXTURE, MATERIAL_LIMIT, MAX_MATERIALS, MAX_TEXTURES, TEXTURE_LIMIT,
};
use crate::render::init::State;
use crate::render::render2d::TextureBindGroup;

pub(crate) trait RenderPass3D {
    fn new_frame(&mut self, render_pass: &mut RenderPass, projection: Mat4, view_matrix: Mat4);

    fn render(
        &mut self,
        indices: &[u32],
        vertices: &[f32],
        materials: &[Option<Arc<Material>>; MATERIAL_LIMIT],
        transforms: &[Mat4],
    ) {
        self.render_instanced(indices, vertices, materials, &[transforms], 1);
    }

    fn render_instanced(
        &mut self,
        indices: &[u32],
        vertices: &[f32],
        materials: &[Option<Arc<Material>>; MATERIAL_LIMIT],
        transforms: &[&[Mat4]],
        num_instances: u32,
    );
}

pub(crate) struct MaterialTextureComBindEdGroup {
    pub(crate) bind_group: BindGroup,
    pub(crate) textures: [Option<Arc<Texture>>; TEXTURE_LIMIT],
    pub(crate) using: [u16; TEXTURE_LIMIT],
    pub(crate) views: [&'static TextureView; TEXTURE_LIMIT],
    pub(crate) materials: [Option<(Arc<Material>, InstancedMaterial)>; MATERIAL_LIMIT],
    pub(crate) raw_materials: [[f32; Material::SIZE_FLOATS]; MATERIAL_LIMIT],
    pub(crate) material_buffer: Buffer,
}

impl MaterialTextureComBindEdGroup {
    pub(crate) fn new(shader: &Shader, state: &State, index: u32) -> Self {
        let textures = [0; TEXTURE_LIMIT].map(|_| None);
        let views: [&'static TextureView; TEXTURE_LIMIT] =
            [DUMMY_TEXTURE.get_view(); TEXTURE_LIMIT];
        let materials = [0; MATERIAL_LIMIT].map(|_| None);
        let raw_materials = [[0.0; Material::SIZE_FLOATS]; MATERIAL_LIMIT];
        let material_buffer =
            state.gen_uniform_buffer_sized(Material::SIZE_BYTES as u64 * *MAX_MATERIALS as u64);

        let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(index),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: material_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureViewArray(&views[..*MAX_TEXTURES]),
                },
            ],
        });

        Self {
            bind_group,
            textures,
            using: [0; TEXTURE_LIMIT],
            views,
            materials,
            raw_materials,
            material_buffer,
        }
    }

    pub(crate) fn set_texture(&mut self, index: usize, texture: Arc<Texture>) {
        self.views[index] = unsafe { (texture.get_view() as *const TextureView).as_ref().unwrap() };
        self.textures[index] = Some(texture);
    }

    pub(crate) fn set_material(&mut self, index: usize, material: Arc<Material>) {
        self.materials[index] = Some((material.clone(), material.into()));
    }

    pub(crate) fn remake(&mut self, state: &State, shader: &Shader, index: u32) {
        self.bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(index),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.material_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureViewArray(&self.views[..*MAX_TEXTURES]),
                },
            ],
        });
    }

    pub fn remap(&mut self) {
        for (i, mat) in self.materials.iter_mut().enumerate() {
            if let Some(mat) = mat {
                mat.1.adapt(&mat.0);
                self.raw_materials[i] = mat.1.raw_data();
            }
        }
    }
}

pub(crate) struct ForwardPass {
    state: &'static State,
    shader: Shader,
    render_pass: *mut c_void,
    projection: Mat4,
    view: Mat4,
    uniform_buffer: Buffer,
    texture_groups: Vec<MaterialTextureComBindEdGroup>,
    uniform: BindGroup,
    pass: usize,
    ibo: Vec<Buffer>,
    vbo: Vec<Buffer>,
}

impl ForwardPass {
    pub(crate) fn new(shader: Shader, state: &State, projection: Mat4, view: Mat4) -> Self {
        let (vbo, ibo) = state.gen_buffers();
        let uniform_buffer = state.gen_uniform_buffer_sized(128);
        state
            .queue
            .write_buffer(&uniform_buffer, 0, projection.cast_bytes());
        state
            .queue
            .write_buffer(&uniform_buffer, 64, view.cast_bytes());
        let uniform = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("uniform bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&DEFAULT_SAMPLER),
                },
            ],
        });

        let texture_groups = vec![MaterialTextureComBindEdGroup::new(&shader, state, 2)];

        Self {
            state: unsafe { Unsafe::cast_static(state) },
            shader,
            render_pass: null_mut(),
            projection,
            view,
            uniform_buffer,
            texture_groups,
            uniform,
            pass: 0,
            ibo: vec![ibo],
            vbo: vec![vbo],
        }
    }
}

impl RenderPass3D for ForwardPass {
    fn new_frame(&mut self, render_pass: &mut RenderPass, projection: Mat4, view_matrix: Mat4) {
        self.pass = 0;

        if self.projection != projection {
            self.projection = projection;
            self.state
                .queue
                .write_buffer(&self.uniform_buffer, 0, projection.cast_bytes());
        }

        if self.view != view_matrix {
            self.view = view_matrix;
            self.state
                .queue
                .write_buffer(&self.uniform_buffer, 64, view_matrix.cast_bytes());
        }

        self.render_pass = render_pass as *mut RenderPass as *mut c_void;

        unsafe {
            (self.render_pass as *mut RenderPass)
                .as_mut()
                .unwrap()
                .set_bind_group(0, &self.uniform, &[])
        };
    }

    fn render_instanced(
        &mut self,
        indices: &[u32],
        vertices: &[f32],
        materials: &[Option<Arc<Material>>; MATERIAL_LIMIT],
        transforms: &[&[Mat4]],
        num_instances: u32,
    ) {
        unsafe {
            if num_instances == 0 {
                return;
            }
            if num_instances != transforms.len() as u32 {
                panic!("Invalid transformation matrix data! Expected {} instances, found {}", num_instances, transforms.len());
            }

            let amount = transforms[0].len();

            for t in transforms {
                if t.len() != amount {
                    panic!("All instances must have the same amount of transform matrices");
                }
            }

            let transforms = slice::from_raw_parts(transforms[0].as_ptr(), amount * num_instances as usize);

            if self.ibo.len() <= self.pass {
                let (vbo, ibo) = self.state.gen_buffers();
                self.vbo.push(vbo);
                self.ibo.push(ibo);
                self.texture_groups.push(MaterialTextureComBindEdGroup::new(
                    &self.shader,
                    self.state,
                    2,
                ));
            }

            let ibo = &self.ibo[self.pass];
            let vbo = &self.vbo[self.pass];
            let texture_group = &mut self.texture_groups[self.pass];
            let render_pass = (self.render_pass as *mut RenderPass)
                .as_mut()
                .expect("You need to call RenderPass2D::new_frame() before rendering!");

            self.state.queue.write_buffer(ibo, 0, indices.cast_bytes());
            self.state.queue.write_buffer(vbo, 0, vertices.cast_bytes());



            render_pass.set_bind_group(2, &texture_group.bind_group, &[]);
            render_pass.set_pipeline(self.shader.get_pipeline());
            render_pass.set_vertex_buffer(0, vbo.slice(..));
            render_pass.set_index_buffer(ibo.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..num_instances);
            self.pass += 1;
        }
    }
}