#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDebugMode {
    GeometricNormals,
    FlatAlbedo,
    Depth,
}

impl RenderDebugMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GeometricNormals => "normals",
            Self::FlatAlbedo => "albedo",
            Self::Depth => "depth",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "normals" => Some(Self::GeometricNormals),
            "albedo" => Some(Self::FlatAlbedo),
            "depth" => Some(Self::Depth),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderDebugSettings {
    pub mode: RenderDebugMode,
    pub depth_max_distance: f32,
}

impl RenderDebugSettings {
    pub const fn new(mode: RenderDebugMode, depth_max_distance: f32) -> Self {
        Self {
            mode,
            depth_max_distance,
        }
    }
}
