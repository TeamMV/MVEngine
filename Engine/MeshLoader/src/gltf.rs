use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use bytebuffer::ByteBuffer;
use mvutils::save::Loader;
use mvutils::unsafe_utils::UnsafeInto;
use mvcore::math::vec::{Vec2, Vec3, Vec4};
use crate::Scene;

pub struct GLTF<'a> {
    scenes: Vec<Scene<'a>>,
    active_scene: u32
}

pub fn read_gltf(path: String) -> Result<GLTF<'_>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let meta = file.metadata().expect("Cannot read file!");
    let mut bytes = vec![0u8; meta.len() as usize];
    file.read_to_end(&mut bytes).expect("Cannot read file!");

    let mut chunks = read_gltf_raw(bytes)?;
    let json = chunks.pop().map(|c| match c {ChunkContent::Json(v) => v}).ok_or("Unable to retrieve JSON chunk".to_string())?;
    let scenes_json = json.get("scenes").expect("No scenes present").as_array().unwrap();
    let active_scene = json.get("scenes").expect("No scenes present").as_u64().unwrap() as u32;

    let mut scenes = vec![];
    for scene_json in scenes_json {
        scenes.push(read_scene(&chunks, scene_json, &json, path.as_str())?);
    }

    Ok(GLTF {
        scenes,
        active_scene,
    })
}

fn read_buffers(chunks: &[Chunk], buffers_json: &Vec<serde_json::Value>, file: &str) -> Result<Vec<Buffer>, String> {
    let mut vec = vec![];

    for (idx, buffer_json) in buffers_json.iter().enumerate() {
        let length = buffer_json.get("byteLength").expect("no byteLength on buffer").as_u64().unwrap();
        let content = if let Some(uri_json) = buffer_json.get("uri") {
            let uri = uri_json.as_str().unwrap();
            let mut path = file.to_string();
            path.push_str(uri);
            let mut file = File::open(path).map_err(|e| e.to_string())?;
            let meta = file.metadata().expect("Cannot read file!");
            let mut bytes = vec![0u8; meta.len() as usize];
            file.read_to_end(&mut bytes).expect("Cannot read file!");
            bytes.as_slice()
        } else {
            match &chunks[idx + 1].content {
                ChunkContent::Json(_) => unreachable!(),
                ChunkContent::Bin(bin) => bin.as_slice()
            }
        };
        vec.push(Buffer {
            length,
            content,
        });
    }

    Ok(vec)
}

fn read_buffer_views(buffers: &[Buffer], buffer_views_json: &Vec<serde_json::Value>) -> Result<Vec<BufferView>, String> {
    let mut vec = vec![];

    for buffer_view_json in buffer_views_json {
        let buffer_idx = buffer_view_json.get("buffer").expect("No buffer present in bufferView").as_u64().unwrap() as usize;
        let byte_offset = buffer_view_json.get("byteOffset").map(|v| v.as_u64().unwrap()).unwrap_or(0);
        let byte_length = buffer_view_json.get("byteLength").expect("No byte length present for bufferView").as_u64().unwrap();
        let byte_stride = buffer_view_json.get("byteStride").map(|v| v.as_u64().unwrap()).unwrap_or(1);
        let target = buffer_view_json.get("target").expect("No target present for bufferView").as_u64().unwrap();

        vec.push(BufferView {
            buffer: &buffers[buffer_idx],
            offset: byte_offset,
            length: byte_length,
            stride: byte_stride,
            target,
        })
    }

    Ok(vec)
}

