use std::any::{Any, TypeId};
use std::collections::HashMap;
use glam::{IVec3, Vec2, Vec3};
use gltf::buffer::View;
use gltf::Gltf;
use itertools::Itertools;
use mvutils::utils::{Binary, TetrahedronOp};
use crate::assets::AssetManager;
use crate::render::color::{Color, RGB};

pub struct Model {
    pub mesh: Mesh,
}

pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vec3>,
    pub indices: Vec<(u16, u16)>,
    pub normals: Vec<Vec3>,
    pub tex_coords: Vec<Vec2>,
    pub materials: Vec<Material>,
}

pub struct Material {
    double_side: bool,
    color: Color<RGB, f32>,
    metallic_fac: f32,
    roughness_fac: f32
}

impl Model {
    pub fn vertex_count(&self) -> u32 {
        self.mesh.vertices.len() as u32
    }
}

pub struct OBJModelLoader;

impl OBJModelLoader {
    pub fn new() -> Self { OBJModelLoader }

    pub fn load_model(&self, data: &str, path: &str, manager: &mut AssetManager) -> Model {

        fn process_face(data: &str) -> IVec3 {
            let tokens = data.split("/").collect::<Vec<_>>();
            let pos = tokens[0].parse::<i32>().unwrap();
            let mut coords = -1;
            let mut normal = -1;
            if tokens.len() > 1 {
                coords = (tokens[1].len() > 0).yn(tokens[1].parse::<i32>().unwrap() - 1, -1);
                if tokens.len() > 2 {
                    normal = tokens[2].parse::<i32>().unwrap();
                }
            }
            IVec3 {
                x: pos,
                y: coords,
                z: normal
            }
        }

        let mut vertices: Vec<Vec3> = Vec::new();
        let mut normals: Vec<Vec3> = Vec::new();
        let mut textures: Vec<Vec2> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut faces: Vec<IVec3> = Vec::new();
        let mut material_map: HashMap<String, u16> = HashMap::new();
        let mut materials: Vec<Material> = Vec::new();

        for line in data.lines() {
            let tokens = line.split_whitespace().collect::<Vec<&str>>();
            if tokens.len() == 0 {
                continue;
            }
            match tokens[0] {
                "mtllib" => {
                    let full_path = path + tokens[1];
                    let file = manager.files.remove(full_path).expect(format!("Mtl file {} not found!", full_path).as_str());
                    self.load_materials(
                        file.contents_utf8().expect(format!("Illegal mtl file format '{}'!", full_path).as_str()),
                        &mut material_map,
                        &mut materials
                    );
                }
                "usemtl" => {

                }
                "v" => {
                    vertices.push(Vec3::new(
                        tokens[1].parse::<f32>().unwrap(),
                        tokens[2].parse::<f32>().unwrap(),
                        tokens[3].parse::<f32>().unwrap(),
                    ));
                }
                "vn" => {
                    normals.push(Vec3::new(
                        tokens[1].parse::<f32>().unwrap(),
                        tokens[2].parse::<f32>().unwrap(),
                        tokens[3].parse::<f32>().unwrap(),
                    ));
                }
                "vt" => {
                    textures.push(Vec2::new(
                        tokens[1].parse::<f32>().unwrap(),
                        tokens[2].parse::<f32>().unwrap(),
                    ));
                }
                "f" => {
                    match tokens.len() {
                        4 => {
                            faces.push(process_face(tokens[1]));
                            faces.push(process_face(tokens[2]));
                            faces.push(process_face(tokens[3]));
                        }
                        5 => {
                            faces.push(process_face(tokens[1]));
                            faces.push(process_face(tokens[2]));
                            faces.push(process_face(tokens[3]));
                            faces.push(process_face(tokens[1]));
                            faces.push(process_face(tokens[3]));
                            faces.push(process_face(tokens[4]));
                        }
                        _ => {
                            panic!("Invalid amount of vertices per face!")
                        }
                    }
                }
                _ => {}
            }
        }
        todo!()
    }

    fn load_materials(&self, data: &str, map: &mut HashMap<String, u16>, materials: &mut Vec<Material>) {

    }
}

pub struct GLTFModelLoader;

impl GLTFModelLoader {
    pub fn new() -> Self { GLTFModelLoader }

    pub fn load_model(&self, data: Binary) -> Model {
        let gltf = Gltf::from_slice(data.as_slice()).expect("There was a Problem loading a 3d-Asset!");
        let mut materials: Vec<Material> = Vec::new();
        //for material in gltf.materials() {
        //    let double_side: bool = material.double_sided();
        //    let mut color: Color<RGB, f32>;
        //    let mut metallic_fac: f32;
        //    let mut roughness_fac: f32;
        //}

        for mesh in gltf.meshes() {
            let name = mesh.name().unwrap_or("").to_string();
            let vertices = self.construct_vec3s(self.get_data_from_buffer_view(&gltf, mesh.primitives().next().unwrap().attributes().nth(0).unwrap().1.index()));
            let normals = self.construct_vec3s(self.get_data_from_buffer_view(&gltf, mesh.primitives().next().unwrap().attributes().nth(1).unwrap().1.index()));
            let tex_coords = self.construct_vec2s(self.get_data_from_buffer_view(&gltf, mesh.primitives().next().unwrap().attributes().nth(2).unwrap().1.index()));
            let indices = self.construct::<u16>(self.get_data_from_buffer_view(&gltf, mesh.primitives().next().unwrap().indices().unwrap().index()))
                .into_iter().map(|i| {(i, materials[mesh.primitives().next().unwrap().material().unwrap().index()])}).collect_vec();
            println!("{}", name);
            let parsed_mesh = Mesh {
                name,
                vertices,
                indices,
                normals,
                tex_coords,
                materials
            };
            return Model {
                mesh: parsed_mesh
            }
        }

        unreachable!()
    }

    fn get_data_from_buffer_view(&self, gltf: &Gltf, idx: usize) -> Binary {
        gltf.views().filter_map(|v| if v.index() == idx {
            Some(gltf.blob.clone().expect("No data present in this gltf file!")[v.offset()..v.offset() + v.length()].to_vec())
        } else { None }).next().expect("This buffer view does not exist!")
    }

    fn construct_vec2s(&self, data: Binary) -> Vec<Vec2> {
        if data.len() % 8 != 0 {panic!("invalid byte size for vec3: {}", data.len())}
        data.into_iter().chunks(8).into_iter().map(|c| {
            let vec = c.chunks(4).into_iter().map(|f| {
                let float = f.collect_vec();
                f32::from_le_bytes([float[0], float[1], float[2], float[3]])
            }).collect_vec();
            Vec2::new(vec[0], vec[1])
        }).collect_vec()
    }

    fn construct_vec3s(&self, data: Binary) -> Vec<Vec3> {
        if data.len() % 12 != 0 {panic!("invalid byte size for vec3: {}", data.len())}
        data.into_iter().chunks(12).into_iter().map(|c| {
            let vec = c.chunks(4).into_iter().map(|f| {
                let float = f.collect_vec();
                f32::from_le_bytes([float[0], float[1], float[2], float[3]])
            }).collect_vec();
            Vec3::new(vec[0], vec[1], vec[2])
        }).collect_vec()
    }

    fn construct<T: FromLeBytes>(&self, data: Binary) -> Vec<T> {
        if data.len() % T::byte_count() != 0 {panic!("invalid byte size for {}: {}", T::name(), data.len())}
        data.into_iter().chunks(T::byte_count()).into_iter().map(|c| {
            let t = c.collect_vec();
            T::from_le_bytes(&t[0..T::byte_count()])
        }).collect_vec()
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


