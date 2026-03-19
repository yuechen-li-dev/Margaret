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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMetadata {
    pub backend_name: String,
    pub scene_name: String,
    pub image_size: ImageSize,
    pub pixel_format: OutputPixelFormat,
    pub sample_count: u32,
    pub object_count: usize,
    pub light_count: usize,
}
