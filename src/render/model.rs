use alloc::rc::Rc;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use glam::{IVec3, IVec4, Mat4, Vec2, Vec3};
use gltf::{Gltf, Semantic};
use gltf::buffer::View;
use gltf::material::{AlphaMode, NormalTexture, OcclusionTexture};
use include_dir::File;
use itertools::Itertools;
use mvutils::utils::{Bytecode, TetrahedronOp};

use crate::ApplicationLoopCallbacks;
use crate::render::color::{Color, RGB};
use crate::render::common::Texture;
use crate::render::consts::MAX_TEXTURES;
use crate::render::RenderCore;
use crate::render::window::Window;

pub struct Model {
    pub(crate) mesh: Mesh,
    pub(crate) materials: Vec<Material>
}

impl Model {
    pub fn vertices_f32(&self) -> Vec<f32> {
        let mut vec: Vec<f32> = vec![];
        for vertex in self.mesh.vertices.iter() {
            vec.push(vertex.x);
            vec.push(vertex.y);
            vec.push(vertex.z);
        }
        vec
    }

    pub(crate) fn data_array(&self) -> Vec<f32> {
        //pos, normal, uv, mat_id
        let mut vec: Vec<f32> = vec![];
        for m_data in self.mesh.enumerate() {
            vec.push(m_data.1.x);
            vec.push(m_data.1.y);
            vec.push(m_data.1.z);
            vec.push(m_data.2.x);
            vec.push(m_data.2.y);
            vec.push(m_data.2.z);
            vec.push(m_data.3.x);
            vec.push(m_data.3.y);
            vec.push(m_data.4 as f32);
        }
        vec
    }
}

pub struct Mesh {
    pub(crate) name: String,
    pub(crate) vertices: Vec<Vec3>,
    pub(crate) indices: Vec<u32>,
    pub(crate) normals: Vec<Vec3>,
    pub(crate) tex_coords: Vec<Vec2>,
    pub(crate) materials: Vec<u16>,
}

impl Mesh {
    pub(crate) fn enumerate(&self) -> Vec<(usize, Vec3, Vec3, Vec2, u16)> {
        let mut vec: Vec<(usize, Vec3, Vec3, Vec2, u16)> = vec![];
        for i in 0..self.vertices.len() {
            vec.push((
                i,
                self.vertices[i],
                self.normals[i],
                self.tex_coords[i],
                self.materials[i],)
            );
        }
        vec
    }
}

pub struct Material {
    pub ambient: Color<RGB, f32>, //Ka
    pub diffuse: Color<RGB, f32>, //Kd
    pub specular: Color<RGB, f32>, //Ks (specular reflectivity)
    pub emission: Color<RGB, f32>,

    pub alpha: f32, //d or Ts
    pub specular_exponent: f32, //Ns (specular exponent)
    pub metallic: f32, //m
    pub roughness: f32,

    //pub transmission_filter: f32, //Tf
    //pub illumination: u32, //illum
    //pub sharpness: i32, //sharpness
    //pub optical_density: f32, //Ni
    //pub alpha_mode: AlphaMode,
    //pub alpha_cutoff: f32,
    //pub double_side: bool,

    pub diffuse_texture: Option<Arc<Texture>>, //map_Kd
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
        self.mesh.vertices.len() as u32
    }

    pub fn is_simple_geometry(&self) -> bool {
        self.vertex_count() < 5000
        && self.materials.len() < 16
        && self.texture_count(TextureType::Geometry) <= *MAX_TEXTURES as u32 / 4
    }

    pub fn texture_count(&self, texture_type: TextureType) -> u32 {
        self.materials.iter().fold(0, |t, m| t + m.texture_count(texture_type))
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
    Lighting
}

impl TextureType {
    pub fn is_geometry(&self) -> bool {
        self == &TextureType::Geometry || self == &TextureType::Any
    }

    pub fn is_lighting(&self) -> bool {
        self == &TextureType::Lighting || self == &TextureType::Any
    }
}

