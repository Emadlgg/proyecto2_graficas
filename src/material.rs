use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 2], // [diffuse, specular]
    pub refractive_index: f32,
    pub has_texture: bool,
}

impl Material {
    pub fn new(diffuse: Color) -> Self {
        Material {
            diffuse,
            specular: 0.0,
            albedo: [1.0, 0.0],
            refractive_index: 1.0,
            has_texture: false,
        }
    }
    
    // Materiales para las capas del terreno
    pub fn grass_top() -> Self {
        Material {
            diffuse: Color::new(34, 139, 34), // Verde pasto
            specular: 5.0,
            albedo: [0.9, 0.1],
            refractive_index: 1.0,
            has_texture: true,
        }
    }
    
    pub fn dirt_layer() -> Self {
        Material {
            diffuse: Color::new(139, 90, 43), // Marrón tierra
            specular: 2.0,
            albedo: [0.95, 0.05],
            refractive_index: 1.0,
            has_texture: true,
        }
    }
    
    pub fn stone_layer() -> Self {
        Material {
            diffuse: Color::new(105, 105, 105), // Gris piedra
            specular: 10.0,
            albedo: [0.8, 0.2],
            refractive_index: 1.0,
            has_texture: true,
        }
    }
    
    // Nuevo material para el cubo central
    pub fn center_block() -> Self {
        Material {
            diffuse: Color::new(70, 130, 180), // Azul acero
            specular: 15.0,
            albedo: [0.85, 0.15],
            refractive_index: 1.0,
            has_texture: true, // Usará la textura center.png
        }
    }
    
    // Material para el suelo (plano)
    pub fn stone_wall() -> Self {
        Material {
            diffuse: Color::new(105, 105, 105),
            specular: 5.0,
            albedo: [0.9, 0.1],
            refractive_index: 1.0,
            has_texture: false,
        }
    }
}