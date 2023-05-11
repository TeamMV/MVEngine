use std::ffi::c_void;
use std::ops::Deref;
use std::ptr::{null, null_mut};
use std::sync::Arc;
use glam::Mat4;
use image::EncodableLayout;
use mvutils::utils::TetrahedronOp;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, CommandEncoder, Extent3d, IndexFormat, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, Sampler, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};
use crate::render::common::{Shader, Bytes, Texture};
use crate::render::consts::{DEFAULT_SAMPLER, DUMMY_TEXTURE, MAX_TEXTURES, TEXTURE_LIMIT};
use crate::render::init::State;

struct TextureBindGroup {
    bind_group: BindGroup,
    textures: [Arc<Texture>; TEXTURE_LIMIT],
    views: [&'static TextureView; TEXTURE_LIMIT]


}

impl TextureBindGroup {
    fn new(shader: &Shader, state: &State) -> Self {
        let textures: [Arc<Texture>; TEXTURE_LIMIT] = [0; TEXTURE_LIMIT].map(|_| unsafe { DUMMY_TEXTURE.clone().unwrap() });
        let views: [&'static TextureView; TEXTURE_LIMIT] = [unsafe { DUMMY_TEXTURE.as_ref().unwrap().get_view() }; TEXTURE_LIMIT];

        let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(1),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(&views[..unsafe { MAX_TEXTURES }]),
                }
            ],
        });

        Self {
            bind_group,
            textures,
            views
        }
    }

    fn set(&mut self, index: usize, texture: Arc<Texture>) {
        self.views[index] = unsafe { (texture.get_view() as *const TextureView).as_ref().unwrap() };
        self.textures[index] = texture;
    }

    fn remake(&mut self, state: &State, shader: &Shader) {
        self.bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(1),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(&self.views[..unsafe { MAX_TEXTURES }]),
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
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(unsafe { DEFAULT_SAMPLER.as_ref().unwrap() }),
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

    pub(crate) fn render(&mut self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT], stripped: bool) {
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

            let mut changed = false;

            for i in 0..MAX_TEXTURES {
                if let Some(ref texture) = textures[i] {
                    if &texture_group.textures[i] != texture {
                        texture_group.set(i, texture.clone());
                        changed = true;
                    }
                }
                else if texture_group.textures[i] != DUMMY_TEXTURE.clone().unwrap() {
                    texture_group.set(i, DUMMY_TEXTURE.clone().unwrap());
                    changed = true;
                }
            }

            if changed {
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

pub(crate) struct EBuffer {
    texture: wgpu::Texture,
    view: TextureView,
}

impl EBuffer {
    pub(crate) fn generate(state: &State, width: u32, height: u32) -> Self {

        let texture = state.device.create_texture(&TextureDescriptor {
            label: Some("Effect buffer"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        Self {
            texture,
            view
        }
    }

    pub(crate) fn resize(&mut self, state: &State, width: u32, height: u32) {
         self.texture = state.device.create_texture(&TextureDescriptor {
            label: Some("Effect buffer"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.view = self.texture.create_view(&TextureViewDescriptor::default());
    }

    pub(crate) fn get_view(&self) -> &TextureView {
        &self.view
    }
}