pub(crate) struct ModelLoader<I: ApplicationLoopCallbacks + 'static> {
    obj: OBJModelLoader<I>,
    gltf: GLTFModelLoader<I>
}

pub enum ModelFileType {
    Obj,
    Gltf
}

impl<I: ApplicationLoopCallbacks> ModelLoader<I> {
    pub(crate) fn new(win: Arc<Window<I>>) -> Self {
        ModelLoader {
            obj: OBJModelLoader::new(win.clone()),
            gltf: GLTFModelLoader::new(win)
        }
    }

    pub(crate) fn load_model(&self, path: &str, file_type: ModelFileType, code: &str) -> Model {
        match file_type {
            ModelFileType::Obj => self.obj.load_model(code, path),
            ModelFileType::Gltf => self.gltf.load_model(vec![]),
        }
    }
}

struct OBJModelLoader<I: ApplicationLoopCallbacks + 'static> {
    win: Arc<Window<I>>,
}

impl<I: ApplicationLoopCallbacks> OBJModelLoader<I> {
    fn new(win: Arc<Window<I>>) -> Self {
        OBJModelLoader {
            win,
        }
    }

    fn load_model(&self, data: &str, path: &str) -> Model {
        fn process_face(data: &str, material: u16) -> IVec4 {
            let tokens = data.split("/").collect::<Vec<_>>();
            let pos = tokens[0].parse::<i32>().unwrap() - 1;
            let mut coords = -1;
            let mut normal = -1;
            if tokens.len() > 1 {
                coords = (tokens[1].len() > 0).yn(tokens[1].parse::<i32>().unwrap() - 1, -1);
                if tokens.len() > 2 {
                    normal = tokens[2].parse::<i32>().unwrap() - 1;
                }
            }
            IVec4 {
                x: pos,
                y: coords,
                z: normal,
                w: material as i32
            }
        }

        let mut name = String::new();
        let mut vertices: Vec<Vec3> = Vec::new();
        let mut normals_vec: Vec<Vec3> = Vec::new();
        let mut normals: Vec<Vec3> = Vec::new();
        let mut tex_coords_vec: Vec<Vec2> = Vec::new();
        let mut tex_coords: Vec<Vec2> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut materials: Vec<u16> = Vec::new();
        let mut faces: Vec<IVec4> = Vec::new();
        let mut material_map: HashMap<String, u16> = HashMap::new();
        let mut available_materials: Vec<Material> = Vec::new();
        let mut current_material: u16 = 0; //indexing starts at 1, 0 = no material / default

        for line in data.lines() {
            let tokens = line.split_whitespace().collect::<Vec<&str>>();
            if tokens.len() == 0 {
                continue;
            }
            match tokens[0] {
                "mtllib" => {
                    let full_path = path.to_string() + tokens[1];
                    self.load_materials(
                        "",
                        path,
                        &mut material_map,
                        &mut available_materials
                    );
                }
                "o" => {
                    name = tokens[1].to_string();
                }
                "usemtl" => {
                    current_material = *material_map.get(tokens[1]).unwrap_or(&0);
                }
                "v" => {
                    vertices.push(Vec3::new(
                        tokens[1].parse::<f32>().unwrap(),
                        tokens[2].parse::<f32>().unwrap(),
                        tokens[3].parse::<f32>().unwrap(),
                    ));
                }
                "vn" => {
                    normals_vec.push(Vec3::new(
                        tokens[1].parse::<f32>().unwrap(),
                        tokens[2].parse::<f32>().unwrap(),
                        tokens[3].parse::<f32>().unwrap(),
                    ));
                }
                "vt" => {
                    tex_coords_vec.push(Vec2::new(
                        tokens[1].parse::<f32>().unwrap(),
                        tokens[2].parse::<f32>().unwrap(),
                    ));
                }
                "p" => {
                    let face = process_face(tokens[1], current_material);
                    faces.push(face);
                    faces.push(face);
                    faces.push(face);
                }
                "l" => {
                    for i in 1..tokens.len() - 1 {
                        let face = process_face(tokens[i], current_material);
                        faces.push(face);
                        faces.push(face);
                        faces.push(process_face(tokens[i + 1], current_material));
                    }
                }
                "f" => {
                    match tokens.len() {
                        4 => {
                            faces.push(process_face(tokens[1], current_material));
                            faces.push(process_face(tokens[2], current_material));
                            faces.push(process_face(tokens[3], current_material));
                        }
                        5 => {
                            let face = process_face(tokens[1], current_material);
                            let duplicate = process_face(tokens[3], current_material);
                            faces.push(face);
                            faces.push(process_face(tokens[2], current_material));
                            faces.push(duplicate);
                            faces.push(face);
                            faces.push(duplicate);
                            faces.push(process_face(tokens[4], current_material));
                        }
                        _ => {
                            panic!("Invalid amount of vertices per face!")
                        }
                    }
                }
                _ => {}
            }
        }

        for _ in 0..faces.len() {
            tex_coords.push(Vec2::new(0.0, 0.0));
            normals.push(Vec3::new(0.0, 0.0, 0.0));
        }

        for face in faces {
            indices.push(face.x as u32);
            materials.push(face.w as u16);

            if face.y >= 0 {
                let coord = tex_coords_vec[face.y as usize];
                tex_coords[face. x as usize] = Vec2::new(coord.x, 1.0 - coord.y);
            }

            if face.z >= 0 {
                normals[face.x as usize] = normals_vec[face.z as usize];
            }
        }

        available_materials.insert(0, Material::default());

        Model {
            mesh: Mesh {
                name,
                vertices,
                indices,
                normals,
                tex_coords,
                materials,
            },
            materials: available_materials,
        }
    }

