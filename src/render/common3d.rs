use std::sync::Arc;

use glam::{Mat4, Vec2, Vec3};
use crate::err::panic;

use crate::render::color::{Color, RGB};
use crate::render::common::Texture;
use crate::render::consts::{MATERIAL_OFFSET, MATRIX_OFFSET, MAX_TEXTURES, VERT_LIMIT, VERTEX_LAYOUT_3D};

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
    pub fn texture_count(&self, texture_type: TextureType) -> u32 {
        self.materials.iter().map(|mat| mat.texture_count(texture_type)).sum()
    }

    pub fn vertex_count(&self) -> u32 {
        self.mesh.vertex_count()
    }

    pub fn prepare(&mut self) {
        self.mesh = self.mesh.prepare();
    }

    pub fn is_simple_geometry(&self) -> bool {
        self.vertex_count() < 5000
            && self.materials.len() < 16
            && self.texture_count(TextureType::Geometry) <= *MAX_TEXTURES as u32 / 4
    }

    pub fn single_batch(&self) -> bool {
        self.texture_count(TextureType::Geometry) <= *MAX_TEXTURES as u32
    }

    pub fn min_batches(&self) -> u32 {
        (self.texture_count(TextureType::Geometry) as f32 / *MAX_TEXTURES as f32).ceil()
            .max(
                (self.vertex_count() as f32 / VERT_LIMIT as f32).ceil()
            ) as u32
    }
}

pub enum Mesh {
    Raw(RawMesh),
    Prepared(PreparedMesh)
}

impl Mesh {
    pub fn vertex_count(&self) -> u32 {
        match self {
            Mesh::Raw(raw) => (raw.vertices.len() / 3) as u32,
            Mesh::Prepared(prepared) => prepared.vert_count
        }
    }

    pub fn prepare(self) -> Mesh {
        match self {
            Mesh::Raw(raw) => Mesh::Prepared(raw.prepare()),
            Mesh::Prepared(prepared) => Mesh::Prepared(prepared)
        }
    }

    pub fn setup_matrix(&mut self, matrix_id: u32) {
        match self {
            Mesh::Raw(raw) => {}
            Mesh::Prepared(prepared) => {}
        }
    }

    pub fn setup_materials(&mut self, material_ids: &[u32]) {

    }
}

pub struct RawMesh {
    pub(crate) name: String,
    pub(crate) indices: Vec<(u32, u16)>,//idx, mat
    pub(crate) vertices: Vec<f32>,
    pub(crate) normals: Vec<f32>,
    pub(crate) tex_coords: Vec<f32>,
}

impl RawMesh {
    fn prepare(self) -> PreparedMesh {
        if self.indices.len() == 0 {
            return PreparedMesh {
                name,
                data: Vec::new(),
                vert_count: 0
            }
        }

        let mut data = Vec::new();
        let mut last_mat = self.indices[0].1;
        let mut count = 0;
        let mut index_offset = 0;
        let mut current = (Vec::new(), Vec::new(), last_mat, 0);

        for index in self.indices {
            if last_mat != index.1 {
                last_mat = index.1;
                current.3 = count;
                data.push(current);
                index_offset += count;
                count = 0;
                current = (Vec::new(), Vec::new(), last_mat, 0);
            }
            let actual_index = index.0 - index_offset;
            current.0.push(actual_index);

            if (count <= actual_index) {
                count += 1;
                current.1.extend_from_slice(&[
                    self.vertices[index.0 * 3],
                    self.vertices[index.0 * 3 + 1],
                    self.vertices[index.0 * 3 + 2],
                    self.normals[index.0 * 3],
                    self.normals[index.0 * 3 + 1],
                    self.normals[index.0 * 3 + 2],
                    self.tex_coords[index.0 * 2],
                    self.tex_coords[index.0 * 2 + 1],
                    0.0,
                    0.0
                ]);
            }
        }

        PreparedMesh {
            name,
            data,
            vert_count: index_offset
        }
    }
}

pub struct PreparedMesh {
    name: String,
    data: Vec<(Vec<u32>, Vec<f32>, u16, u32)>,
    vert_count: u32
}

impl PreparedMesh {
    pub fn setup_matrix(&mut self, matrix_id: u32) {
        for data in self.data.iter_mut() {
            let mut idx = MATRIX_OFFSET as usize;
            while idx < data.1.len() {
                data.1[idx] = matrix_id as f32;
                idx += VERTEX_LAYOUT_3D.array_stride / 4;
            }
        }
    }

    pub fn setup_materials(&mut self, material_ids: &[u32]) {
        for (data, id) in self.data.iter_mut().zip(material_ids.iter().cloned()) {
            let mut idx = MATERIAL_OFFSET as usize;
            while idx < data.1.len() {
                data.1[idx] = id as f32;
                idx += VERTEX_LAYOUT_3D.array_stride / 4;
            }
        }
    }
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

    pub const SIZE_BYTES: usize = 92;

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
