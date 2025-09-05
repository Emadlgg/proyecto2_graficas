use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 2], // [diffuse, specular]
    pub refractive_index: f32,
}

impl Material {
    pub fn new(diffuse: Color) -> Self {
        Material {
            diffuse,
            specular: 0.0,
            albedo: [1.0, 0.0],
            refractive_index: 1.0,
        }
    }
    
    // Materiales predefinidos para el laboratorio
    pub fn old_wood() -> Self {
        Material {
            diffuse: Color::new(139, 90, 43),
            specular: 10.0,
            albedo: [0.9, 0.1],
            refractive_index: 1.0,
        }
    }
    
    pub fn rusty_metal() -> Self {
        Material {
            diffuse: Color::new(183, 65, 14),
            specular: 50.0,
            albedo: [0.6, 0.3],
            refractive_index: 1.0,
        }
    }
    
    pub fn clear_glass() -> Self {
        Material {
            diffuse: Color::new(255, 255, 255),
            specular: 125.0,
            albedo: [0.0, 0.5],
            refractive_index: 1.5,
        }
    }
    
    pub fn magic_crystal() -> Self {
        Material {
            diffuse: Color::new(147, 0, 211),
            specular: 100.0,
            albedo: [0.4, 0.6],
            refractive_index: 2.4,
        }
    }
    
    pub fn stone_wall() -> Self {
        Material {
            diffuse: Color::new(105, 105, 105),
            specular: 5.0,
            albedo: [0.9, 0.1],
            refractive_index: 1.0,
        }
    }
}