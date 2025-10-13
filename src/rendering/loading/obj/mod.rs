pub mod token;

use crate::rendering::api::Renderer;
use crate::rendering::backend::buffer::MemoryProperties;
use crate::rendering::backend::image::{ImageUsage, MVImageCreateInfo};
use crate::rendering::implementation::scene::Scene;
use crate::rendering::implementation::scene::material::Material;
use crate::rendering::implementation::scene::model::{SceneModel, StandaloneModel};
use crate::rendering::loading::ModelLoadingError;
use crate::rendering::loading::obj::token::{Command, Token, Tokenizer};
use crate::utils::hashable::{Float, Vec2, Vec3, Vec4};
use gpu_alloc::UsageFlags;
use hashbrown::{HashMap, HashSet};
use include_dir::{Dir, File};
use mvutils::into_peek::{IntoPeekable, NewIntoPeekable};
use std::path::Path;
use std::sync::Arc;
use log::info;
use crate::rendering::implementation::scene::mesh::{Mesh, MeshVertex};

type Tokens<'a> = IntoPeekable<Tokenizer<'a>, Token>;

pub struct OBJModelLoader<'a> {
    directory: Dir<'a>,
    current_scene: Option<Scene>,
    material_map: HashMap<String, u32>,
}

impl<'a> OBJModelLoader<'a> {
    pub fn new(dir: Dir<'a>) -> Self {
        Self {
            directory: dir,
            current_scene: None,
            material_map: HashMap::new(),
        }
    }

    fn grab_file(&self, filename: &str, file_ext: &str) -> Result<&File, ModelLoadingError> {
        let complete_name = format!("{filename}.{file_ext}");
        let path = Path::new(&complete_name);
        self.directory
            .get_file(path)
            .ok_or(ModelLoadingError::MissingFile(complete_name))
    }

    fn grab_file_ext(&self, file: &str) -> Result<&File, ModelLoadingError> {
        let path = Path::new(file);
        self.directory
            .get_file(path)
            .ok_or(ModelLoadingError::MissingFile(file.to_string()))
    }

    pub fn load_scene(&mut self, name: &str, r: &mut Renderer) -> Result<Scene, ModelLoadingError> {
        self.current_scene = Some(Scene::new());

        let obj_file = self.grab_file(name, "obj")?;

        if let Some(contents) = obj_file.contents_utf8() {
            let contents = contents.to_string(); //make borrow checker happy
            let mut tokens = Tokenizer::new(&contents).into_peekable();

            let mut used_material = 0;
            let mut current_obj_name = String::new();

            let mut positions: Vec<Vec3> = Vec::new();
            let mut texcoords: Vec<Vec2> = Vec::new();
            let mut normals: Vec<Vec3> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            let mut vertices: Vec<MeshVertex> = Vec::new();
            let mut used_materials: HashSet<u8> = HashSet::new();

            while let Some(token) = tokens.peek() {
                tokens.next();
                match token.want_command()? {
                    Command::O | Command::G => {
                        if !vertices.is_empty() {
                            let mesh = Mesh {
                                vertices: std::mem::take(&mut vertices),
                                indices: std::mem::take(&mut indices),
                            };
                            let model = SceneModel {
                                name: current_obj_name.clone(),
                                mesh: Arc::new(mesh),
                                materials: used_materials.iter().copied().collect(),
                            };
                            if let Some(scene) = &mut self.current_scene {
                                scene.models.push(model);
                            }

                            used_materials.clear();
                        }

                        current_obj_name = Self::parse_next_string(&mut tokens)?;
                    }
                    Command::V => {
                        positions.push(Self::parse_vec3(&mut tokens)?);
                    }
                    Command::VT => {
                        texcoords.push(Self::parse_vec2(&mut tokens)?);
                    }
                    Command::VN => {
                        normals.push(Self::parse_vec3(&mut tokens)?);
                    }
                    Command::F => {
                        let mut face_verts = Vec::new();

                        while let Some(peek) = tokens.peek() {
                            if let Token::Command(_) = peek {
                                break;
                            }
                            face_verts.push(Self::parse_face_part(&mut tokens, &positions, &texcoords, &normals)?);
                        }

                        used_materials.insert(used_material);

                        // Triangulate (fan method)
                        if face_verts.len() >= 3 {
                            let base = vertices.len() as u32;

                            for i in 1..face_verts.len() - 1 {
                                let (p1, t1, n1) = face_verts[0];
                                let (p2, t2, n2) = face_verts[i];
                                let (p3, t3, n3) = face_verts[i + 1];

                                vertices.extend_from_slice(&[
                                    MeshVertex { position: p1, uv: t1, normal: n1, material_id: used_material },
                                    MeshVertex { position: p2, uv: t2, normal: n2, material_id: used_material },
                                    MeshVertex { position: p3, uv: t3, normal: n3, material_id: used_material },
                                ]);

                                indices.extend_from_slice(&[
                                    base + (i as u32 * 3),
                                    base + (i as u32 * 3 + 1),
                                    base + (i as u32 * 3 + 2),
                                ]);
                            }
                        }
                    }
                    Command::S => {
                        let group = Self::parse_next_string(&mut tokens)?;
                        info!("OBJModelLoader: Group {group}");
                    }
                    Command::Usemtl => {
                        let name = Self::parse_next_string(&mut tokens)?;
                        if let Some(idx) = self.material_map.get(&name) {
                            used_material = *idx as u8;
                        } else {
                            println!("mat map not found: {:?}, query: {name}", self.material_map);
                        }
                    }
                    Command::Mtllib => {
                        let filename = Self::parse_next_string(&mut tokens)?;
                        let mats = self.parse_materials(&filename, r)?;
                        if let Some(scene) = &mut self.current_scene {
                            scene.materials = mats;
                        }
                    }
                    other => {
                        return Err(ModelLoadingError::IllegalContent(format!(
                            "Expected <obj-command>, found {other:?}"
                        )));
                    }
                }
            }

            if !vertices.is_empty() {
                let mesh = Mesh {
                    vertices,
                    indices,
                };
                let model = SceneModel {
                    name: current_obj_name,
                    mesh: Arc::new(mesh),
                    materials: used_materials.into_iter().collect(),
                };
                if let Some(scene) = &mut self.current_scene {
                    scene.models.push(model);
                }
            }

            if let Some(scene) = self.current_scene.take() {
                Ok(scene)
            } else {
                Err(ModelLoadingError::IllegalContent("The OBJModelLoader was unable to build a scene from the files!".to_string()))
            }
        } else {
            Err(ModelLoadingError::IllegalContent("Non UTF8 obj file!".to_string()))
        }
    }

