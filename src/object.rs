use nalgebra_glm::Vec3;
use crate::sphere::Sphere;
use crate::cube::Cube;
use crate::material::Material;

pub enum Object {
    Sphere(Sphere),
    Cube(Cube),
}

impl Object {
    pub fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        match self {
            Object::Sphere(sphere) => sphere.ray_intersect(ray_origin, ray_direction),
            Object::Cube(cube) => cube.ray_intersect(ray_origin, ray_direction),
        }
    }
    
    pub fn get_material(&self) -> &Material {
        match self {
            Object::Sphere(sphere) => &sphere.material,
            Object::Cube(cube) => &cube.material,
        }
    }
    
    pub fn get_center(&self) -> Vec3 {
        match self {
            Object::Sphere(sphere) => sphere.center,
            Object::Cube(cube) => (cube.min + cube.max) * 0.5,
        }
    }
}