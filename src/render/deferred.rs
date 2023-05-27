use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::Arc;
use glam::{Mat4, Vec2, Vec4};
use json::Null;
use mvutils::utils::TetrahedronOp;
use mvutils::unsafe_utils::{Nullable, Unsafe, UnsafeRef};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Buffer, BufferDescriptor, BufferSlice, BufferUsages, Color, CommandEncoder, IndexFormat, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureView};
use crate::render::common::{Bytes, Shader, Texture};
use crate::render::consts::{BIND_GROUP_EFFECT, BIND_GROUP_LIGHTING_3D, DEFAULT_SAMPLER, DUMMY_TEXTURE, DUMMY_VERT, MAX_LIGHTS, MAX_TEXTURES, TEXTURE_LIMIT, VERTEX_LAYOUT_NONE};
use crate::render::init::State;
use crate::render::render::TextureBindGroup;
use crate::render::render3d::RenderPass3D;

pub(crate) struct DeferredPass {
    state: &'static State,
    shader: Shader,
    vbo: Vec<Buffer>,
    ibo: Vec<Buffer>,
    texture_groups: Vec<TextureBindGroup>,
    uniform: BindGroup,
    uniform_buffer: Buffer,
    material_buffer: Buffer,
    geom_pass: Nullable<RenderPass<'static>>,
    light_pass: Nullable<RenderPass<'static>>,
    projection: Mat4,
    view: Mat4,
    pass: usize,
    sibo: Buffer,

    albedo: Texture,
    position: Texture,
    normal: Texture,
    depth: Texture,

    light_group: BindGroup,
    light_buffer: Buffer,
    light_shader: Shader
}

impl DeferredPass {
    pub(crate) fn new(shader: Shader, state: &State) -> Self {
        let (vbo, ibo) = state.gen_buffers();

        let uniform = state.gen_uniform_buffer_sized(128);

        state.queue.write_buffer(&uniform, 0, Mat4::default().cast_bytes());
        state.queue.write_buffer(&uniform, 64, Mat4::default().cast_bytes());

        let material = state.gen_uniform_buffer_sized(9632);

        let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("geometry pass bind group"),
            layout: &shader.get_pipeline().get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: material.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(unsafe { DEFAULT_SAMPLER.as_ref().unwrap() }),
                }
            ],
        });

        let albedo = Texture::buffer(state);
        let normal = Texture::buffer(state);
        let position = Texture::buffer(state);

        let texture_group = TextureBindGroup::new(&shader, state, 2);

        let light_shader = Shader::new_glsl(DUMMY_VERT, include_str!("shaders/light.frag")).setup_pipeline(state, VERTEX_LAYOUT_NONE, &[BIND_GROUP_LIGHTING_3D]);

        let light_buffer = state.gen_uniform_buffer_sized((32 + 64 * unsafe { MAX_LIGHTS }) as u64);

        let light_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("lighting pass bind group"),
            layout: &light_shader.get_pipeline().get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(albedo.get_view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(normal.get_view()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(position.get_view()),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(unsafe { DEFAULT_SAMPLER.as_ref().unwrap()})
                },
                BindGroupEntry {
                    binding: 4,
                    resource: light_buffer.as_entire_binding()
                }
            ],
        });

        let mut inst = Self {
            state: unsafe { (state as *const State).as_ref() }.unwrap(),
            shader,
            vbo: vec![vbo],
            ibo: vec![ibo],
            texture_groups: vec![texture_group],
            uniform: bind_group,
            uniform_buffer: uniform,
            material_buffer: material,
            geom_pass: Nullable::null(),
            light_pass: Nullable::null(),
            projection: Mat4::default(),
            view: Mat4::default(),
            pass: 0,
            sibo: state.gen_ibo(),

            albedo,
            position,
            normal,
            depth: Texture::gen_depth(state),

            light_group,
            light_buffer,
            light_shader
        };

        inst
    }

    fn begin_geom(&self, enc: &mut CommandEncoder) -> Nullable<RenderPass<'static>> {
        let mut geom = enc.begin_render_pass(&RenderPassDescriptor {
            label: Some("Geometry Pass"),
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: self.position.get_view(),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: true,
                    },
                }),
                Some(RenderPassColorAttachment {
                    view: self.normal.get_view(),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: true,
                    },
                }),
                Some(RenderPassColorAttachment {
                    view: self.albedo.get_view(),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: true,
                    },
                }),
            ],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: self.depth.get_view(),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        unsafe { geom.set_bind_group(0, Unsafe::cast_static(&self.uniform), &[]) };

        unsafe { Nullable::new(geom).cast_bytes() }
    }

    fn begin_light(&self, enc: &mut CommandEncoder, target: &TextureView) -> Nullable<RenderPass<'static>> {
        let light = enc.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lighting Pass"),
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color { //Maybe ill use my Color as well here...
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: false,
                    },
                })
            ],
            depth_stencil_attachment: None,
        });

        unsafe { Nullable::new(light).cast_bytes() }
    }

    pub(crate) fn new_frame(&mut self, proj: Mat4, view: Mat4) {
        self.pass = 0;

        if self.projection != proj {
            self.projection = proj;
            self.state.queue.write_buffer(&self.uniform_buffer, 0, proj.cast_bytes());
        }

        if self.view!= view {
            self.view = view;
            self.state.queue.write_buffer(&self.uniform_buffer, 64, proj.cast_bytes());
        }
    }

    pub(crate) fn render(&mut self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT], stripped: bool, instances: u32, target_view: &TextureView, enc: &mut CommandEncoder) {
        unsafe {
            //geom pass
            self.geom_pass = self.begin_geom(enc);

            if self.ibo.len() <= self.pass {
                let (vbo, ibo) = self.state.gen_buffers();
                self.vbo.push(vbo);
                self.ibo.push(ibo);
                self.texture_groups.push(TextureBindGroup::new(&self.shader, self.state, 2));
            }
            let ibo = &self.ibo[self.pass];
            let vbo = &self.vbo[self.pass];
            let texture_group = &mut self.texture_groups[self.pass];

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

            self.geom_pass.set_bind_group(2, Unsafe::cast_static(&texture_group.bind_group), &[]);
            self.geom_pass.set_pipeline(Unsafe::cast_static(stripped.yn(self.shader.get_stripped_pipeline(), self.shader.get_pipeline())));
            self.geom_pass.set_vertex_buffer(0, Unsafe::cast(vbo.slice(..)));
            self.geom_pass.set_index_buffer(Unsafe::cast(ibo.slice(..)), IndexFormat::Uint32);
            self.geom_pass.draw_indexed(0..indices.len() as u32, 0, 0..instances);
            self.pass += 1;

            //lighting pass
            self.state.queue.write_buffer(&self.sibo, 0, &[0, 1, 2, 0, 2, 3].as_slice().cast_bytes());

            self.light_pass = self.begin_light(enc, target_view);

            self.light_pass.set_bind_group(1, Unsafe::cast_static(&self.light_group), &[]);
            self.light_pass.set_pipeline(Unsafe::cast_static(self.light_shader.get_pipeline()));
            self.light_pass.set_index_buffer(Unsafe::cast(self.sibo.slice(..)), IndexFormat::Uint16);
            self.light_pass.draw_indexed(0..6, 0, 0..1);
        }
    }
}

impl RenderPass3D for DeferredPass {
    fn render_batch(&self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT], transforms: &[Mat4]) {
        todo!()
    }

    fn render_model_instanced(&self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT], canvas: &[f32; 6], transforms: &[Mat4], hum_instances: u32) {
        todo!()
    }
}