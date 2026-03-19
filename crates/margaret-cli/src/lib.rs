use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use margaret_core::camera::Camera;
use margaret_core::color::ColorRgb;
use margaret_core::image::ImageSize;
use margaret_core::material::{MaterialDescription, MaterialId, MaterialKind};
use margaret_core::math::{Point3, Vec3};
use margaret_core::render::{RenderMode, RenderSettings};
use margaret_core::scene::{Geometry, SceneDescription, SceneObject, Triangle};
use margaret_cpu::CpuRendererBackend;

const DEFAULT_DEPTH_MAX_DISTANCE: f32 = 6.0;
const DEFAULT_OUTPUT_PATH: &str = "margaret-m2a-normals.ppm";

pub fn run() -> std::io::Result<()> {
    run_from_args(env::args_os())
}

fn run_from_args<I>(args: I) -> std::io::Result<()>
where
    I: IntoIterator<Item = OsString>,
{
    let config = CliConfig::parse(args)?;
    if config.show_help {
        return Ok(());
    }

    let scene = hardcoded_scene();
    let backend = CpuRendererBackend::new();
    let metadata = backend.describe_render(&scene, config.image_size);
    let image = backend.render(&scene, config.image_size, config.render_settings);

    if let Some(parent) = config.output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    image.write_ppm(&config.output_path)?;

    println!("Margaret M2a CPU direct-light render");
    println!("scene: {}", metadata.scene_name);
    println!("backend: {}", metadata.backend_name);
    println!("mode: {}", config.render_settings.mode.as_str());
    println!(
        "image: {}x{} {:?}",
        metadata.image_size.width, metadata.image_size.height, metadata.pixel_format
    );
    println!("samples: {}", metadata.sample_count);
    println!("objects: {}", metadata.object_count);
    println!("lights: {}", metadata.light_count);
    println!("output: {}", config.output_path.display());

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct CliConfig {
    pub image_size: ImageSize,
    pub render_settings: RenderSettings,
    pub output_path: PathBuf,
    pub show_help: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            image_size: ImageSize::new(320, 240),
            render_settings: RenderSettings::new(
                RenderMode::Debug(margaret_core::render::RenderDebugMode::GeometricNormals),
                DEFAULT_DEPTH_MAX_DISTANCE,
            ),
            output_path: PathBuf::from(DEFAULT_OUTPUT_PATH),
            show_help: false,
        }
    }
}

impl CliConfig {
    fn parse<I>(args: I) -> std::io::Result<Self>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut config = Self::default();
        let mut arguments = args.into_iter();
        let _program_name = arguments.next();

        while let Some(argument) = arguments.next() {
            let text = argument.to_string_lossy();

            match text.as_ref() {
                "--mode" => {
                    let value = next_argument(&mut arguments, "--mode")?;
                    let value_text = value.to_string_lossy();
                    let render_mode = RenderMode::parse(value_text.as_ref()).ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "unsupported render mode '{value_text}', expected normals, albedo, depth, or lit"
                            ),
                        )
                    })?;
                    config.render_settings.mode = render_mode;
                    if config.output_path == Path::new(DEFAULT_OUTPUT_PATH) {
                        config.output_path = PathBuf::from(format!(
                            "margaret-m2a-{}.ppm",
                            config.render_settings.mode.as_str()
                        ));
                    }
                }
                "--width" => {
                    let value = next_argument(&mut arguments, "--width")?;
                    let width = parse_dimension(&value, "width")?;
                    config.image_size.width = width;
                }
                "--height" => {
                    let value = next_argument(&mut arguments, "--height")?;
                    let height = parse_dimension(&value, "height")?;
                    config.image_size.height = height;
                }
                "--output" => {
                    let value = next_argument(&mut arguments, "--output")?;
                    config.output_path = PathBuf::from(value);
                }
                "--help" | "-h" => {
                    print_usage();
                    config.show_help = true;
                    return Ok(config);
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("unknown argument '{text}'"),
                    ));
                }
            }
        }

        if config.image_size.width == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "width must be greater than zero",
            ));
        }

        if config.image_size.height == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "height must be greater than zero",
            ));
        }

        Ok(config)
    }
}

