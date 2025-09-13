use nalgebra_glm::Vec3;
use crate::material::Material;

pub struct Plane {
    pub point: Vec3,     // Un punto en el plano
    pub normal: Vec3,    // Vector normal del plano
    pub material: Material,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, material: Material) -> Self {
        Plane {
            point,
            normal: nalgebra_glm::normalize(&normal),
            material,
        }
    }
    
    pub fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        let denom = nalgebra_glm::dot(&self.normal, ray_direction);
        
        // Si el rayo es paralelo al plano
        if denom.abs() < 1e-6 {
            return None;
        }
        
        let t = nalgebra_glm::dot(&(self.point - ray_origin), &self.normal) / denom;
        
        if t > 0.001 {
            Some(t)
        } else {
            None
        }
    }
    
    pub fn get_normal(&self, _point: &Vec3) -> Vec3 {
        self.normal
    }
}