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

    pub fn diffuse_albedo(&self) -> ColorRgb {
        match self.kind {
            MaterialKind::Diffuse { albedo, .. } => albedo,
            MaterialKind::SpecularReflector { reflectance } => reflectance,
            MaterialKind::Dielectric { .. } => ColorRgb::WHITE,
        }
    }

    pub fn emissive_radiance(&self) -> ColorRgb {
        match self.kind {
            MaterialKind::Diffuse { emission, .. } => emission,
            MaterialKind::SpecularReflector { .. } | MaterialKind::Dielectric { .. } => {
                ColorRgb::BLACK
            }
        }
    }

    pub fn is_emissive(&self) -> bool {
        self.emissive_radiance() != ColorRgb::BLACK
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialKind {
    Diffuse {
        albedo: ColorRgb,
        emission: ColorRgb,
    },
    SpecularReflector {
        reflectance: ColorRgb,
    },
    Dielectric {
        refractive_index: f32,
    },
}
