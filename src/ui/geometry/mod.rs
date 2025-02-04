use num_traits::Float;

#[derive(Clone, PartialEq, Debug)]
pub struct SimpleRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl SimpleRect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn inside(&self, x: i32, y: i32) -> bool {
        self.x <= x && self.x + self.width >= x && self.y <= y && self.y + self.height >= y
    }

    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    rotation: f32,
    origin: (i32, i32),

    pub bounding: SimpleRect
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32, rotation: f32, origin: (i32, i32)) -> Self {
        let mut this = Self { x, y, width, height, rotation, origin, bounding: SimpleRect::new(0, 0, 0, 0) };
        this.update();
        this
    }

    pub fn inside(&self, x: i32, y: i32) -> bool {
        let (tx, ty) = self.rot_points_r((x, y), -self.rotation);

        self.x <= tx && self.x + self.width >= tx && self.y <= ty && self.y + self.height >= ty
    }

    fn rot_points(&self, point: (i32, i32)) -> (i32, i32) {
        self.rot_points_r(point, self.rotation)
    }

    fn rot_points_r(&self, point: (i32, i32), rot: f32) -> (i32, i32) {
        let translated_x = point.0 - self.origin.0;
        let translated_y = point.1 - self.origin.1;

        let rot_cos = rot.cos();
        let rot_sin = rot.sin();

        let rotated_x = translated_x as f32 * rot_cos - translated_y as f32 * rot_sin;
        let rotated_y = translated_x as f32 * rot_sin + translated_y as f32 * rot_cos;

        (
            (rotated_x as i32 + self.origin.0),
            (rotated_y as i32 + self.origin.1),
        )
    }

    pub fn center(&self) -> (i32, i32) {
        let original_center = (
            self.x + self.width / 2,
            self.y + self.height / 2,
        );
        self.rot_points(original_center)
    }

    fn update(&mut self) {
        let tl = self.rot_points((self.x, self.y));
        let tr = self.rot_points((self.x + self.width, self.y));
        let bl = self.rot_points((self.x, self.y + self.height));
        let br = self.rot_points((self.x + self.width, self.y + self.height));

        let min_x = tl.0.min(tr.0).min(bl.0).min(br.0);
        let max_x = tl.0.max(tr.0).max(bl.0).max(br.0);
        let min_y = tl.1.min(tr.1).min(bl.1).min(br.1);
        let max_y = tl.1.max(tr.1).max(bl.1).max(br.1);

        self.bounding = SimpleRect::new(
            min_x,
            min_y,
            max_x - min_x,
            max_y - min_y
        );
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn origin(&self) -> (i32, i32) {
        self.origin
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x;
        self.update();
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
        self.update();
    }

    pub fn set_width(&mut self, width: i32) {
        self.width = width;
        self.update();
    }

    pub fn set_height(&mut self, height: i32) {
        self.height = height;
        self.update();
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
        self.update();
    }

    pub fn set_origin(&mut self, origin: (i32, i32)) {
        self.origin = origin;
        self.update();
    }

    pub fn add_x(&mut self, x: i32) {
        self.x += x;
        self.update();
    }

    pub fn add_y(&mut self, y: i32) {
        self.y += y;
        self.update();
    }

    pub fn add_width(&mut self, width: i32) {
        self.width += width;
        self.update();
    }

    pub fn add_height(&mut self, height: i32) {
        self.height += height;
        self.update();
    }

    pub fn add_rotation(&mut self, rotation: f32) {
        self.rotation += rotation;
        self.update();
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0, 0, 0, 0, 0f32, (0, 0))
    }
}