pub type UiResult<T> = Result<T, UiResErr>;

pub enum ResType {
    Color,
    Shape,
    Adaptive,
    Texture
}

pub struct UiResErr {
    pub resource_name: String,
    pub res_type: ResType,
}

impl UiResErr {
    pub fn new(name: &str, ty: ResType) -> Self {
        Self {
            resource_name: name.to_string(),
            res_type: ty,
        }
    }
}

#[macro_export]
macro_rules! get_color {
    ($res:ident, $name:ident) => {
        $res.resolve_color($name).ok_or(UiResErr::new(stringify!($name), ResType::Color))
    };
}

#[macro_export]
macro_rules! get_shape {
    ($res:ident, $name:ident) => {
        $res.resolve_shape($name).ok_or(UiResErr::new(stringify!($name), ResType::Shape))
    };
}

#[macro_export]
macro_rules! get_adaptive {
    ($res:ident, $name:ident) => {
        $res.resolve_adaptive($name).ok_or(UiResErr::new(stringify!($name), ResType::Adaptive))
    };
}

#[macro_export]
macro_rules! get_texture {
    ($res:ident, $name:ident) => {
        $res.resolve_texture($name).ok_or(UiResErr::new(stringify!($name), ResType::Texture))
    };
}