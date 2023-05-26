use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::Arc;
use glam::{Mat4, Vec2, Vec4};
use mvutils::utils::TetrahedronOp;
use mvutils::unsafe_utils::{Nullable, UnsafeRef};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Buffer, BufferDescriptor, BufferUsages, Color, CommandEncoder, IndexFormat, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureView};
use crate::render::common::{Bytes, Shader, Texture};
use crate::render::consts::{BIND_GROUP_EFFECT, DEFAULT_SAMPLER, DUMMY_TEXTURE, EFFECT_VERT, MAX_TEXTURES, TEXTURE_LIMIT, VERTEX_LAYOUT_NONE};
use crate::render::init::State;
use crate::render::render::TextureBindGroup;
use crate::render::render3d::RenderPass3D;

const LIGHT_PASS_SHADER_FRAG_RAW: &str = include_str!("shaders/light.frag");

pub(crate) struct DeferredPass {
    state: &'static State,
    shader: Shader,
    vbo: Vec<Buffer>,
    ibo: Vec<Buffer>,
    texture_groups: Vec<TextureBindGroup>,
    uniform: BindGroup,
    uniform_buffer: Buffer,
    geom_pass: Option<UnsafeRef<RenderPass<'static>>>,
    light_pass: Option<UnsafeRef<RenderPass<'static>>>,
    projection: Mat4,
    view: Mat4,
    pass: usize,
    sibo: Buffer,
    sbind_group: Option<BindGroup>,

    albedo: Texture,
    position: Texture,
    normal: Texture,
    depth: Texture,

    light_shader: Shader
}

impl DeferredPass {
    pub(crate) fn new(shader: Shader, state: &State) -> Self {
        let (vbo, ibo) = state.gen_buffers();

        let uniform = state.gen_uniform_buffer_sized(128);

        state.queue.write_buffer(&uniform, 0, Mat4::default().cast_bytes());
        state.queue.write_buffer(&uniform, 64, Mat4::default().cast_bytes());

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
                    resource: BindingResource::Sampler(unsafe { DEFAULT_SAMPLER.as_ref().unwrap() }),
                }
            ],
        });

        let texture_group = TextureBindGroup::new(&shader, state);

        let mut inst = Self {
            state: unsafe { (state as *const State).as_ref() }.unwrap(),
            shader,
            vbo: vec![vbo],
            ibo: vec![ibo],
            texture_groups: vec![texture_group],
            uniform: bind_group,
            uniform_buffer: uniform,
            geom_pass: None,
            light_pass: None,
            projection: Mat4::default(),
            view: Mat4::default(),
            pass: 0,
            sibo: state.gen_ibo(),
            sbind_group: None,

            albedo: Texture::buffer(state),
            position: Texture::buffer(state),
            normal: Texture::buffer(state),
            depth: Texture::gen_depth(state),

            light_shader: Shader::new_glsl(EFFECT_VERT, LIGHT_PASS_SHADER_FRAG_RAW).setup_pipeline(state, VERTEX_LAYOUT_NONE, &[BIND_GROUP_EFFECT])
        };

        inst.sbind_group = Some(state.device.create_bind_group(&BindGroupDescriptor {
            label: Some("lighting pass bind group"),
            layout: &inst.light_shader.get_pipeline().get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(inst.position.get_view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(inst.normal.get_view()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(inst.albedo.get_view()),
                }
            ],
        }));

        inst
    }

    fn begin_geom(&self, enc: &mut CommandEncoder) -> UnsafeRef<RenderPass> {
        let geom = enc.begin_render_pass(&RenderPassDescriptor {
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

        UnsafeRef::new(geom)
    }

    fn begin_light(&self, enc: &mut CommandEncoder, target: &TextureView) -> UnsafeRef<RenderPass> {
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

        Nullable::new(light)
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

        unsafe { self.geom_pass.as_mut().unwrap().set_bind_group(0, &self.uniform, &[]) };
    }

    pub(crate) fn render(&mut self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT], stripped: bool, instances: u32, target_view: &TextureView, enc: &mut CommandEncoder) {
        unsafe {
            //geom pass
            self.geom_pass = Some(self.begin_geom(enc).revive());

            if self.ibo.len() <= self.pass {
                let (vbo, ibo) = self.state.gen_buffers();
                self.vbo.push(vbo);
                self.ibo.push(ibo);
                self.texture_groups.push(TextureBindGroup::new(&self.shader, self.state));
            }
            let ibo = &self.ibo[self.pass];
            let vbo = &self.vbo[self.pass];
            let texture_group = &mut self.texture_groups[self.pass];
            let mut render_pass = self.geom_pass.as_mut().expect("You need to call RenderPass2D::new_frame() before rendering!");

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

            render_pass.set_bind_group(1, &texture_group.bind_group, &[]);
            render_pass.set_pipeline(stripped.yn(self.shader.get_stripped_pipeline(), self.shader.get_pipeline()));
            render_pass.set_vertex_buffer(0, vbo.slice(..));
            render_pass.set_index_buffer(ibo.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..instances);
            self.pass += 1;

            //lighting pass
            self.state.queue.write_buffer(&self.sibo, 0, &[0, 1, 2, 0, 2, 3].as_slice().cast_bytes());

            self.light_pass = Some(self.begin_light(enc, target_view).revive());
            render_pass = self.light_pass.as_mut().expect("You need to call RenderPass2D::new_frame() before rendering!");

            render_pass.set_bind_group(1, &self.sbind_group.unwrap(), &[]);
            render_pass.set_pipeline(self.light_shader.get_pipeline());
            render_pass.set_index_buffer(self.sibo.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..1);
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