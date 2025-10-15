use bincode::{
    Decode, Encode,
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
};

/// Wrapper struct for serializing/deserializing [`bevy::math::Vec3`]
#[repr(transparent)]
pub struct Vec3(bevy::math::Vec3);

impl Encode for Vec3 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.x.encode(encoder)?;
        self.0.y.encode(encoder)?;
        self.0.z.encode(encoder)?;
        Ok(())
    }
}

impl<Context> Decode<Context> for Vec3 {
    fn decode<T: Decoder>(decoder: &mut T) -> Result<Self, DecodeError> {
        let x = f32::decode(decoder)?;
        let y = f32::decode(decoder)?;
        let z = f32::decode(decoder)?;
        Ok(Vec3(bevy::math::Vec3::new(x, y, z)))
    }
}

impl From<bevy::math::Vec3> for Vec3 {
    fn from(v: bevy::math::Vec3) -> Self {
        Vec3(v)
    }
}

impl From<Vec3> for bevy::math::Vec3 {
    fn from(v: Vec3) -> Self {
        v.0
    }
}

/// Wrapper struct for serializing/deserializing [`bevy::math::Vec2`]
#[repr(transparent)]
pub struct Vec2(bevy::math::Vec2);

impl Encode for Vec2 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.x.encode(encoder)?;
        self.0.y.encode(encoder)?;
        Ok(())
    }
}

impl<Context> Decode<Context> for Vec2 {
    fn decode<T: Decoder>(decoder: &mut T) -> Result<Self, DecodeError> {
        let x = f32::decode(decoder)?;
        let y = f32::decode(decoder)?;
        Ok(Vec2(bevy::math::Vec2::new(x, y)))
    }
}

impl From<bevy::math::Vec2> for Vec2 {
    fn from(v: bevy::math::Vec2) -> Self {
        Vec2(v)
    }
}

impl From<Vec2> for bevy::math::Vec2 {
    fn from(v: Vec2) -> Self {
        v.0
    }
}

/// Wrapper struct for serializing/deserializing [`bevy::math::Vec4`]
#[repr(transparent)]
pub struct Vec4(bevy::math::Vec4);

impl Encode for Vec4 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.x.encode(encoder)?;
        self.0.y.encode(encoder)?;
        self.0.z.encode(encoder)?;
        self.0.w.encode(encoder)?;
        Ok(())
    }
}

impl<Context> Decode<Context> for Vec4 {
    fn decode<T: Decoder>(decoder: &mut T) -> Result<Self, DecodeError> {
        let x = f32::decode(decoder)?;
        let y = f32::decode(decoder)?;
        let z = f32::decode(decoder)?;
        let w = f32::decode(decoder)?;
        Ok(Vec4(bevy::math::Vec4::new(x, y, z, w)))
    }
}

impl From<bevy::math::Vec4> for Vec4 {
    fn from(v: bevy::math::Vec4) -> Self {
        Vec4(v)
    }
}

impl From<Vec4> for bevy::math::Vec4 {
    fn from(v: Vec4) -> Self {
        v.0
    }
}

/// Wrapper struct for serializing/deserializing [`bevy::math::Quat`]
#[repr(transparent)]
pub struct Quat(bevy::math::Quat);

impl Encode for Quat {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.x.encode(encoder)?;
        self.0.y.encode(encoder)?;
        self.0.z.encode(encoder)?;
        self.0.w.encode(encoder)?;
        Ok(())
    }
}

impl<Context> Decode<Context> for Quat {
    fn decode<T: Decoder>(decoder: &mut T) -> Result<Self, DecodeError> {
        let x = f32::decode(decoder)?;
        let y = f32::decode(decoder)?;
        let z = f32::decode(decoder)?;
        let w = f32::decode(decoder)?;
        Ok(Quat(bevy::math::Quat::from_xyzw(x, y, z, w)))
    }
}

impl From<bevy::math::Quat> for Quat {
    fn from(q: bevy::math::Quat) -> Self {
        Quat(q)
    }
}

impl From<Quat> for bevy::math::Quat {
    fn from(q: Quat) -> Self {
        q.0
    }
}
