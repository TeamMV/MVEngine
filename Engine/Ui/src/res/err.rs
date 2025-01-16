pub type UiResult<T> = Result<T, UiResErr>;

pub enum ResType {
    Color,
    Shape
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
    ($res:ident.$name:ident) => {
        $res.resolve_color($name).ok_or(UiResErr::new(stringify!($name), ResType::Color))?;
    };
}

#[macro_export]
macro_rules! get_shape {
    ($res:ident.$name:ident) => {
        $res.resolve_shape($name).ok_or(UiResErr::new(stringify!($name), ResType::Shape))?;
    };
}