use margaret_core::image::ImageSize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VulkanRendererBackend;

impl VulkanRendererBackend {
    pub const fn new() -> Self {
        Self
    }

    pub const fn backend_name(&self) -> &'static str {
        "vulkan"
    }

    pub fn supports_size(&self, image_size: ImageSize) -> bool {
        image_size.width > 0 && image_size.height > 0
    }
}

#[cfg(test)]
mod tests {
    use super::VulkanRendererBackend;
    use margaret_core::image::ImageSize;

    #[test]
    fn scaffold_accepts_non_zero_image_sizes() {
        let backend = VulkanRendererBackend::new();

        assert!(backend.supports_size(ImageSize::new(640, 480)));
        assert!(!backend.supports_size(ImageSize::new(0, 480)));
    }
}
