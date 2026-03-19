use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use margaret_core::color::ColorRgba8;
use margaret_core::image::ImageSize;

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

    pub fn set_pixel(&mut self, x: u32, y: u32, color: ColorRgba8) {
        let index = (y * self.size.width + x) as usize;
        self.pixels[index] = color;
    }

    pub fn write_ppm(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "P3")?;
        writeln!(writer, "{} {}", self.size.width, self.size.height)?;
        writeln!(writer, "255")?;

        for pixel in &self.pixels {
            writeln!(writer, "{} {} {}", pixel.r, pixel.g, pixel.b)?;
        }

        writer.flush()
    }
}