    fn parse_materials(
        &mut self,
        name: &str,
        r: &mut Renderer,
    ) -> Result<Vec<Material>, ModelLoadingError> {
        let mut mats = vec![];
        let mut current_mat: Option<Material> = None;
        let mut current_mat_name = String::new();

        let mat_file = self.grab_file_ext(name)?;
        let contents = mat_file
            .contents_utf8()
            .ok_or(ModelLoadingError::IllegalContent(
                "Cannot load as UTF8!".to_string(),
            ))?;
        let contents = contents.to_string(); // make borrow checker happy...

        let mut tokens = Tokenizer::new(&contents).into_peekable();

        while let Some(token) = tokens.peek() {
            tokens.next();
            let command = token.want_command()?;

            match command {
                Command::Newmtl => {
                    if let Some(mat) = current_mat.take() {
                        let id = mats.len();
                        self.material_map.insert(current_mat_name, id as u32);
                        mats.push(mat);
                    }

                    current_mat_name = Self::parse_next_string(&mut tokens)?;

                    let m = Material::default();
                    current_mat = Some(m);
                }

                other => {
                    let Some(m) = &mut current_mat else {
                        return Err(ModelLoadingError::IllegalContent(format!(
                            "Found command {other:?} outside of any material!"
                        )));
                    };

                    match other {
                        Command::Ka => m.ambient = Self::parse_vec3(&mut tokens)?,
                        Command::Kd => {
                            m.color = {
                                let v = Self::parse_vec3(&mut tokens)?;
                                Vec4::new(*v.x, *v.y, *v.z, *m.color.w)
                            }
                        }
                        Command::Ks => m.specular = Self::parse_vec3(&mut tokens)?,
                        Command::Ke => m.emission = Self::parse_vec3(&mut tokens)?,
                        Command::Ns => m.shininess = Self::parse_next_float(&mut tokens)?,
                        Command::Ni => m.ior = Self::parse_next_float(&mut tokens)?,
                        Command::D => {
                            m.color.w = Self::parse_next_float(&mut tokens)?;
                        }
                        Command::Tr => {
                            // note: Tr is the inverse of D (Tr = 1 - d)
                            let tr = Self::parse_next_float(&mut tokens)?;
                            m.color.w = (1.0 - *tr).into();
                        }
                        Command::Illum => {
                            m.illum_model = Self::parse_next_int(&mut tokens)? as u32;
                        }

                        // ---- texture maps ----
                        Command::MapKa => m.map_ka = self.parse_texture_usage(&mut tokens, r)?,
                        Command::MapKd => m.map_kd = self.parse_texture_usage(&mut tokens, r)?,
                        Command::MapKs => m.map_ks = self.parse_texture_usage(&mut tokens, r)?,
                        Command::MapBump => {
                            m.map_bump = self.parse_texture_usage(&mut tokens, r)?
                        }

                        // shouldn’t happen if Command is clean
                        other => {
                            return Err(ModelLoadingError::IllegalContent(format!(
                                "Found unexpected command {other:?} in .mtl file!"
                            )));
                        }
                    }
                }
            }
        }

        // push the last material
        if let Some(mat) = current_mat.take() {
            let id = mats.len();
            mats.push(mat);
            self.material_map.insert(current_mat_name, id as u32);
        }

        Ok(mats)
    }

