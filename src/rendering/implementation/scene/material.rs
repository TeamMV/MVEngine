use crate::utils::hashable::{Float, Vec3, Vec4};
use mvengine_proc_macro::chash;
use std::cell::Cell;

#[chash]
#[repr(C)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Material {
    pub color: Vec4, // rgba — alpha = opacity
    pub ambient: Vec3,

    /// Specular reflectivity (Ks)
    pub specular: Vec3,

    /// Emissive color (Ke)
    pub emission: Vec3,

    /// Index of refraction (Ni)
    pub ior: Float,

    /// Shininess (Ns)
    pub shininess: Float,

    /// Illumination scene (illum)
    pub illum_model: u32,

    pub map_ka: u32,   // ambient map
    pub map_kd: u32,   // diffuse/albedo map
    pub map_ks: u32,   // specular map
    pub map_bump: u32, // normal/bump map

    /// Padding for 16-byte alignment (std140 safety)
    _pad: [u32; 2],
}

impl Default for Material {
    fn default() -> Self {
        Self {
            cached_hash: Cell::new(None),
            //cannot be asked to make the proc macro better LMAO
            inner: Material__CHASH {
                color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                ambient: Vec3::new(0.0, 0.0, 0.0),
                specular: Vec3::new(0.0, 0.0, 0.0),
                emission: Vec3::new(0.0, 0.0, 0.0),
                ior: 1.0.into(),
                shininess: 1.0.into(),
                illum_model: 1,
                map_ka: 0,
                map_kd: 0,
                map_ks: 0,
                map_bump: 0,
                _pad: [0, 0],
            },
        }
    }
}
