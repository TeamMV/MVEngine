use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::Arc;

use glam::Mat4;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, RenderPass};

use crate::render::common::{Bytes, Shader, Texture};
use crate::render::common3d::Material;
use crate::render::consts::{DEFAULT_SAMPLER, MAX_MATERIALS, TEXTURE_LIMIT};
use crate::render::init::State;
use crate::render::render2d::TextureBindGroup;

pub(crate) trait RenderPass3D {
    fn new_frame(&mut self,
                 render_pass: &mut RenderPass,
                 projection: Mat4,
                 view_matrix: Mat4);

    fn render_batch(
        &self,
        indices: &[u32],
        vertices: &[f32],
        textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT],
        transforms: &[Mat4],
    );

    fn render_model_instanced(
        &self,
        indices: &[u32],
        vertices: &[f32],
        textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT],
        transforms: &[Mat4],
        num_instances: u32,
    );
}

pub(crate) struct ForwardPass {
    state: &'static State,
    shader: Shader,
    render_pass: *mut c_void,
    projection: Mat4,
    view: Mat4,
    uniform_buffer: Buffer,
    material_buffer: Buffer,
    uniform: BindGroup,
    pass_idx: usize,
    ibos: Vec<Buffer>,
    vbos: Vec<Buffer>,
}

impl ForwardPass {
    fn new(shader: Shader, state: &State, projection: Mat4, view: Mat4) -> Self {
        let(vbo, ibo) = state.gen_buffers();
        let uniform_buffer = state.gen_uniform_buffer_sized(128);
        let material_buffer = state.gen_uniform_buffer_sized((Material::SIZE_BYTES * *MAX_MATERIALS) as u64);
        let uniform = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("uniform bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: material_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&DEFAULT_SAMPLER),
                },
            ],
        });
        Self {
            state,
            shader,
            render_pass: null_mut(),
            projection,
            view,
            uniform_buffer,
            uniform,
            pass_idx: 0,
            ibos: vec![ibo],
            vbos: vec![vbo],
        }
    }

    fn prepare_render(&mut self) {
        if self.pass_idx >= self.ibos.len() {
            let (vbo, ibo) = self.state.gen_buffers();
            self.ibos.push(ibo);
            self.vbos.push(vbo);
        }

        let ibo = &self.ibos[self.pass_idx];
        let vbo = &self.vbos[self.pass_idx];
    }
}

impl RenderPass3D for ForwardPass {
    fn new_frame(&mut self, render_pass: &mut RenderPass, projection: Mat4, view_matrix: Mat4) {
        self.pass_idx = 0;

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

    fn render_batch(
        &self,
        indices: &[u32],
        vertices: &[f32],
        textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT],
        transforms: &[Mat4],
    ) {
        let (vbo, ibo) = self.state.gen_buffers();
        self.state.queue.write_buffer(&vbo, 0, vertices.cast_bytes());
        self.state.queue.write_buffer(&ibo, 0, indices.cast_bytes());

    }

    fn render_model_instanced(
        &self,
        indices: &[u32],
        vertices: &[f32],
        textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT],
        transforms: &[Mat4],
        num_instances: u32,
    ) {

    }
}
