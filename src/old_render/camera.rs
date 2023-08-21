use glam::{EulerRot, Mat4, Quat, Vec2, Vec3};

#[derive(Clone)]
pub struct Camera2D {
    pub position: Vec2,
    pub rotation: f32,
    pub zoom: f32,
    projection: Mat4,
    pub z_near: f32,
    pub z_far: f32,
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

    pub(crate) fn get_projection_mat(&self) -> Mat4 {
        self.projection
    }

    pub(crate) fn update_projection_mat(&mut self, width: i32, height: i32) {
        self.projection = Mat4::orthographic_lh(0.0, width as f32, 0.0, height as f32, self.z_near, self.z_far);
    }
}

#[derive(Clone)]
pub struct Camera3D {
    pub position: Vec3,
    pub rotation: Vec3,
    pub zoom: f32,
    projection: Mat4,
    pub z_near: f32,
    pub z_far: f32,
    pub fov: f32,
}

impl Default for Camera3D {
    fn default() -> Self {
        Camera3D::new(Vec3::new(0.0, 0.0, 20.0), Vec3::default())
    }
}

impl Camera3D {
    pub fn new(pos: Vec3, rot: Vec3) -> Self {
        Camera3D {
            position: pos,
            rotation: rot,
            zoom: 1.0,
            projection: Mat4::default(),
            z_near: 0.1,
            z_far: 2000.0,
            fov: std::f32::consts::FRAC_PI_4,
        }
    }

    pub(crate) fn get_view_mat(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::new(self.zoom, self.zoom, self.zoom),
            Quat::from_euler(EulerRot::XYZ, self.rotation.x, self.rotation.y, self.rotation.z),
            self.position)
    }

    pub(crate) fn get_projection_mat(&self) -> Mat4 {
        self.projection
    }

    pub(crate) fn update_projection_mat(&mut self, width: i32, height: i32) {
        println!("{}", self.z_near);
        self.projection = Mat4::perspective_lh(self.fov, width as f32 / height as f32, self.z_near, self.z_far);
    }
}