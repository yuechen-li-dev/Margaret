use crate::color::ColorRgb;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialDescription {
    pub id: MaterialId,
    pub name: String,
    pub kind: MaterialKind,
}

impl MaterialDescription {
    pub fn new(id: MaterialId, name: impl Into<String>, kind: MaterialKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialKind {
    Diffuse { albedo: ColorRgb },
}
