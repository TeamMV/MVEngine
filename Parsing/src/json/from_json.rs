use crate::json::types::JsonElement;
use std::array;

#[derive(Clone, Debug)]
pub enum FromJsonError {
    IllegalConversion,
    NoSuchField(String),
}

//error illegal conversion
fn eic<O>(o: Option<O>) -> Result<O, FromJsonError> {
    o.ok_or(FromJsonError::IllegalConversion)
}

pub trait FromJsonTrait {
    fn illegal_conversion<O>(o: Option<O>) -> Result<O, FromJsonError> {
        eic(o)
    }

    fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
    where
        Self: Sized;
}

impl FromJsonTrait for String {
    fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
    where
        Self: Sized,
    {
        eic(json.as_string())
    }
}

macro_rules! from_json_int {
    ($($t:ty,)*) => {
        $(
        impl FromJsonTrait for $t {
            fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
            where
                Self: Sized,
            {
                eic(json.as_int().map(|x| x as $t))
            }
        }
        )*
    };
}

from_json_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128,);

macro_rules! from_json_float {
    ($($t:ty,)*) => {
        $(
        impl FromJsonTrait for $t {
            fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
            where
                Self: Sized,
            {
                eic(json.as_float().map(|x| x as $t))
            }
        }
        )*
    };
}

from_json_float!(f32, f64,);

impl FromJsonTrait for bool {
    fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
    where
        Self: Sized,
    {
        eic(json.as_bool())
    }
}

impl FromJsonTrait for char {
    fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
    where
        Self: Sized,
    {
        eic(String::from_json(json)?.chars().next())
    }
}

impl<T: FromJsonTrait, const N: usize> FromJsonTrait for [T; N] {
    fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
    where
        Self: Sized,
    {
        let arr = eic(json.as_array())?;
        Ok(array::try_from_fn(|i| T::from_json(&arr[i]))?)
    }
}

impl<T: FromJsonTrait> FromJsonTrait for Vec<T> {
    fn from_json(json: &JsonElement) -> Result<Self, FromJsonError>
    where
        Self: Sized,
    {
        let arr = eic(json.as_array())?;
        let mut vec = Vec::with_capacity(arr.len());
        for element in arr {
            vec.push(T::from_json(element)?);
        }
        Ok(vec)
    }
}

macro_rules! impl_fromjson_tuple {
    ($($name:ident $idx:tt),+ $(,)?) => {
        impl<$($name: FromJsonTrait),+> FromJsonTrait for ($($name,)+) {
            fn from_json(elem: &JsonElement) -> Result<Self, FromJsonError> {
                let arr = eic(elem.as_array())?;
                Ok((
                    $(
                        $name::from_json(eic(arr.get($idx))?)?,
                    )+
                ))
            }
        }
    };
}

impl_fromjson_tuple!(A 0);
impl_fromjson_tuple!(A 0, B 1);
impl_fromjson_tuple!(A 0, B 1, C 2);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13);
impl_fromjson_tuple!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14);