fn read_accessors(buffer_views: &[BufferView], accessors_json: &Vec<serde_json::Value>) -> Result<Vec<Accessor>, String> {
    let mut vec = vec![];

    for accessor_json in accessors_json {
        let buffer_view_idx = accessor_json.get("bufferView").expect("No buffer view present for accessor").as_u64().unwrap() as usize;
        let byte_offset = accessor_json.get("byteOffset").map(|v| v.as_u64().unwrap()).unwrap_or(0);
        let ty: DataType = accessor_json.get("type").expect("No type present for accessor").as_str().unwrap().into();
        let component_type = accessor_json.get("componentType").expect("No component type present for accessor").as_u64().unwrap();
        let count = accessor_json.get("count").expect("No count present for accessor").as_u64().unwrap();
        let min: MinMax = accessor_json.get("min").expect("No min present for accessor").as_array().unwrap().into();
        let max: MinMax = accessor_json.get("max").expect("No max present for accessor").as_array().unwrap().into();

        vec.push(Accessor {
            view: &buffer_views[buffer_view_idx],
            offset: byte_offset,
            ty,
            component_type,
            count,
            min,
            max,
        });
    }

    Ok(vec)
}

fn read_scene(chunks: &[Chunk], scene_json: &serde_json::Value, json: &serde_json::Value, file: &str) -> Result<Scene<'_>, String> {
    let meshes_json = json.get("meshes").expect("No meshes present").as_array().unwrap();
    let nodes_json = json.get("nodes").expect("No nodes present").as_array().unwrap();

    let accessors_json = json.get("accessors").expect("No accessors present").as_array().unwrap();
    let buffer_views_json = json.get("bufferViews").expect("No bufferViews present").as_array().unwrap();
    let buffers_json = json.get("buffer").expect("No buffers present").as_array().unwrap();

    let buffers = read_buffers(chunks, buffers_json, file)?;
    let buffer_views = read_buffer_views(&buffers, buffer_views_json)?;
    let accessors = read_accessors(&buffer_views, accessors_json)?;

    let name = scene_json.get("name").map(|v| v.as_str().unwrap().to_string());
    let scene_nodes = scene_json.get("nodes").expect("No nodes present in the scene").as_array().unwrap();

    for scene_node_json in scene_nodes {
        if let Some(mesh_json) =  nodes_json[scene_node_json.as_u64().unwrap() as usize].get("mesh") {
            let mesh_idx = mesh_json.as_u64().unwrap();
            let mesh_json_indexed = &meshes_json[mesh_idx as usize];
            let primitives_json = mesh_json_indexed.get("primitives").expect("mesh has no primitives").as_array().unwrap();
            for primitive_json in primitives_json {
                let prim = read_primitive(chunks, primitive_json, &accessors)?;
            }
        }
    }

    Err()
}

fn read_primitive(chunks: &[Chunk], primitive_json: &serde_json::Value, accessors: &[Accessor]) -> Result<Primitive, String> {
    let mode = primitive_json.get("mode").expect("no mode present on primitive").as_u64().unwrap();
    let material_idx = primitive_json.get("material").map(|v| v.as_u64().unwrap());
    let attributes_json = primitive_json.get("attributes").expect("No attributes present on primitive");
    let indices_idx = primitive_json.get("indices").expect("No indices present on primitive").as_u64().unwrap();
    //TODO: get data from accessor via buffer view and buffer same for position and normal data as well
}

struct Accessor<'a> {
    view: &'a BufferView<'a>,
    offset: u64,
    ty: DataType,
    component_type: u64,
    count: u64,
    min: MinMax,
    max: MinMax,
}

enum DataType {
    Float,
    Float2,
    Float3,
    Float4
}

impl From<&str> for DataType {
    fn from(value: &str) -> Self {
        match value {
            "VEC2" => DataType::Float2,
            "VEC3" => DataType::Float3,
            "VEC4" => DataType::Float4,
            _ => DataType::Float
        }
    }
}

enum MinMax {
    Float(f64),
    Float2(Vec2),
    Float3(Vec3),
    Float4(Vec4),
}

impl MinMax {
    pub fn as_float(&self) -> f64 {
        match self {
            MinMax::Float(f) => *f,
            _ => unreachable!()
        }
    }

    pub fn as_float2(&self) -> Vec2 {
        match self {
            MinMax::Float2(f) => *f,
            _ => unreachable!()
        }
    }

