use glam::{EulerRot, Mat4, Quat, Vec2, Vec3, Vec3A};

#[derive(Clone)]
pub struct OrthographicCamera {
    pub position: Vec2,
    pub rotation: f32,
    pub zoom: f32,
    pub near: f32,
    pub far: f32,

    projection: Mat4,
    view: Mat4,
}

impl OrthographicCamera {
    pub fn new(width: u32, height: u32) -> Self {
        OrthographicCamera {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            zoom: 1.0,
            projection: Mat4::default(),
            view: Mat4::default(),
            near: 0.01,
            far: 100.0,
        }
        .setup(width, height)
    }

    pub fn get_view(&self) -> Mat4 {
        self.view
    }

    pub fn get_projection(&self) -> Mat4 {
        self.projection
    }

    pub fn update_view(&mut self) {
        self.view = Mat4::from_scale_rotation_translation(
            Vec3::splat(self.zoom),
            Quat::from_rotation_z(self.rotation),
            (self.position, 1.0).into(),
        );

        *self.view.col_mut(1) *= -1.0f32; // this should invert the up direction?
    }

    pub fn update_projection(&mut self, width: u32, height: u32) {
        self.projection =
            Mat4::orthographic_lh(0.0, width as f32, 0.0, height as f32, self.near, self.far);
    }

    fn setup(mut self, width: u32, height: u32) -> Self {
        self.update_view();
        self.update_projection(width, height);
        self
    }
}

pub struct PerspectiveCamera {
    pub fov: f32,
    pub position: Vec3A,
    pub rotation: Vec3A,
    pub zoom: f32,
    pub near: f32,
    pub far: f32,

    projection: Mat4,
    view: Mat4,
}

impl PerspectiveCamera {
    pub fn new(width: u32, height: u32) -> Self {
        PerspectiveCamera {
            fov: 80.0,
            position: Vec3A::default(),
            rotation: Vec3A::default(),
            zoom: 1.0,
            near: 0.1,
            far: 1000.0,
            projection: Mat4::default(),
            view: Mat4::default(),
        }
        .setup(width, height)
    }

    pub fn update_view(&mut self) {
        self.view = Mat4::from_scale_rotation_translation(
            Vec3::splat(self.zoom),
            Quat::from_euler(
                EulerRot::XYZ,
                self.rotation.x,
                self.rotation.y,
                self.rotation.z,
            ),
            self.position.into(),
        );
    }

    pub fn update_projection(&mut self, width: u32, height: u32) {
        self.projection = Mat4::perspective_lh(
            self.fov.to_radians(),
            width as f32 / height as f32,
            self.near,
            self.far,
        );
    }

    pub fn get_view(&self) -> Mat4 {
        self.view
    }

    pub fn get_projection(&self) -> Mat4 {
        self.projection
    }

    fn setup(mut self, width: u32, height: u32) -> Self {
        self.update_view();
        self.update_projection(width, height);
        self
    }
}
