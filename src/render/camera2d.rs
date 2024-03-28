use glam::{Mat4, Quat, Vec2, Vec3};

#[derive(Clone)]
pub struct Camera2D {
    pub(crate) position: Vec2,
    pub(crate) rotation: f32,
    pub(crate) zoom: f32,
    projection: Mat4,
    view: Mat4,
    pub(crate) z_near: f32,
    pub(crate) z_far: f32,
}

impl Camera2D {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Camera2D {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            zoom: 1.0,
            projection: Mat4::default(),
            view: Mat4::default(),
            z_near: 0.01,
            z_far: 100.0,
        }
            .setup(width, height)
    }

    pub(crate) fn get_view(&self) -> Mat4 {
        self.view
    }

    pub(crate) fn get_projection(&self) -> Mat4 {
        self.projection
    }

    pub(crate) fn update_view(&mut self) {
        self.view = Mat4::from_scale_rotation_translation(
            Vec3::new(self.zoom, self.zoom, self.zoom),
            Quat::from_rotation_z(self.rotation),
            Vec3::from((self.position, 1.0)),
        );

        *self.view.col_mut(1) *= -1.0f32; // this should invert the up direction?
    }

    pub(crate) fn update_projection(&mut self, width: u32, height: u32) {
        self.projection = Mat4::orthographic_lh(
            0.0,
            width as f32,
            0.0,
            height as f32,
            self.z_near,
            self.z_far,
        );
    }

    fn setup(mut self, width: u32, height: u32) -> Self {
        self.update_view();
        self.update_projection(width, height);
        self
    }
}