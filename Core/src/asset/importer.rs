use std::rc::Rc;
use gltf::{Gltf, Semantic};
use gltf::buffer::Source;
use crate::render::mesh::Mesh;
use crate::render::model::Model;
use crate::render::model::Material;
use crate::render::texture::Texture;

struct AssetImporter {

}

impl AssetImporter {
    pub(crate) fn import_model(gltf: gltf::Result<Gltf>) -> Model {
        let gltf = gltf.unwrap_or_else(|e| {
            log::error!("Failed to load model, path:, error: {e}");
            panic!();
        });

        // for scene in gltf.scenes() {
        //     Self::process_node(gltf, scene.nodes().nth(0).unwrap_or_else(|e| {
        //         log::error!("Failed to get root node? this should never happen, error: {e}");
        //         panic!();
        //     }));
        // }

        todo!();
    }

    fn process_node(gltf: Gltf, node: gltf::Node, parent_transform: glam::Mat4) -> (Vec<Mesh>, Vec<String>, Vec<Texture>, Vec<Material>) {

        let transform = node.transform(); // TODO: as glam::mat4
        for mesh in node.mesh() {
            for primitive in mesh.primitives() {
                let pos = primitive.get(&Semantic::Positions).unwrap().view().unwrap();
                let data = match pos.buffer().source() {
                    Source::Bin => gltf.blob.as_ref().unwrap()[pos.offset()..pos.offset() + pos.length()],
                    Source::Uri(file) => std::fs::read(file).unwrap()[pos.offset()..pos.offset() + pos.length()],
                };
            }
        }

        todo!();
    }

    pub(crate) fn import_texture() -> Texture {

        todo!()
    }
}