fn next_argument<I>(arguments: &mut I, flag_name: &str) -> std::io::Result<OsString>
where
    I: Iterator<Item = OsString>,
{
    arguments.next().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("missing value for {flag_name}"),
        )
    })
}

fn parse_dimension(value: &OsString, label: &str) -> std::io::Result<u32> {
    let text = value.to_string_lossy();
    text.parse::<u32>().map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid {label} '{text}'"),
        )
    })
}

fn print_usage() {
    println!("Usage: margaret-cli [--mode normals|albedo|depth|lit] [--width N] [--height N] [--output PATH]");
}

fn hardcoded_scene() -> SceneDescription {
    let camera = Camera::new(
        "main-camera",
        Point3::new(0.0, 0.0, 3.4),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::Y,
        40.0,
    );

    let red = MaterialId(0);
    let green = MaterialId(1);
    let white = MaterialId(2);
    let light = MaterialId(3);

    let mut scene = SceneDescription::new("m2a-hardcoded-lit-scene", camera);
    scene.materials.push(MaterialDescription::new(
        red,
        "red",
        MaterialKind::Diffuse {
            albedo: ColorRgb::new(0.8, 0.2, 0.2),
            emission: ColorRgb::BLACK,
        },
    ));
    scene.materials.push(MaterialDescription::new(
        green,
        "green",
        MaterialKind::Diffuse {
            albedo: ColorRgb::new(0.2, 0.8, 0.2),
            emission: ColorRgb::BLACK,
        },
    ));
    scene.materials.push(MaterialDescription::new(
        white,
        "white",
        MaterialKind::Diffuse {
            albedo: ColorRgb::new(0.8, 0.8, 0.8),
            emission: ColorRgb::BLACK,
        },
    ));
    scene.materials.push(MaterialDescription::new(
        light,
        "ceiling-light",
        MaterialKind::Diffuse {
            albedo: ColorRgb::BLACK,
            emission: ColorRgb::new(5.0, 4.8, 4.4),
        },
    ));

    scene.objects.push(make_quad(
        "floor",
        white,
        Point3::new(-1.2, -1.0, 1.2),
        Point3::new(1.2, -1.0, 1.2),
        Point3::new(1.2, -1.0, -1.2),
        Point3::new(-1.2, -1.0, -1.2),
    ));
    scene.objects.push(make_quad(
        "ceiling",
        white,
        Point3::new(-1.2, 1.0, -1.2),
        Point3::new(1.2, 1.0, -1.2),
        Point3::new(1.2, 1.0, 1.2),
        Point3::new(-1.2, 1.0, 1.2),
    ));
    scene.objects.push(make_quad(
        "back-wall",
        white,
        Point3::new(-1.2, -1.0, -1.2),
        Point3::new(1.2, -1.0, -1.2),
        Point3::new(1.2, 1.0, -1.2),
        Point3::new(-1.2, 1.0, -1.2),
    ));
    scene.objects.push(make_quad(
        "left-wall",
        red,
        Point3::new(-1.2, -1.0, -1.2),
        Point3::new(-1.2, -1.0, 1.2),
        Point3::new(-1.2, 1.0, 1.2),
        Point3::new(-1.2, 1.0, -1.2),
    ));
    scene.objects.push(make_quad(
        "right-wall",
        green,
        Point3::new(1.2, -1.0, 1.2),
        Point3::new(1.2, -1.0, -1.2),
        Point3::new(1.2, 1.0, -1.2),
        Point3::new(1.2, 1.0, 1.2),
    ));
    scene.objects.push(make_quad(
        "center-panel",
        white,
        Point3::new(-0.45, -1.0, -0.2),
        Point3::new(0.45, -1.0, -0.7),
        Point3::new(0.45, 0.2, -0.7),
        Point3::new(-0.45, 0.2, -0.2),
    ));
    scene.objects.push(make_quad(
        "light",
        light,
        Point3::new(-0.35, 0.99, -0.35),
        Point3::new(0.35, 0.99, -0.35),
        Point3::new(0.35, 0.99, 0.35),
        Point3::new(-0.35, 0.99, 0.35),
    ));

    scene
}

