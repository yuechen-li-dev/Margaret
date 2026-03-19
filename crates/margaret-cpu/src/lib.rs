use margaret_core::image::{ImageSize, OutputPixelFormat, RenderMetadata};
use margaret_core::scene::SceneDescription;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CpuRendererBackend;

impl CpuRendererBackend {
    pub const fn new() -> Self {
        Self
    }

    pub const fn backend_name(&self) -> &'static str {
        "cpu"
    }

    pub fn describe_render(
        &self,
        scene: &SceneDescription,
        image_size: ImageSize,
    ) -> RenderMetadata {
        RenderMetadata {
            backend_name: self.backend_name().to_string(),
            scene_name: scene.name.clone(),
            image_size,
            pixel_format: OutputPixelFormat::Rgba8Unorm,
            sample_count: 1,
            object_count: scene.objects.len(),
            light_count: scene.lights.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CpuRendererBackend;
    use margaret_testutil::{sample_image_size, sample_scene};

    #[test]
    fn describe_render_reports_basic_scene_counts() {
        let backend = CpuRendererBackend::new();
        let metadata = backend.describe_render(&sample_scene(), sample_image_size());

        assert_eq!(metadata.backend_name, "cpu");
        assert_eq!(metadata.object_count, 1);
        assert_eq!(metadata.light_count, 1);
    }
}
