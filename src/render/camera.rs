use std::convert::identity;
use std::ops::Mul;
use cgmath::{Matrix2, Matrix4, Ortho, ortho, SquareMatrix, Transform, Transform2, Vector2, Vector3, Zero};
use cgmath::num_traits::real::Real;

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

    pub fn get_view_mat(&self) -> Matrix4<f32> {
        cam_fn_call!(get_view_mat, self);
    }

    pub fn get_projection_mat(&self) -> &Matrix4<f32> {
        cam_fn_call!(get_projection_mat, self);
    }

    pub fn update_projection_mat(&self, width: u16, height: u16) {
        cam_fn_call!(update_projection_mat, self, width, height);
    }
}

#[derive(Clone)]
pub struct Camera2D {
    position: Vector2<f32>,
    rotation: f32,
    projection: Matrix4<f32>,
    z_near: f32,
    z_far: f32,
}

impl Default for Camera2D {
    fn default() -> Self { Camera2D::new(0.0, 0.0) }
}

impl Camera2D {
    pub(crate) fn new(x: f32, y: f32) -> Self {
        Camera2D {
            position: Vector2::new(x, y),
            rotation: 0.0,
            projection: Matrix4::identity(),
            z_near: 0.01,
            z_far: 1000.0,
        }
    }

    pub(crate) fn get_view_mat(&self) -> Matrix4<f32> {
        let mut matrix: Matrix4<f32> = Matrix4::identity();
        matrix.translate(self.position.x, self.position.y, 0.0);
        matrix.rotate(0.0, 0.0, self.rotation);
        matrix
    }

    pub(crate) fn get_projection_mat(&self) -> &Matrix4<f32> {
        &self.projection
    }

    pub(crate) fn update_projection_mat(&mut self, width: u16, height: u16) {
        self.projection = cgmath::ortho(0.0, width as f32, 0.0, height as f32, self.z_near, self.z_far);
    }
}

pub trait Trans<T> {
    fn rotate(&mut self, x: T, y: T, z: T);
    fn translate(&mut self, x: T, y: T, z: T);
    fn scale(&mut self, x: T, y: T, z: T);
}

impl Trans<f32> for Matrix2<f32> {
    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        let mut rot_z = Matrix2::identity();
        //[cos, -sin]
        //[sin, cos ]
        rot_z.x.x = z.to_radians().cos();
        rot_z.x.y = z.to_radians().sin() * -1.0;
        rot_z.y.x = z.to_radians().sin();
        rot_z.y.y = z.to_radians().cos();
        self.mul(rot_z);
    }

    fn translate(&mut self, x: f32, y: f32, z: f32) {
        let mut trns = Matrix2::identity();
        //[0, tx]
        //[1, ty]
        trns.x.x = 0.0;
        trns.x.y = x;
        trns.y.x = 1.0;
        trns.y.y = y;
        self.mul(trns);
    }

    fn scale(&mut self, x: f32, y: f32, z: f32) {
        let mut scl = Matrix2::identity();
        //[sx, 0]
        //[0, sy]
        scl.x.x = x;
        scl.y.x = y;
        self.mul(scl);
    }
}

impl Trans<f32> for Matrix4<f32> {
    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        let x_sin = x.to_radians().sin();
        let x_cos = x.to_radians().cos();
        let y_sin = y.to_radians().sin();
        let y_cos = y.to_radians().cos();
        let z_sin = z.to_radians().sin();
        let z_cos = z.to_radians().cos();

        let mut rot_x: Matrix4<f32> = Matrix4::identity();
        //[0, 0, 0, 0]
        //[0, cos, -sin, 0]
        //[0, sin, cos, 0]
        //[0, 0, 0, 0]
        rot_x.y.y = x_cos;
        rot_x.y.z = -x_sin;
        rot_x.z.y = x_sin;
        rot_x.z.z = x_cos;

        let mut rot_y: Matrix4<f32> = Matrix4::identity();
        //[cos, 0, sin, 0]
        //[0, 0, 0, 0]
        //[-sin, 0, cos, 0]
        //[0, 0, 0, 0]
        rot_y.x.x = y_cos;
        rot_y.x.z = y_sin;
        rot_y.z.x = -y_sin;
        rot_y.z.z = y_cos;

        let mut rot_z: Matrix4<f32> = Matrix4::identity();
        //[cos, -sin, 0, 0]
        //[sin, cos, 0, 0]
        //[0, 0, 0, 0]
        //[0, 0, 0, 0]

        rot_z.x.x = z_cos;
        rot_z.x.y = -z_sin;
        rot_z.y.x = z_sin;
        rot_z.y.y = z_cos;
        self.mul(rot_x.mul(rot_y).mul(rot_z));
    }

    fn translate(&mut self, x: f32, y: f32, z: f32) {
        let mut trns = Matrix4::identity();
        //[1, 0, 0, 0]
        //[0, 1, 0, 0]
        //[0, 0, 1, 0]
        //[tx, ty, tz, 1]
        trns.w.x = x;
        trns.w.y = y;
        trns.w.z = z;
        self.mul(trns);
    }

    fn scale(&mut self, x: f32, y: f32, z: f32) {
        let mut scl = Matrix4::identity();
        //[sx, 0, 0, 0]
        //[0, sy, 0, 0]
        //[0, 0, sz, 0]
        //[0, 0, 0, 1]
        scl.x.x = x;
        scl.y.y = y;
        scl.z.z = z;
        self.mul(scl);
    }
}