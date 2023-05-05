use std::ffi::c_void;
use std::ops::Deref;
use std::ptr::{null, null_mut};
use glam::Mat4;
use image::EncodableLayout;
use mvutils::utils::TetrahedronOp;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, CommandEncoder, IndexFormat, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, Sampler, TextureView};
use crate::render::common::{Shader, Bytes, Texture};
use crate::render::consts::{DEFAULT_SAMPLER, DUMMY_TEXTURE};
use crate::render::init::State;

struct TextureBindGroup {
    bind_group: BindGroup,
    textures: [&'static Texture; 1],
    views: [&'static TextureView; 1],
    samplers: [&'static Sampler; 1]
}

impl TextureBindGroup {
    fn new(shader: &Shader, state: &State) -> Self {
        let textures: [&'static Texture; 1] = [unsafe { DUMMY_TEXTURE.as_ref().unwrap() }; 1];
        let views: [&'static TextureView; 1] = [unsafe { DUMMY_TEXTURE.as_ref().unwrap().get_view() }; 1];
        let samplers: [&'static Sampler; 1] = [unsafe { DEFAULT_SAMPLER.as_ref().unwrap() }; 1];

        let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(1),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(&views),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::SamplerArray(&samplers),
                }
            ],
        });

        Self {
            bind_group,
            textures,
            views,
            samplers,
        }
    }

    fn set(&mut self, index: usize, texture: &Texture) {
        let texture = unsafe { (texture as *const Texture).as_ref() }.unwrap();
        self.textures[index] = texture;
        self.views[index] = texture.get_view();
        self.samplers[index] = texture.get_sampler();
    }

    fn remake(&mut self, state: &State, shader: &Shader) {
        self.bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(1),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(&self.views),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::SamplerArray(&self.samplers),
                }
            ],
        });
    }
}

pub(crate) struct RenderPass2D {
    state: &'static State,
    shader: Shader,
    vbo: Vec<Buffer>,
    ibo: Vec<Buffer>,
    texture_groups: Vec<TextureBindGroup>,
    uniform: BindGroup,
    uniform_buffer: Buffer,
    pass: usize,
    projection: Mat4,
    view: Mat4,
    render_pass: *mut c_void,
}

impl RenderPass2D {
    pub(crate) fn new(shader: Shader, state: &State, projection: Mat4, view: Mat4) -> Self {
        let (vbo, ibo) = state.gen_buffers();

        let uniform = state.gen_uniform_buffer_sized(128);

        state.queue.write_buffer(&uniform, 0, projection.cast_bytes());
        state.queue.write_buffer(&uniform, 64, view.cast_bytes());

        let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                }
            ],
        });

        let texture_group = TextureBindGroup::new(&shader, state);

        Self {
            state: unsafe { (state as *const State).as_ref() }.unwrap(),
            shader,
            vbo: vec![vbo],
            ibo: vec![ibo],
            texture_groups: vec![texture_group],
            uniform: bind_group,
            uniform_buffer: uniform,
            pass: 0,
            projection,
            view,
            render_pass: null_mut(),
        }
    }

    pub(crate) fn new_frame(&mut self, render_pass: &mut RenderPass, projection: Mat4, view_matrix: Mat4) {
        self.pass = 0;

        if self.projection != projection {
            self.projection = projection;
            self.state.queue.write_buffer(&self.uniform_buffer, 0, projection.cast_bytes());
        }

        if self.view!= view_matrix {
            self.view = view_matrix;
            self.state.queue.write_buffer(&self.uniform_buffer, 64, view_matrix.cast_bytes());
        }

        self.render_pass = render_pass as *mut RenderPass as *mut c_void;
    }

    pub(crate) fn render(&mut self, indices: &[u32], vertices: &[f32], texture: Option<&Texture>, stripped: bool) {
        unsafe {
            if self.ibo.len() <= self.pass {
                let (vbo, ibo) = self.state.gen_buffers();
                self.vbo.push(vbo);
                self.ibo.push(ibo);
                self.texture_groups.push(TextureBindGroup::new(&self.shader, self.state));
            }
            let ibo = &self.ibo[self.pass];
            let vbo = &self.vbo[self.pass];
            let texture_group = &mut self.texture_groups[self.pass];
            let render_pass = (self.render_pass as *mut RenderPass).as_mut().expect("You need to call RenderPass2D::next_frame() before rendering!");

            self.state.queue.write_buffer(ibo, 0, indices.cast_bytes());
            self.state.queue.write_buffer(vbo, 0, vertices.cast_bytes());

            if let Some(texture) = texture {
                if texture_group.textures[0] != texture {
                    texture_group.set(0, texture);
                    texture_group.remake(self.state, &self.shader)
                }
            }
            else if texture_group.textures[0] != DUMMY_TEXTURE.as_ref().unwrap() {
                texture_group.set(0, DUMMY_TEXTURE.as_ref().unwrap());
                texture_group.remake(self.state, &self.shader);
            }

            render_pass.set_bind_group(0, &self.uniform, &[]);
            render_pass.set_bind_group(1, &texture_group.bind_group, &[]);
            render_pass.set_pipeline(stripped.yn(self.shader.get_stripped_pipeline(), self.shader.get_pipeline()));
            render_pass.set_vertex_buffer(0, vbo.slice(..));
            render_pass.set_index_buffer(ibo.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            self.pass += 1;
        }
    }

    pub(crate) fn finish(&mut self) {
        self.render_pass = null_mut();
    }
}

pub(crate) struct EffectPass {

}

#[cfg(feature = "3d")]
pub(crate) trait RenderPass3D {
    fn render_batch(&self, ) {

    }

    fn render_model(&self, ) {

    }
}

#[cfg(feature = "3d")]
pub(crate) struct ForwardPass {

}