    fn load_materials(&self, data: &str, path: &str, map: &mut HashMap<String, u16>, materials: &mut Vec<Material>) {
        let mut name = String::new();
        let mut material = Material::default();

        for line in data.lines() {
            let tokens = line.split_whitespace().collect::<Vec<&str>>();
            if tokens.len() == 0 {
                continue;
            }
            match tokens[0] {
                "newmtl" => {
                    if !name.is_empty() {
                        map.insert(name, materials.len() as u16 + 1);
                        materials.push(material);
                    }
                    name = tokens[1].to_string();
                    material = Material::default();
                }
                "Ka" => {
                    material.ambient.set_r(tokens[1].parse::<f32>().unwrap());
                    material.ambient.set_g(tokens[2].parse::<f32>().unwrap());
                    material.ambient.set_b(tokens[3].parse::<f32>().unwrap());
                }
                "Kd" => {
                    material.diffuse.set_r(tokens[1].parse::<f32>().unwrap());
                    material.diffuse.set_g(tokens[2].parse::<f32>().unwrap());
                    material.diffuse.set_b(tokens[3].parse::<f32>().unwrap());
                }
                "Ks" => {
                    material.specular.set_r(tokens[1].parse::<f32>().unwrap());
                    material.specular.set_g(tokens[2].parse::<f32>().unwrap());
                    material.specular.set_b(tokens[3].parse::<f32>().unwrap());
                }
                "d" => {
                    material.alpha = tokens[1].parse::<f32>().unwrap();
                }
                "Ns" => {
                    material.specular_exponent = tokens[1].parse::<f32>().unwrap();
                }
                "m" => {
                    material.metallic = tokens[1].parse::<f32>().unwrap();
                }
                //"Ni" => {
                //    material.optical_density = tokens[1].parse::<f32>().unwrap();
                //}
                //"illum" => {
                //    material.illumination = tokens[1].parse::<i32>().unwrap();
                //}
                //"Tf" => {
                //    material.transmission_filter = tokens[1].parse::<i32>().unwrap();
                //}
                //"sharpness" => {
                //    material.sharpness = tokens[1].parse::<i32>().unwrap();
                //}
                "map_Kd" => {
                    //material.diffuse_texture = Some(self.load_texture(path.to_string() + tokens[1]));
                }
                //"map_Ks" => {
                //    material.specular_texture = Some(self.load_texture(path + tokens[1]));
                //}
                //"map_d" => {
                //    material.occlusion_texture = Some(self.load_texture(path + tokens[1]));
                //}
                //"refl" => {
                //    material.reflection_texture = Some(self.load_texture(path + tokens[1]));
                //}
                "normal" => {
                    //material.normal_texture = Some(self.load_texture(path.to_string() + tokens[1]));
                }
                _ => {}
            }
        }
        if !name.is_empty() {
            map.insert(name, materials.len() as u16 + 1);
            materials.push(material);
        }
    }

