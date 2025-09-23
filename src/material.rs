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
    
    pub fn with_texture(diffuse: Color) -> Self {
        Material {
            diffuse,
            specular: 0.0,
            albedo: [1.0, 0.0],
            refractive_index: 1.0,
            has_texture: true,
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
    
    // Material para el cubo central
    pub fn center_block() -> Self {
        Material {
            diffuse: Color::new(70, 130, 180), // Azul acero
            specular: 15.0,
            albedo: [0.85, 0.15],
            refractive_index: 1.0,
            has_texture: true,
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
    
    // Material para agua
    pub fn water_surface() -> Self {
        Material {
            diffuse: Color::new(64, 164, 223), // Azul agua
            specular: 50.0,
            albedo: [0.3, 0.7], // Más reflectante
            refractive_index: 1.33, // Índice de refracción del agua
            has_texture: false,
        }
    }
    
    // Material para lava
    pub fn lava_surface() -> Self {
        Material {
            diffuse: Color::new(255, 100, 0), // Naranja/rojo lava
            specular: 20.0,
            albedo: [0.8, 0.2], // Menos reflectante que agua
            refractive_index: 1.0,
            has_texture: true, // Usará textura procedural de lava
        }
    }
    
    // Materiales antiguos para compatibilidad
    pub fn old_wood() -> Self {
        Material {
            diffuse: Color::new(139, 90, 43),
            specular: 10.0,
            albedo: [0.9, 0.1],
            refractive_index: 1.0,
            has_texture: false,
        }
    }
    
    pub fn old_wood_textured() -> Self {
        Material {
            diffuse: Color::new(139, 90, 43),
            specular: 10.0,
            albedo: [0.9, 0.1],
            refractive_index: 1.0,
            has_texture: true,
        }
    }
    
    pub fn rusty_metal() -> Self {
        Material {
            diffuse: Color::new(183, 65, 14),
            specular: 50.0,
            albedo: [0.6, 0.3],
            refractive_index: 1.0,
            has_texture: false,
        }
    }
    
    pub fn clear_glass() -> Self {
        Material {
            diffuse: Color::new(255, 255, 255),
            specular: 125.0,
            albedo: [0.0, 0.5],
            refractive_index: 1.5,
            has_texture: false,
        }
    }
    
    pub fn magic_crystal() -> Self {
        Material {
            diffuse: Color::new(147, 0, 211),
            specular: 100.0,
            albedo: [0.4, 0.6],
            refractive_index: 2.4,
            has_texture: false,
        }
    }
}