#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

impl ImageSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn pixel_count(self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputPixelFormat {
    Rgba8Unorm,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMetadata {
    pub backend_name: String,
    pub scene_name: String,
    pub image_size: ImageSize,
    pub pixel_format: OutputPixelFormat,
    pub debug_mode: RenderDebugMode,
    pub sample_count: u32,
    pub object_count: usize,
    pub light_count: usize,
}
