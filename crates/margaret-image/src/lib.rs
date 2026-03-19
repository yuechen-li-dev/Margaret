use margaret_core::color::ColorRgba8;
use margaret_core::image::{ImageSize, RenderMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedImage {
    pub size: ImageSize,
    pub pixels: Vec<ColorRgba8>,
}

impl OwnedImage {
    pub fn new(size: ImageSize, fill: ColorRgba8) -> Self {
        let pixel_count = size.pixel_count() as usize;
        let pixels = vec![fill; pixel_count];
        Self { size, pixels }
    }
}

pub fn placeholder_image(metadata: &RenderMetadata) -> OwnedImage {
    let fill = if metadata.object_count == 0 {
        ColorRgba8::OPAQUE_BLACK
    } else {
        ColorRgba8::new(32, 48, 96, 255)
    };

    OwnedImage::new(metadata.image_size, fill)
}

#[cfg(test)]
mod tests {
    use super::placeholder_image;
    use margaret_core::image::{ImageSize, OutputPixelFormat, RenderMetadata};

    #[test]
    fn placeholder_image_matches_requested_size() {
        let metadata = RenderMetadata {
            backend_name: "cpu".to_string(),
            scene_name: "test".to_string(),
            image_size: ImageSize::new(4, 2),
            pixel_format: OutputPixelFormat::Rgba8Unorm,
            sample_count: 1,
            object_count: 1,
            light_count: 1,
        };

        let image = placeholder_image(&metadata);

        assert_eq!(image.pixels.len(), 8);
    }
}
