#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Debug(RenderDebugMode),
    Lit,
}

impl RenderMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Debug(mode) => mode.as_str(),
            Self::Lit => "lit",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        if name == "lit" {
            return Some(Self::Lit);
        }

        RenderDebugMode::parse(name).map(Self::Debug)
    }
}

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
pub struct RenderSettings {
    pub mode: RenderMode,
    pub depth_max_distance: f32,
}

impl RenderSettings {
    pub const fn new(mode: RenderMode, depth_max_distance: f32) -> Self {
        Self {
            mode,
            depth_max_distance,
        }
    }
}
