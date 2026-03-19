use crate::color::ColorRgb;
use crate::math::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub enum Light {
    Directional {
        direction: Vec3,
        intensity: ColorRgb,
    },
}