    fn parse_next_some(tokens: &mut Tokens) -> Result<Token, ModelLoadingError> {
        tokens.next().ok_or(ModelLoadingError::UnexpectedEndOfFile)
    }

    fn parse_next_float(tokens: &mut Tokens) -> Result<Float, ModelLoadingError> {
        let t = Self::parse_next_some(tokens)?;
        match t {
            Token::FloatLit(f) => Ok(f.into()),
            other => Err(ModelLoadingError::IllegalContent(format!(
                "Expected Float, found {other:?}!"
            ))),
        }
    }

    fn parse_next_string(tokens: &mut Tokens) -> Result<String, ModelLoadingError> {
        let t = Self::parse_next_some(tokens)?;
        match t {
            Token::StrLit(s) => Ok(s),
            Token::FloatLit(f) => Ok(f.to_string()),
            other => Err(ModelLoadingError::IllegalContent(format!(
                "Expected String, found {other:?}!"
            ))),
        }
    }

    fn parse_next_int(tokens: &mut Tokens) -> Result<i32, ModelLoadingError> {
        let t = Self::parse_next_some(tokens)?;
        match t {
            Token::FloatLit(f) => Ok(f as i32),
            Token::StrLit(s) => s.parse::<i32>().map_err(|_| {
                ModelLoadingError::IllegalContent(format!("Expected integer, found {s:?}"))
            }),
            other => Err(ModelLoadingError::IllegalContent(format!(
                "Expected integer, found {other:?}"
            ))),
        }
    }

    fn parse_vec2(tokens: &mut Tokens) -> Result<Vec2, ModelLoadingError> {
        let x = Self::parse_next_float(tokens)?;
        let y = Self::parse_next_float(tokens)?;
        Ok(Vec2::new(*x, *y))
    }

    fn parse_vec3(tokens: &mut Tokens) -> Result<Vec3, ModelLoadingError> {
        let x = Self::parse_next_float(tokens)?;
        let y = Self::parse_next_float(tokens)?;
        let z = Self::parse_next_float(tokens)?;
        Ok(Vec3::new(*x, *y, *z))
    }

    fn parse_vec4(tokens: &mut Tokens) -> Result<Vec4, ModelLoadingError> {
        let x = Self::parse_next_float(tokens)?;
        let y = Self::parse_next_float(tokens)?;
        let z = Self::parse_next_float(tokens)?;
        let w = Self::parse_next_float(tokens)?;
        Ok(Vec4::new(*x, *y, *z, *w))
    }

    fn parse_face_part(tokens: &mut Tokens, positions: &[Vec3], texcoords: &[Vec2], normals: &[Vec3])
                       -> Result<(Vec3, Vec2, Vec3), ModelLoadingError>
    {
        let idx_str = Self::parse_next_string(tokens)?;
        let parts: Vec<_> = idx_str.split('/').collect();

        let vi = parts.get(0)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| ModelLoadingError::IllegalContent("Missing vertex index".into()))?;

        let vti = parts.get(1).and_then(|s| s.parse::<i32>().ok());
        let vni = parts.get(2).and_then(|s| s.parse::<i32>().ok());

        let pos = positions
            .get((vi - 1) as usize)
            .copied()
            .ok_or_else(|| ModelLoadingError::IllegalContent(format!("Invalid vertex index {vi}")))?;

        let tex = vti
            .and_then(|i| texcoords.get((i - 1) as usize).copied())
            .unwrap_or(Vec2::new(0.0, 0.0));

        let nor = vni
            .and_then(|i| normals.get((i - 1) as usize).copied())
            .unwrap_or(Vec3::new(0.0, 0.0, 0.0));

        Ok((pos, tex, nor))
    }

    fn parse_texture_usage(
        &mut self,
        tokens: &mut Tokens,
        r: &mut Renderer,
    ) -> Result<u32, ModelLoadingError> {
        let texture_name = Self::parse_next_string(tokens)?;
        let file = self.grab_file_ext(&texture_name)?;

        let t = r.load_texture(
            &texture_name,
            file.contents(),
            MemoryProperties::HOST_VISIBLE,
            ImageUsage::SAMPLED,
            UsageFlags::HOST_ACCESS,
        );

        let id = if let Some(scene) = &mut self.current_scene {
            let id = scene.textures.len();
            scene.textures.push(t);
            id as u32
        } else {
            0
        };

        Ok(id)
    }
}
