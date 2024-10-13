use mvcore::math::mat::Mat4;
use mvcore::math::vec::{Vec3, Vec4};
use crate::camera::Camera;

pub mod gltf;
pub mod camera;

pub struct Material {

}

pub struct Scene<'a> {
    nodes: Vec<Node<'a>>,
    name: Option<String>
}

pub struct Node<'a> {
    children: Vec<&'a Node<'a>>,
    inner: NodeInner,
}

pub enum NodeTransform {
    Matrix(Mat4),
    Individual {
        translate: Vec4,
        rotate: Vec4,
        scale: Vec3
    }
}

pub enum NodeInner {
    Camera(Camera),
    Mesh(Mesh)
}

pub struct Mesh {}