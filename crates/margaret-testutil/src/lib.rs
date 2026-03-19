use margaret_core::camera::Camera;
use margaret_core::color::ColorRgb;
use margaret_core::image::ImageSize;
use margaret_core::material::{MaterialDescription, MaterialId, MaterialKind};
use margaret_core::math::{Point3, Vec3};
use margaret_core::scene::{Geometry, SceneDescription, SceneObject, Triangle};

pub fn sample_image_size() -> ImageSize {
    ImageSize::new(640, 360)
}

pub fn sample_scene() -> SceneDescription {
    let camera = Camera::new(
        "main",
        Point3::new(0.0, 0.0, 3.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::Y,
        45.0,
    );
    let material_id = MaterialId(0);

    let mut scene = SceneDescription::new("triangle-scene", camera);
    scene.materials.push(MaterialDescription::new(
        material_id,
        "matte-gray",
        MaterialKind::Diffuse {
            albedo: ColorRgb::new(0.5, 0.5, 0.5),
            emission: ColorRgb::BLACK,
        },
    ));
    scene.objects.push(SceneObject::new(
        "preview-triangles",
        Geometry::TriangleMesh {
            triangles: vec![Triangle::new(
                Point3::new(-1.0, -1.0, 0.0),
                Point3::new(1.0, -1.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
            )],
        },
        material_id,
    ));
    scene
}
