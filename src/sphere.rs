use nalgebra_glm::Vec3;
use crate::material::Material;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Sphere {
            center,
            radius,
            material,
        }
    }
    
    pub fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        // Vector desde el origen del rayo hasta el centro de la esfera
        let oc = ray_origin - self.center;
        
        // Coeficientes para la ecuación cuadrática
        // a = dot(ray_direction, ray_direction) = 1.0 (porque ray_direction está normalizado)
        let a = nalgebra_glm::dot(ray_direction, ray_direction);
        
        // b = 2.0 * dot(oc, ray_direction)
        let b = 2.0 * nalgebra_glm::dot(&oc, ray_direction);
        
        // c = dot(oc, oc) - radius^2
        let c = nalgebra_glm::dot(&oc, &oc) - self.radius * self.radius;
        
        // Discriminante de la ecuación cuadrática
        let discriminant = b * b - 4.0 * a * c;
        
        // El rayo intersecta la esfera si el discriminante es mayor que cero
        if discriminant > 0.0 {
            // Calculamos ambas soluciones y tomamos la más cercana (positiva)
            let sqrt_discriminant = discriminant.sqrt();
            let t1 = (-b - sqrt_discriminant) / (2.0 * a);
            let t2 = (-b + sqrt_discriminant) / (2.0 * a);
            
            // Retornamos la distancia más cercana que sea positiva
            if t1 > 0.0 {
                Some(t1)
            } else if t2 > 0.0 {
                Some(t2)
            } else {
                None
            }
        } else {
            None
        }
    }
}