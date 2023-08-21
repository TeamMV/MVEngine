use std::sync::Arc;

use glam::{Mat4, Vec2, Vec3};

use crate::render::color::{Color, RGB};
use crate::render::common::Texture;
use crate::render::consts::{
    MAX_TEXTURES, VERTEX_3D_MODEL_SIZE_FLOATS, VERTEX_LAYOUT_MODEL_3D_MAT_ID_OFFSET,
};

pub struct Light {
    direction: Vec3,
    position: Vec<f32>,
    attenuation: f32,
    color: Color<RGB, f32>,
}

pub struct ModelArray {
    amount: usize,
    model: Model,
    transforms: Vec<Mat4>,
}

pub struct Model {
    pub(crate) mesh: Mesh,
    pub(crate) materials: Vec<Material>,
}

impl Model {
    //pub fn vertices_f32(&self) -> Vec<f32> {
    //    let mut vec: Vec<f32> = vec![];
    //    for vertex in self.mesh.vertices.iter() {
    //        vec.push(vertex.x);
    //        vec.push(vertex.y);
    //        vec.push(vertex.z);
    //    }
    //    vec
    //}

    //pub(crate) fn data_array(&self) -> Vec<f32> {
    //    //pos, normal, uv, mat_id
    //    let mut vec: Vec<f32> = vec![];
    //    for m_data in self.mesh {
    //        vec.push(m_data.1.x);
    //        vec.push(m_data.1.y);
    //        vec.push(m_data.1.z);
    //        vec.push(m_data.2.x);
    //        vec.push(m_data.2.y);
    //        vec.push(m_data.2.z);
    //        vec.push(m_data.3.x);
    //        vec.push(m_data.3.y);
    //        vec.push(m_data.4 as f32);
    //    }
    //    vec
    //}

    pub fn recalculate(&mut self) {
        let mut iter = self
            .materials
            .iter();
        iter.nth(VERTEX_LAYOUT_MODEL_3D_MAT_ID_OFFSET);
        let mut iter = iter.step_by(VERTEX_3D_MODEL_SIZE_FLOATS);
        while let Some(mat_id) = iter.next() {}
    }
}

pub struct Mesh {
    pub(crate) name: String,
    pub(crate) data: Vec<f32>,
}

impl Mesh {
    //pub(crate) fn enumerate(&self) -> Vec<(usize, Vec3, Vec3, Vec2, u16)> {
    //    let mut vec: Vec<(usize, Vec3, Vec3, Vec2, u16)> = vec![];
    //    for i in 0..self.vertices.len() {
    //        vec.push((
    //            i,
    //            self.vertices[i],
    //            self.normals[i],
    //            self.tex_coords[i],
    //            self.materials[i],
    //        ));
    //    }
    //    vec
    //}
}

pub struct Material {
    pub ambient: Color<RGB, f32>,
    //Ka
    pub diffuse: Color<RGB, f32>,
    //Kd
    pub specular: Color<RGB, f32>,
    //Ks (specular reflectivity)
    pub emission: Color<RGB, f32>,

    pub alpha: f32,
    //d or Ts
    pub specular_exponent: f32,
    //Ns (specular exponent)
    pub metallic: f32,
    //m
    pub roughness: f32,

    //pub transmission_filter: f32, //Tf
    //pub illumination: u32, //illum
    //pub sharpness: i32, //sharpness
    //pub optical_density: f32, //Ni
    //pub alpha_mode: AlphaMode,
    //pub alpha_cutoff: f32,
    //pub double_side: bool,
    pub diffuse_texture: Option<Arc<Texture>>,
    //map_Kd
    pub metallic_roughness_texture: Option<Arc<Texture>>,
    pub normal_texture: Option<Arc<Texture>>, //norm

                                              //pub specular_texture: Option<Arc<Texture>>, //map_Ks
                                              //pub occlusion_texture: Option<Arc<Texture>>, //map_d
                                              //pub reflection_texture: Option<Arc<Texture>>, //refl
                                              //pub bump_texture: Option<Arc<Texture>>, //bump
                                              //pub emission_texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn new() -> Self {
        Material {
            ambient: Color::<RGB, f32>::new(0.2, 0.2, 0.2, 1.0),
            diffuse: Color::<RGB, f32>::new(0.8, 0.8, 0.8, 1.0),
            specular: Color::<RGB, f32>::white(),
            alpha: 1.0,
            //transmission_filter: 0.0,
            specular_exponent: 0.0,
            //illumination: 1,
            //sharpness: 0,
            //optical_density: 0.0,
            diffuse_texture: None,
            //specular_texture: None,
            //occlusion_texture: None,
            //reflection_texture: None,
            normal_texture: None,
            //double_side: false,
            metallic_roughness_texture: None,
            metallic: 1.0,
            roughness: 1.0,
            emission: Color::<RGB, f32>::black(),
            //emission_texture: None,
            //alpha_mode: AlphaMode::Opaque,
            //alpha_cutoff: 0.5,
        }
    }

    pub fn texture_count(&self, texture_type: TextureType) -> u32 {
        let mut sum = 0;
        if self.diffuse_texture.is_some() && texture_type.is_geometry() {
            sum += 1;
        }
        if self.normal_texture.is_some() && texture_type.is_geometry() {
            sum += 1;
        }
        if self.metallic_roughness_texture.is_some() && texture_type.is_geometry() {
            sum += 1;
        }
        sum
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    pub fn vertex_count(&self) -> u32 {
        todo!()
    }

    pub fn is_simple_geometry(&self) -> bool {
        self.vertex_count() < 5000
            && self.materials.len() < 16
            && self.texture_count(TextureType::Geometry) <= *MAX_TEXTURES as u32 / 4
    }

    pub fn texture_count(&self, texture_type: TextureType) -> u32 {
        self.materials
            .iter()
            .fold(0, |t, m| t + m.texture_count(texture_type))
    }

    pub fn single_batch(&self) -> bool {
        self.texture_count(TextureType::Geometry) <= *MAX_TEXTURES as u32
    }

    pub fn min_batches(&self) -> u32 {
        (self.texture_count(TextureType::Geometry) as f32 / *MAX_TEXTURES as f32).ceil() as u32
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TextureType {
    Any,
    Geometry,
    Lighting,
}

impl TextureType {
    pub fn is_geometry(&self) -> bool {
        self == &TextureType::Geometry || self == &TextureType::Any
    }

    pub fn is_lighting(&self) -> bool {
        self == &TextureType::Lighting || self == &TextureType::Any
    }
}
