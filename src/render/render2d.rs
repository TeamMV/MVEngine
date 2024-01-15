use std::ffi::c_void;
use std::mem;
use std::ptr::null_mut;
use std::sync::Arc;

use glam::Mat4;
use mvutils::utils::TetrahedronOp;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, Extent3d, IndexFormat,
    RenderPass, TextureDescriptor, TextureDimension, TextureUsages, TextureView,
    TextureViewDescriptor,
};

use crate::render::common::{Bytes, EffectShader, Shader, Texture};
use crate::render::consts::{
    BIND_GROUPS, BIND_GROUP_EFFECT, DEFAULT_SAMPLER, DUMMY_TEXTURE, EFFECT_INDICES, MAX_TEXTURES,
    TEXTURE_LIMIT,
};
use crate::render::init::State;

pub(crate) struct TextureBindGroup {
    pub(crate) bind_group: BindGroup,
    pub(crate) textures: [Arc<Texture>; TEXTURE_LIMIT],
    pub(crate) views: [&'static TextureView; TEXTURE_LIMIT],
}

impl TextureBindGroup {
    pub(crate) fn new(shader: &Shader, state: &State, index: u32) -> Self {
        let textures: [Arc<Texture>; TEXTURE_LIMIT] =
            [0; TEXTURE_LIMIT].map(|_| DUMMY_TEXTURE.clone());
        let views: [&'static TextureView; TEXTURE_LIMIT] =
            [DUMMY_TEXTURE.get_view(); TEXTURE_LIMIT];

        let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(index),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureViewArray(&views[..*MAX_TEXTURES]),
            }],
        });

        Self {
            bind_group,
            textures,
            views,
        }
    }

    pub(crate) fn set(&mut self, index: usize, texture: Arc<Texture>) {
        self.views[index] = unsafe { (texture.get_view() as *const TextureView).as_ref().unwrap() };
        self.textures[index] = texture;
    }

    pub(crate) fn remake(&mut self, state: &State, shader: &Shader, index: u32) {
        self.bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(index),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureViewArray(&self.views[..*MAX_TEXTURES]),
            }],
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
    smoothing: [f32; 1],
    smoothing_buffer: Buffer,
    render_pass: *mut c_void,
}

