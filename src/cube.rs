use nalgebra_glm::Vec3;
use crate::material::Material;

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        let half_size = size / 2.0;
        Cube {
            min: Vec3::new(
                center.x - half_size,
                center.y - half_size,
                center.z - half_size,
            ),
            max: Vec3::new(
                center.x + half_size,
                center.y + half_size,
                center.z + half_size,
            ),
            material,
        }
    }
    
    pub fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<f32> {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        
        for i in 0..3 {
            let ray_dir_component = ray_direction[i];
            let ray_origin_component = ray_origin[i];
            let box_min = self.min[i];
            let box_max = self.max[i];
            
            if ray_dir_component.abs() < 1e-6 {
                if ray_origin_component < box_min || ray_origin_component > box_max {
                    return None;
                }
            } else {
                let t1 = (box_min - ray_origin_component) / ray_dir_component;
                let t2 = (box_max - ray_origin_component) / ray_dir_component;
                
                let t_near = t1.min(t2);
                let t_far = t1.max(t2);
                
                t_min = t_min.max(t_near);
                t_max = t_max.min(t_far);
                
                if t_min > t_max {
                    return None;
                }
            }
        }
        
        if t_min > 0.0 {
            Some(t_min)
        } else if t_max > 0.0 {
            Some(t_max)
        } else {
            None
        }
    }
}