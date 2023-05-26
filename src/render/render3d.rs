use std::sync::Arc;
use glam::Mat4;
use crate::render::common::Texture;
use crate::render::consts::TEXTURE_LIMIT;

pub(crate) trait RenderPass3D {
    fn render_batch(&self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT], transforms: &[Mat4]);

    fn render_model(&self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT], canvas: &[f32; 6], transform: Mat4) {
        self.render_model_instanced(indices, vertices, textures, canvas, &[transform], 1);
    }

    fn render_model_instanced(&self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT],  canvas: &[f32; 6], transforms: &[Mat4], hum_instances: u32);

}

pub(crate) struct ForwardPass {

}

impl RenderPass3D for ForwardPass {
    fn render_batch(&self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT], transforms: &[Mat4]) {

    }

    fn render_model_instanced(&self, indices: &[u32], vertices: &[f32], textures: &[Option<Arc<Texture>>; TEXTURE_LIMIT], canvas: &[f32; 6], transforms: &[Mat4], hum_instances: u32) {

    }
}