    pub fn as_float3(&self) -> Vec3 {
        match self {
            MinMax::Float3(f) => *f,
            _ => unreachable!()
        }
    }

    pub fn as_float4(&self) -> Vec4 {
        match self {
            MinMax::Float4(f) => *f,
            _ => unreachable!()
        }
    }
}

impl From<&Vec<serde_json::Value>> for MinMax {
    fn from(value: &Vec<serde_json::Value>) -> Self {
        let value = value.iter().map(|v| v.as_f64().unwrap() as f32).collect::<Vec<_>>();

        if value.len() == 4 {
            return Self::Float4(Vec4::new(value[0], value[1], value[2], value[3]));
        }
        if value.len() == 3 {
            return Self::Float3(Vec3::new(value[0], value[1], value[2]));
        }
        if value.len() == 2 {
            return Self::Float2(Vec2::new(value[0], value[1]));
        }
        Self::Float(value[0] as f64)
    }
}

struct BufferView<'a> {
    buffer: &'a Buffer<'a>,
    offset: u64,
    length: u64,
    stride: u64,
    target: u64
}

struct Buffer<'a> {
    length: u64,
    content: &'a [u8]
}

struct Primitive {
    mode: u64,
    indices: Vec<u16>,
    vertices: Vec<Vec3>,
    normals: Option<Vec<Vec3>>,
    uv: Option<Vec<Vec2>>,
    material_index: usize
}

fn read_gltf_raw(bytes: Vec<u8>) -> Result<Vec<Chunk>, String> {
    let mut bytes_read = 0u32;
    let mut buffer = ByteBuffer::from_vec(bytes);
    let magic = buffer.pop_u32().ok_or("Could not read file".to_string())?;
    if magic != 0x46546C67 {
        return Err("This is not a GLTF file!".to_string());
    }
    let version = buffer.pop_u32().ok_or("Could not read file".to_string())?;
    let length = buffer.pop_u32().ok_or("Could not read file".to_string())?;

    bytes_read += 3 * 4;

    let first_chunk = read_chunk(&mut buffer)?;
    if !first_chunk.am_i_json() {
        return Err("The first chunk has to be json content!".to_string());
    }
    bytes_read += first_chunk.length;
    if bytes_read >= length {
        return Ok(vec![first_chunk]);
    }

    let mut vec = vec![first_chunk];

    loop {
        let chunk = read_chunk(&mut buffer)?;
        vec.push(chunk);
        bytes_read += first_chunk.length;
        if bytes_read >= length {
            return Ok(vec);
        }
    }
}

fn read_chunk(buffer: &mut ByteBuffer) -> Result<Chunk, String> {
    let length = buffer.pop_u32().ok_or("Could not read chunk")?;
    let chunk_type = buffer.pop_u32().ok_or("Could not read chunk")?;
    let binary = buffer.pop_bytes(length as usize).ok_or("Could not read chunk data")?;
    if Chunk::is_json(chunk_type) {
        let json: serde_json::Value = serde_json::from_slice(binary.as_slice()).map_err(|e| e.to_string())?;

        Ok(Chunk {
            length,
            chunk_type,
            content: ChunkContent::Json(json),
        })
    } else {
        Ok(Chunk {
            length,
            chunk_type,
            content: ChunkContent::Bin(binary),
        })
    }
}

struct Chunk {
    length: u32,
    chunk_type: u32,
    content: ChunkContent
}

impl Chunk {
    pub fn am_i_json(&self) -> bool {
        self.chunk_type == 0x4E4F534A
    }

    pub fn am_i_bin(&self) -> bool {
        self.chunk_type == 0x004E4942
    }

    pub fn is_json(t: u32) -> bool {
        t == 0x4E4F534A
    }

    pub fn is_bin(t: u32) -> bool {
        t == 0x004E4942
    }
}

enum ChunkContent {
    Json(serde_json::Value),
    Bin(Vec<u8>),
}