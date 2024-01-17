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
use crate::render::common3d::{Material, DUMMY_MATERIAL, TextureType};
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

    fn check_texture(&mut self, change: &mut bool, id: &mut u16, tex: &Option<Arc<Texture>>) {
        if *id != 0 {
            *change = true;
            *id = if let Some(t) = tex { t.get_id() } else { 0 } as u16;
        }
    }
}

pub(crate) struct MaterialTextureComBindEdGroup {
    pub(crate) bind_group: BindGroup,
    pub(crate) textures: [Arc<Texture>; TEXTURE_LIMIT],
    pub(crate) views: [&'static TextureView; TEXTURE_LIMIT],
    pub(crate) materials: [Arc<Material>; MATERIAL_LIMIT],
    pub(crate) raw_materials: [[f32; Material::SIZE_FLOATS]; MATERIAL_LIMIT],
    pub(crate) material_buffer: Buffer,
}

impl MaterialTextureComBindEdGroup {
    pub(crate) fn new(shader: &Shader, state: &State, index: u32) -> Self {
        let textures: [Arc<Texture>; TEXTURE_LIMIT] =
            [0; TEXTURE_LIMIT].map(|_| DUMMY_TEXTURE.clone());
        let views: [&'static TextureView; TEXTURE_LIMIT] =
            [DUMMY_TEXTURE.get_view(); TEXTURE_LIMIT];
        let materials = [0; MATERIAL_LIMIT].map(|_| DUMMY_MATERIAL.clone());
        let raw_materials = [0; MATERIAL_LIMIT].map(|_| DUMMY_MATERIAL.raw_data());
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
            views,
            materials,
            raw_materials,
            material_buffer,
        }
    }

    pub(crate) fn set_texture(&mut self, index: usize, texture: Arc<Texture>) {
        self.views[index] = unsafe { (texture.get_view() as *const TextureView).as_ref().unwrap() };
        self.textures[index] = texture;
    }

    pub(crate) fn set_material(&mut self, index: usize, material: Arc<Material>, texture_type: TextureType) {
        self.materials[index] = material;
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

            let mut changed = false;

            for (i, material) in materials.iter().enumerate().take(*MAX_MATERIALS) {
                if let Some(ref material) = material {
                    if &texture_group.materials[i] != material {
                        //remake all the textures if needed
                        self.check_texture(&mut changed, &mut material.diffuse_id,    &material.diffuse_texture);
                        self.check_texture(&mut changed, &mut material.metallic_id,   &material.metallic_roughness_texture);
                        self.check_texture(&mut changed, &mut material.normal_id,     &material.normal_texture);
                        self.check_texture(&mut changed, &mut material.specular_id,   &material.specular_texture);
                        self.check_texture(&mut changed, &mut material.occlusion_id,  &material.occlusion_texture);
                        self.check_texture(&mut changed, &mut material.reflection_id, &material.reflection_texture);
                        self.check_texture(&mut changed, &mut material.bump_id,       &material.bump_texture);
                        self.check_texture(&mut changed, &mut material.emission_id,   &material.emission_texture);
                        //if material.diffuse_id != 0 {
                        //    material.diffuse_id = if let Some(ref tex) = material.diffuse_texture { tex.get_id() } else { 0 } as u16
                        //}

                        texture_group.set_material(i, material.clone(), TextureType::Any);//i assume Any since this is the combined pass
                        changed = true;
                    }
                } else if texture_group.materials[i] != DUMMY_MATERIAL.clone() {
                    texture_group.set_material(i, DUMMY_MATERIAL.clone(), TextureType::Any);
                    changed = true;
                }
            }

            if changed {
                texture_group.remake(self.state, &self.shader, 1);
            }

            render_pass.set_bind_group(2, &texture_group.bind_group, &[]);
            render_pass.set_pipeline(self.shader.get_pipeline());
            render_pass.set_vertex_buffer(0, vbo.slice(..));
            render_pass.set_index_buffer(ibo.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..num_instances);
            self.pass += 1;
        }
    }
}