fn make_quad(
    name: &str,
    material_id: MaterialId,
    a: Point3,
    b: Point3,
    c: Point3,
    d: Point3,
) -> SceneObject {
    SceneObject::new(
        name,
        Geometry::TriangleMesh {
            triangles: vec![Triangle::new(a, b, c), Triangle::new(a, c, d)],
        },
        material_id,
    )
}

#[cfg(test)]
mod tests {
    use super::{hardcoded_scene, run_from_args, CliConfig, DEFAULT_DEPTH_MAX_DISTANCE};
    use margaret_core::render::{RenderDebugMode, RenderMode};
    use margaret_core::scene::Geometry;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn hardcoded_scene_contains_room_geometry_and_emissive_quad() {
        let scene = hardcoded_scene();

        assert_eq!(scene.objects.len(), 7);
        assert_eq!(scene.materials.len(), 4);
        assert_eq!(scene.lights.len(), 0);

        for object in &scene.objects {
            match &object.geometry {
                Geometry::TriangleMesh { triangles } => assert_eq!(triangles.len(), 2),
            }
        }
    }

    #[test]
    fn cli_config_parses_debug_mode_and_output() {
        let config = CliConfig::parse(vec![
            OsString::from("margaret-cli"),
            OsString::from("--mode"),
            OsString::from("albedo"),
            OsString::from("--width"),
            OsString::from("64"),
            OsString::from("--height"),
            OsString::from("48"),
            OsString::from("--output"),
            OsString::from("frame.ppm"),
        ])
        .unwrap();

        assert_eq!(
            config.render_settings.mode,
            RenderMode::Debug(RenderDebugMode::FlatAlbedo)
        );
        assert_eq!(
            config.render_settings.depth_max_distance,
            DEFAULT_DEPTH_MAX_DISTANCE
        );
        assert_eq!(config.image_size.width, 64);
        assert_eq!(config.image_size.height, 48);
        assert_eq!(config.output_path, PathBuf::from("frame.ppm"));
        assert!(!config.show_help);
    }

    #[test]
    fn cli_config_parses_lit_mode() {
        let config = CliConfig::parse(vec![
            OsString::from("margaret-cli"),
            OsString::from("--mode"),
            OsString::from("lit"),
        ])
        .unwrap();

        assert_eq!(config.render_settings.mode, RenderMode::Lit);
        assert_eq!(config.output_path, PathBuf::from("margaret-m2a-lit.ppm"));
    }

    #[test]
    fn cli_config_rejects_unknown_mode() {
        let error = CliConfig::parse(vec![
            OsString::from("margaret-cli"),
            OsString::from("--mode"),
            OsString::from("wireframe"),
        ])
        .unwrap_err();

        assert!(error.to_string().contains("unsupported render mode"));
    }

    #[test]
    fn cli_config_updates_default_output_name_when_mode_changes() {
        let config = CliConfig::parse(vec![
            OsString::from("margaret-cli"),
            OsString::from("--mode"),
            OsString::from("depth"),
        ])
        .unwrap();

        assert_eq!(
            config.render_settings.mode,
            RenderMode::Debug(RenderDebugMode::Depth)
        );
        assert_eq!(config.output_path, PathBuf::from("margaret-m2a-depth.ppm"));
    }

    #[test]
    fn run_from_args_writes_lit_image_to_disk() {
        let mut output_path = std::env::temp_dir();
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        output_path.push(format!("margaret-lit-test-{unique_id}.ppm"));

        run_from_args(vec![
            OsString::from("margaret-cli"),
            OsString::from("--mode"),
            OsString::from("lit"),
            OsString::from("--width"),
            OsString::from("16"),
            OsString::from("--height"),
            OsString::from("12"),
            OsString::from("--output"),
            output_path.as_os_str().to_os_string(),
        ])
        .unwrap();

        let metadata = std::fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 0);

        std::fs::remove_file(output_path).unwrap();
    }
}
