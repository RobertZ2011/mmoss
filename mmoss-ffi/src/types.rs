/// FFI-compatible 3D vector
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<bevy::math::Vec3> for Vec3 {
    fn from(v: bevy::math::Vec3) -> Self {
        Vec3 {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vec3> for bevy::math::Vec3 {
    fn from(v: Vec3) -> Self {
        bevy::math::Vec3::new(v.x, v.y, v.z)
    }
}

/// FFI-compatible quaternion
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<bevy::math::Quat> for Quat {
    fn from(v: bevy::math::Quat) -> Self {
        Quat {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}

impl From<Quat> for bevy::math::Quat {
    fn from(v: Quat) -> Self {
        bevy::math::Quat::from_xyzw(v.x, v.y, v.z, v.w)
    }
}
