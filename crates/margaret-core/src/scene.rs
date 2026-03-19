use crate::camera::Camera;
use crate::light::Light;
use crate::material::{MaterialDescription, MaterialId};
use crate::math::{Point3, Vec3};

#[derive(Debug, Clone, PartialEq)]
pub struct SceneDescription {
    pub name: String,
    pub camera: Camera,
    pub materials: Vec<MaterialDescription>,
    pub objects: Vec<SceneObject>,
    pub lights: Vec<Light>,
}

impl SceneDescription {
    pub fn new(name: impl Into<String>, camera: Camera) -> Self {
        Self {
            name: name.into(),
            camera,
            materials: Vec::new(),
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneObject {
    pub name: String,
    pub geometry: Geometry,
    pub material_id: MaterialId,
}

impl SceneObject {
    pub fn new(name: impl Into<String>, geometry: Geometry, material_id: MaterialId) -> Self {
        Self {
            name: name.into(),
            geometry,
            material_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    TriangleMesh { triangles: Vec<Triangle> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub vertices: [Point3; 3],
}

impl Triangle {
    pub const fn new(a: Point3, b: Point3, c: Point3) -> Self {
        Self {
            vertices: [a, b, c],
        }
    }

    pub fn geometric_normal(&self) -> Vec3 {
        let edge_ab = self.vertices[1] - self.vertices[0];
        let edge_ac = self.vertices[2] - self.vertices[0];
        edge_ab.cross(edge_ac).normalized()
    }

    pub fn area(&self) -> f32 {
        let edge_ab = self.vertices[1] - self.vertices[0];
        let edge_ac = self.vertices[2] - self.vertices[0];
        edge_ab.cross(edge_ac).length() * 0.5
    }

    pub fn centroid(&self) -> Point3 {
        let sum = Vec3::new(
            self.vertices[0].x + self.vertices[1].x + self.vertices[2].x,
            self.vertices[0].y + self.vertices[1].y + self.vertices[2].y,
            self.vertices[0].z + self.vertices[1].z + self.vertices[2].z,
        );

        Point3::new(sum.x / 3.0, sum.y / 3.0, sum.z / 3.0)
    }
}
