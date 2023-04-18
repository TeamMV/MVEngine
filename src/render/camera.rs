use std::convert::identity;
use std::ops::Mul;
use glam::{Mat2, Mat3, Mat4, Quat, Vec2, Vec3};

#[derive(Clone)]
pub enum Camera{
    Is2d(Camera2D),
    Is3d(Camera2D)
}

macro_rules! cam_fn_call {
    ($name:ident, $sel:ident) => {
        return match $sel {
            Camera::Is2d(_2d) => {
                _2d.$name()
            }
            Camera::Is3d(_3d) => {
                _3d.$name()
            }
        }
    };
    ($name:ident, $sel:ident, $($params:ident),*) => {
        return match $sel {
            Camera::Is2d(_2d) => {
                _2d.$name($($params, )*)
            }
            Camera::Is3d(_3d) => {
                _3d.$name($($params, )*)
            }
        }
    };
}

impl Camera {
    pub fn new_2d() -> Camera {
        Camera::Is2d(Camera2D::default())
    }

    pub fn get_view_mat(&self) -> Mat4 {
        cam_fn_call!(get_view_mat, self);
    }

    pub fn get_projection_mat(&self) -> &Mat4 {
        cam_fn_call!(get_projection_mat, self);
    }

    pub fn update_projection_mat(&mut self, width: i32, height: i32) {
        cam_fn_call!(update_projection_mat, self, width, height);
    }
}

#[derive(Clone)]
pub struct Camera2D {
    position: Vec2,
    rotation: f32,
    zoom: f32,
    projection: Mat4,
    z_near: f32,
    z_far: f32,
}

impl Default for Camera2D {
    fn default() -> Self { Camera2D::new(0.0, 0.0) }
}

impl Camera2D {
    pub(crate) fn new(x: f32, y: f32) -> Self {
        Camera2D {
            position: Vec2::new(x, y),
            rotation: 0.0,
            zoom: 1.0,
            projection: Mat4::default(),
            z_near: 0.01,
            z_far: 1000.0,
        }
    }

    pub(crate) fn get_view_mat(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::new(self.zoom, self.zoom, self.zoom),
            Quat::from_rotation_z(self.rotation),
            Vec3::from((self.position, 0.0)))
    }

    pub(crate) fn get_projection_mat(&self) -> &Mat4 {
        &self.projection
    }

    pub(crate) fn update_projection_mat(&mut self, width: i32, height: i32) {
        self.projection = Mat4::orthographic_lh(0.0, width as f32, 0.0, height as f32, self.z_near, self.z_far);
    }
}