impl RenderPass2D {
    pub(crate) fn new(shader: Shader, state: &State, projection: Mat4, view: Mat4) -> Self {
        let (vbo, ibo) = state.gen_buffers();

        let uniform = state.gen_uniform_buffer_sized(128);

        let smoothing_buffer = state.gen_uniform_buffer_sized(4);

        state
            .queue
            .write_buffer(&uniform, 0, projection.cast_bytes());
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
                    resource: smoothing_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&DEFAULT_SAMPLER),
                },
            ],
        });

        let texture_group = TextureBindGroup::new(&shader, state, 1);

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
            smoothing: [1.0f32],
            smoothing_buffer,
            render_pass: null_mut(),
        }
    }

    pub(crate) fn new_frame(
        &mut self,
        render_pass: &mut RenderPass,
        projection: Mat4,
        view_matrix: Mat4,
    ) {
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

    pub(crate) fn set_smoothing(&mut self, smoothing: f32) {
        self.smoothing = [smoothing];
        self.state.queue.write_buffer(&self.smoothing_buffer, 0, self.smoothing.as_slice().cast_bytes());
    }

    pub(crate) fn render(
        &mut self,
        indices: &[u32],
        vertices: &[f32],
        textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT],
        stripped: bool,
    ) {
        unsafe {
            if self.ibo.len() <= self.pass {
                let (vbo, ibo) = self.state.gen_buffers();
                self.vbo.push(vbo);
                self.ibo.push(ibo);
                self.texture_groups
                    .push(TextureBindGroup::new(&self.shader, self.state, 1));
            }
            let ibo = &self.ibo[self.pass];
            let vbo = &self.vbo[self.pass];
            let texture_group = &mut self.texture_groups[self.pass];
            let render_pass = (self.render_pass as *mut RenderPass)
                .as_mut()
                .expect("You need to call RenderPass2D::new_frame() before rendering!");

            self.state.queue.write_buffer(ibo, 0, indices.cast_bytes());
            self.state.queue.write_buffer(vbo, 0, vertices.cast_bytes());

            let mut changed = false;

            for (i, texture) in textures.iter().enumerate().take(*MAX_TEXTURES) {
                if let Some(ref texture) = texture {
                    if &texture_group.textures[i] != texture {
                        texture_group.set(i, texture.clone());
                        changed = true;
                    }
                } else if texture_group.textures[i] != DUMMY_TEXTURE.clone() {
                    texture_group.set(i, DUMMY_TEXTURE.clone());
                    changed = true;
                }
            }

            if changed {
                texture_group.remake(self.state, &self.shader, 1);
            }

            render_pass.set_bind_group(1, &texture_group.bind_group, &[]);
            render_pass.set_pipeline(stripped.yn(
                self.shader.get_stripped_pipeline(),
                self.shader.get_pipeline(),
            ));
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
    state: &'static State,
    render_pass: *mut c_void,
    ibo: Vec<Buffer>,
    vbo: Buffer,
    pass: usize,
    bind_group_a: BindGroup,
    bind_group_b: BindGroup,
    uniform: Buffer,
}

impl EffectPass {
    pub(crate) fn new(state: &State, buffer: &EBuffer) -> Self {
        let ibo = state.gen_ibo_sized(24);
        let vbo = state.gen_vbo_sized(0);
        let uniform = state.gen_uniform_buffer_sized(16);
        let bind_group_a = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Effect bind group"),
            layout: BIND_GROUPS
                .get(&BIND_GROUP_EFFECT)
                .expect("Cannot find effect bind group!"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(buffer.get_read()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&DEFAULT_SAMPLER),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform.as_entire_binding(),
                },
            ],
        });
        let bind_group_b = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Effect bind group"),
            layout: BIND_GROUPS
                .get(&BIND_GROUP_EFFECT)
                .expect("Cannot find effect bind group!"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(buffer.get_write()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&DEFAULT_SAMPLER),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform.as_entire_binding(),
                },
            ],
        });
        EffectPass {
            state: unsafe { (state as *const State).as_ref() }.unwrap(),
            render_pass: null_mut(),
            ibo: vec![ibo],
            vbo,
            pass: 0,
            bind_group_a,
            bind_group_b,
            uniform,
        }
    }

    pub(crate) fn rebind(&mut self, state: &State, buffer: &EBuffer) {
        self.bind_group_a = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Effect bind group"),
            layout: BIND_GROUPS
                .get(&BIND_GROUP_EFFECT)
                .expect("Cannot find effect bind group!"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(buffer.get_read()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&DEFAULT_SAMPLER),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.uniform.as_entire_binding(),
                },
            ],
        });
        self.bind_group_b = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Effect bind group"),
            layout: BIND_GROUPS
                .get(&BIND_GROUP_EFFECT)
                .expect("Cannot find effect bind group!"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(buffer.get_write()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&DEFAULT_SAMPLER),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.uniform.as_entire_binding(),
                },
            ],
        });
    }

    pub(crate) fn swap(&mut self) {
        mem::swap(&mut self.bind_group_a, &mut self.bind_group_b);
    }

    pub(crate) fn new_frame(&mut self, time: f32, width: u32, height: u32) {
        self.state.queue.write_buffer(
            &self.uniform,
            0,
            [width as f32, height as f32, time].as_slice().cast_bytes(),
        );
    }

    pub(crate) fn new_target(&mut self, render_pass: &mut RenderPass) {
        self.render_pass = render_pass as *mut RenderPass as *mut c_void;
        unsafe {
            (self.render_pass as *mut RenderPass)
                .as_mut()
                .unwrap()
                .set_vertex_buffer(0, self.vbo.slice(..))
        };
        unsafe {
            (self.render_pass as *mut RenderPass)
                .as_mut()
                .unwrap()
                .set_bind_group(0, &self.bind_group_a, &[])
        };
    }

    pub(crate) fn render(&mut self, shader: Arc<EffectShader>) {
        unsafe {
            if self.ibo.len() <= self.pass {
                self.ibo.push(self.state.gen_ibo_sized(24));
            }
            let ibo = &self.ibo[self.pass];
            let render_pass = (self.render_pass as *mut RenderPass)
                .as_mut()
                .expect("You need to call EffectPass::new_target() before rendering!");

            self.state
                .queue
                .write_buffer(ibo, 0, EFFECT_INDICES.as_slice().cast_bytes());

            render_pass.set_pipeline(shader.get_pipeline());
            render_pass.set_bind_group(1, shader.get_uniforms(), &[]);
            render_pass.set_index_buffer(ibo.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..6, 0, 0..1);
            self.pass += 1;
        }
    }

    pub(crate) fn finish(&mut self) {
        self.render_pass = null_mut();
        self.pass = 0;
    }
}

pub(crate) struct EBuffer {
    read_texture: wgpu::Texture,
    write_texture: wgpu::Texture,
    read: TextureView,
    write: TextureView,
}

impl EBuffer {
    pub(crate) fn generate(state: &State, width: u32, height: u32) -> Self {
        let read_texture = state.device.create_texture(&TextureDescriptor {
            label: Some("Effect buffer A"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: state.config.format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let write_texture = state.device.create_texture(&TextureDescriptor {
            label: Some("Effect buffer B"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: state.config.format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let read = read_texture.create_view(&TextureViewDescriptor::default());

        let write = write_texture.create_view(&TextureViewDescriptor::default());

        Self {
            read_texture,
            write_texture,
            read,
            write,
        }
    }

    pub(crate) fn resize(&mut self, state: &State, width: u32, height: u32) {
        self.read_texture = state.device.create_texture(&TextureDescriptor {
            label: Some("Effect buffer A"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: state.config.format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.write_texture = state.device.create_texture(&TextureDescriptor {
            label: Some("Effect buffer B"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: state.config.format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.read = self
            .read_texture
            .create_view(&TextureViewDescriptor::default());
        self.write = self
            .write_texture
            .create_view(&TextureViewDescriptor::default());
    }

    pub(crate) fn swap(&mut self) {
        mem::swap(&mut self.read_texture, &mut self.write_texture);
        mem::swap(&mut self.read, &mut self.write);
    }

    pub(crate) fn get_read(&self) -> &TextureView {
        &self.read
    }

    pub(crate) fn get_write(&self) -> &TextureView {
        &self.write
    }
}