    fn load_texture(&self, bytes: Bytecode) -> Arc<Texture> {
        Arc::new(Texture::new(bytes))
        //if let Some(texture) = unsafe { self.manager.as_mut() }.unwrap().textures.get(path.as_str()) {
        //    texture.clone()
        //}
        //else {
        //    let file = unsafe { self.manager.as_mut() }.unwrap().files.remove(path.as_str()).expect(format!("Texture file {} not present or already loaded with different name!", path).as_str());
        //    let texture = Arc::new(self.core.create_texture(file.contents()));
        //    unsafe { self.manager.as_mut() }.unwrap().textures.insert(path.to_string(), texture.clone());
        //    texture
        //}
    }
}

struct GLTFModelLoader<I: ApplicationLoopCallbacks + 'static> {
    win: Arc<Window<I>>
}

impl<I: ApplicationLoopCallbacks> GLTFModelLoader<I> {
    fn new(win: Arc<Window<I>>) -> Self {
        GLTFModelLoader {
            win
        }
    }

    fn load_model(&self, data: Bytecode) -> Model {
        let gltf = Gltf::from_slice(data.as_slice()).expect("There was a Problem loading a 3d-Asset!");
        let mut materials: Vec<Material> = Vec::new();
        for material in gltf.materials() {
            let mut mat = Material::new();
            //mat.double_side = material.double_sided();
            mat.emission = Color::<RGB, f32>::new(material.emissive_factor()[0], material.emissive_factor()[1], material.emissive_factor()[2], 1.0);
            mat.roughness = material.pbr_metallic_roughness().roughness_factor();
            mat.metallic = material.pbr_metallic_roughness().metallic_factor();
            //mat.alpha_mode = material.alpha_mode();
            //mat.alpha_cutoff = material.alpha_cutoff().unwrap_or(0.5);
            //if let Some(info) = material.emissive_texture() {
            //    mat.emission_texture = Some(rc_mut(self.construct_texture(&gltf, &info)));
            //}
            //if let Some(info) = material.occlusion_texture() {
            //    mat.occlusion_texture = Some(rc_mut(self.construct_texture_occ(&gltf, &info)));
            //}
            if let Some(info) = material.pbr_metallic_roughness().metallic_roughness_texture() {
                mat.metallic_roughness_texture = Some(Arc::new(self.construct_texture(&gltf, &info)));
            }
            if let Some(info) = material.normal_texture() {
                mat.normal_texture = Some(Arc::new(self.construct_texture_nor(&gltf, &info)));
            }
            if let Some(info) = material.pbr_metallic_roughness().base_color_texture() {
                mat.diffuse_texture = Some(Arc::new(self.construct_texture(&gltf, &info)));
            }
        }

        for mesh in gltf.meshes() {
            let name = mesh.name().unwrap_or("").to_string();
            let mut vertices: Vec<Vec3> = Vec::new();
            let mut normals: Vec<Vec3> = Vec::new();
            let mut tex_coords: Vec<Vec2> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();
            for primitive in mesh.primitives() {
                vertices.append(&mut self.construct_vec3s(self.get_data_from_buffer_view(&gltf, primitive.get(&Semantic::Positions).unwrap().index())));
                normals.append(&mut self.construct_vec3s(self.get_data_from_buffer_view(&gltf, primitive.get(&Semantic::Normals).unwrap().index())));
                tex_coords.append(&mut self.construct_vec2s(self.get_data_from_buffer_view(&gltf, primitive.get(&Semantic::TexCoords(0)).unwrap().index())));
                indices.append(&mut self.construct::<u32>(self.get_data_from_buffer_view(&gltf, primitive.indices().unwrap().index())));
            }

            println!("{}", name);
            let parsed_mesh = Mesh {
                name,
                vertices,
                indices,
                normals,
                tex_coords,
                materials: Vec::new()
            };
            return Model {
                mesh: parsed_mesh,
                materials
            }
        }

        unreachable!()
    }

    fn get_data_from_buffer_view(&self, gltf: &Gltf, idx: usize) -> Bytecode {
        gltf.views().filter_map(|v| if v.index() == idx {
            Some(gltf.blob.clone().expect("No data present in this gltf file!")[v.offset()..v.offset() + v.length()].to_vec())
        } else { None }).next().expect("This buffer view does not exist!")
    }

    fn construct_vec2s(&self, data: Bytecode) -> Vec<Vec2> {
        if data.len() % 8 != 0 {panic!("invalid byte size for vec3: {}", data.len())}
        data.into_iter().chunks(8).into_iter().map(|c| {
            let vec = c.chunks(4).into_iter().map(|f| {
                let float = f.collect_vec();
                f32::from_le_bytes([float[0], float[1], float[2], float[3]])
            }).collect_vec();
            Vec2::new(vec[0], vec[1])
        }).collect_vec()
    }

    fn construct_vec3s(&self, data: Bytecode) -> Vec<Vec3> {
        if data.len() % 12 != 0 {panic!("invalid byte size for vec3: {}", data.len())}
        data.into_iter().chunks(12).into_iter().map(|c| {
            let vec = c.chunks(4).into_iter().map(|f| {
                let float = f.collect_vec();
                f32::from_le_bytes([float[0], float[1], float[2], float[3]])
            }).collect_vec();
            Vec3::new(vec[0], vec[1], vec[2])
        }).collect_vec()
    }

    fn construct<T: FromLeBytes>(&self, data: Bytecode) -> Vec<T> {
        if data.len() % T::byte_count() != 0 {panic!("invalid byte size for {}: {}", T::name(), data.len())}
        data.into_iter().chunks(T::byte_count()).into_iter().map(|c| {
            let t = c.collect_vec();
            T::from_le_bytes(&t[0..T::byte_count()])
        }).collect_vec()
    }

    fn construct_texture(&self, gltf: &Gltf, src: &gltf::texture::Info) -> Texture {
        let img_idx = src.texture().source().index();
        let buffer_view = gltf.images().nth(img_idx).unwrap().index();
        let binary = self.get_data_from_buffer_view(gltf, buffer_view);
        self.win.create_texture(binary)
    }

    fn construct_texture_occ(&self, gltf: &Gltf, src: &OcclusionTexture) -> Texture {
        let img_idx = src.texture().source().index();
        let buffer_view = gltf.images().nth(img_idx).unwrap().index();
        let binary = self.get_data_from_buffer_view(gltf, buffer_view);
        self.win.create_texture(binary)
    }

    fn construct_texture_nor(&self, gltf: &Gltf, src: &NormalTexture) -> Texture {
        let img_idx = src.texture().source().index();
        let buffer_view = gltf.images().nth(img_idx).unwrap().index();
        let binary = self.get_data_from_buffer_view(gltf, buffer_view);
        self.win.create_texture(binary)
    }
}

trait FromLeBytes: Sized {
    fn from_le_bytes(bytes: &[u8]) -> Self;
    fn byte_count() -> usize;
    fn name() -> String;
}

impl FromLeBytes for u16 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        u16::from_le_bytes([bytes[0], bytes[1]])
    }

    fn byte_count() -> usize {
        2
    }

    fn name() -> String {
        "UNSIGNED_SHORT".to_string()
    }
}

impl FromLeBytes for u32 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn byte_count() -> usize {
        4
    }

    fn name() -> String {
        "UNSIGNED_INT".to_string()
    }
}

impl FromLeBytes for f32 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn byte_count() -> usize {
        4
    }

    fn name() -> String {
        "FLOAT".to_string()
    }
}


