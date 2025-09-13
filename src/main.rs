mod framebuffer;
mod color;
mod cube;
mod camera;
mod material;
mod stats;

use framebuffer::Framebuffer;
use color::Color;
use cube::Cube;
use camera::OrbitCamera;
use material::Material;
use stats::RenderStats;
use nalgebra_glm::Vec3;
use minifb::{Key, Window, WindowOptions};
use image::open;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

// Definir todas las estructuras directamente en main.rs
pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
        }
    }
}

pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
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

// Estructura para manejar texturas
#[derive(Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGB data (3 bytes por pixel)
}

impl Texture {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let img = open(path)?;
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        let data = rgb_img.into_raw();
        
        Ok(Texture {
            width,
            height,
            data,
        })
    }
    
    pub fn sample(&self, u: f32, v: f32) -> Color {
        // Clamp UV coordinates to [0, 1]
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        
        // Convert UV to pixel coordinates
        let x = ((u * self.width as f32) as u32).min(self.width - 1);
        let y = ((v * self.height as f32) as u32).min(self.height - 1);
        
        // Calculate index in RGB data
        let index = ((y * self.width + x) * 3) as usize;
        
        if index + 2 < self.data.len() {
            Color::new(
                self.data[index],     // R
                self.data[index + 1], // G
                self.data[index + 2], // B
            )
        } else {
            Color::new(255, 0, 255) // Magenta for errors
        }
    }
    
    // Crear textura de prueba procedural si no hay archivo
    pub fn create_test_texture() -> Self {
        let size = 64;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        
        for y in 0..size {
            for x in 0..size {
                // Crear patrón de tablero de ajedrez
                let checker = ((x / 8) + (y / 8)) % 2;
                if checker == 0 {
                    data.extend_from_slice(&[139, 90, 43]); // Marrón
                } else {
                    data.extend_from_slice(&[101, 67, 33]); // Marrón más oscuro
                }
            }
        }
        
        Texture {
            width: size,
            height: size,
            data,
        }
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    
    // Cargar textura (con fallback a textura procedural)
    println!("Loading texture...");
    let texture = match Texture::load_from_file("texture.png") {
        Ok(tex) => {
            println!("Texture loaded successfully: {}x{}", tex.width, tex.height);
            tex
        }
        Err(_) => {
            println!("Could not load texture.png, using procedural texture");
            Texture::create_test_texture()
        }
    };
    
    // Crear cámara orbital con vista inicial en ángulo
    let mut camera = OrbitCamera::new(
        Vec3::new(0.0, 0.0, -5.0), // Target: el cubo
        6.0 // Distancia inicial
    );
    // Establecer ángulos iniciales para vista 3D
    camera.orbit(0.7, 0.3); // 40° horizontal, 17° vertical
    
    // Crear cubo con material texturizado
    let cube = Cube::new(Vec3::new(0.0, 0.0, -5.0), 2.0, Material::old_wood_textured());
    
    // Crear suelo para que se vean las sombras
    let floor = Plane::new(
        Vec3::new(0.0, -2.0, -5.0), // Punto en el suelo (debajo del cubo)
        Vec3::new(0.0, 1.0, 0.0),   // Normal apuntando hacia arriba
        Material::stone_wall()      // Material de piedra gris
    );
    
    // Crear luces para el laboratorio
    let lights = vec![
        // Luz principal desde arriba (como una antorcha)
        Light::new(Vec3::new(-1.0, 3.0, -3.0), Color::new(255, 200, 150), 1.0),
        
        // Luz secundaria más suave desde otro ángulo
        Light::new(Vec3::new(2.0, 2.0, -4.0), Color::new(200, 220, 255), 0.6),
        
        // Luz ambiental muy suave
        Light::new(Vec3::new(0.0, 5.0, -8.0), Color::new(100, 120, 150), 0.3),
    ];
    
    // Crear ventana
    let mut window = Window::new(
        "Textured Cube - With Shadows",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });
    
    window.set_target_fps(60);
    
    // Test inicial simplificado
    println!("Testing cube and floor intersections...");
    let test_ray_origin = Vec3::new(0.0, 0.0, 0.0);
    let test_ray_direction = Vec3::new(0.0, 0.0, -1.0);
    
    if let Some(distance) = cube.ray_intersect(&test_ray_origin, &test_ray_direction) {
        println!("Cube: HIT at distance {:.2}", distance);
    } else {
        println!("Cube: MISS");
    }
    
    if let Some(distance) = floor.ray_intersect(&test_ray_origin, &test_ray_direction) {
        println!("Floor: HIT at distance {:.2}", distance);
    } else {
        println!("Floor: MISS");
    }
    println!();
    
    println!("\n=== CONTROLS ===");
    println!("Arrow Keys: Orbit around cube");
    println!("W/S: Zoom in/out");
    println!("Escape: Exit");
    println!("================\n");
    
    // Loop principal
    let mut frame_count = 0;
    let mut stats = RenderStats::new();
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Controles de cámara
        let orbit_speed = 0.05;
        let zoom_speed = 0.2;
        
        if window.is_key_down(Key::Left) {
            camera.orbit(-orbit_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(orbit_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, orbit_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, -orbit_speed);
        }
        if window.is_key_down(Key::W) {
            camera.zoom(-zoom_speed);
        }
        if window.is_key_down(Key::S) {
            camera.zoom(zoom_speed);
        }
        
        stats.reset();
        
        render(&mut framebuffer, &cube, &floor, &lights, &camera, &texture, &mut stats);
        
        if frame_count % 120 == 0 {
            println!("\n--- Frame {} ---", frame_count);
            println!("Textured cube + Floor rendering");
            println!("Camera: distance {:.1}, yaw {:.2}, pitch {:.2}", 
                camera.distance, camera.yaw, camera.pitch);
            stats.print_summary();
        }
        frame_count += 1;
        
        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

fn render(framebuffer: &mut Framebuffer, cube: &Cube, floor: &Plane, lights: &[Light], camera: &OrbitCamera, texture: &Texture, stats: &mut RenderStats) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;
            let screen_x = screen_x * aspect_ratio;
            
            // Usar la cámara para calcular la dirección del rayo
            let ray_direction = camera.get_ray_direction(screen_x, screen_y);
            let pixel_color = cast_ray(&camera.eye, &ray_direction, cube, floor, lights, texture, stats);
            
            framebuffer.set_current_color(pixel_color);
            framebuffer.point(x, y);
        }
    }
}

fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, cube: &Cube, floor: &Plane, lights: &[Light], texture: &Texture, stats: &mut RenderStats) -> Color {
    let background_color = Color::new(15, 25, 35);
    let mut closest_distance = f32::INFINITY;
    let mut hit_material: Option<&Material> = None;
    let mut hit_point = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_normal = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_object = 0; // 0 = none, 1 = cube, 2 = floor
    
    stats.rays_cast += 1;
    
    // Probar intersección con el cubo
    stats.objects_tested += 1;
    if let Some(distance) = cube.ray_intersect(ray_origin, ray_direction) && distance > 0.001 && distance < closest_distance {
        closest_distance = distance;
        hit_material = Some(&cube.material);
        hit_point = ray_origin + ray_direction * distance;
        hit_normal = cube.get_normal(&hit_point);
        hit_object = 1;
    }
    
    // Probar intersección con el suelo
    stats.objects_tested += 1;
    if let Some(distance) = floor.ray_intersect(ray_origin, ray_direction) && distance > 0.001 && distance < closest_distance {
        closest_distance = distance;
        hit_material = Some(&floor.material);
        hit_point = ray_origin + ray_direction * distance;
        hit_normal = floor.get_normal(&hit_point);
        hit_object = 2;
    }
    
    if let Some(material) = hit_material {
        stats.hits += 1;
        
        // Calcular iluminación con sombras y texturas
        calculate_lighting(&hit_point, &hit_normal, material, lights, cube, floor, texture, hit_object)
    } else {
        stats.misses += 1;
        background_color
    }
}

fn calculate_lighting(hit_point: &Vec3, normal: &Vec3, material: &Material, lights: &[Light], cube: &Cube, floor: &Plane, texture: &Texture, hit_object: u8) -> Color {
    let mut total_r: f32 = 0.0;
    let mut total_g: f32 = 0.0;
    let mut total_b: f32 = 0.0;
    
    // Obtener color base (de textura o material)
    let base_color = if hit_object == 1 && material.has_texture {
        // Es el cubo y tiene textura
        let (u, v) = cube.get_uv_coordinates(hit_point);
        texture.sample(u, v)
    } else {
        // Usar color del material
        material.diffuse
    };
    
    // Luz ambiental
    let ambient_strength = 0.1;
    total_r += base_color.r as f32 * ambient_strength;
    total_g += base_color.g as f32 * ambient_strength;
    total_b += base_color.b as f32 * ambient_strength;
    
    for light in lights {
        let light_dir = nalgebra_glm::normalize(&(light.position - hit_point));
        let light_distance = nalgebra_glm::distance(&light.position, hit_point);
        
        // Verificar si hay sombra (probar tanto el cubo como el suelo)
        let shadow_ray_origin = hit_point + normal * 0.001;
        let mut in_shadow = false;
        
        // Verificar si el cubo proyecta sombra
        if let Some(shadow_distance) = cube.ray_intersect(&shadow_ray_origin, &light_dir) {
            if shadow_distance > 0.001 && shadow_distance < light_distance {
                in_shadow = true;
            }
        }
        
        // Verificar si el suelo proyecta sombra (aunque es raro)
        if !in_shadow {
            if let Some(shadow_distance) = floor.ray_intersect(&shadow_ray_origin, &light_dir) {
                if shadow_distance > 0.001 && shadow_distance < light_distance {
                    in_shadow = true;
                }
            }
        }
        
        if !in_shadow {
            // Calcular luz difusa
            let diff = nalgebra_glm::dot(normal, &light_dir).max(0.0);
            let attenuation = 1.0 / (1.0 + 0.1 * light_distance + 0.01 * light_distance * light_distance);
            
            let light_contribution = diff * light.intensity * attenuation;
            
            total_r += base_color.r as f32 * light.color.r as f32 / 255.0 * light_contribution;
            total_g += base_color.g as f32 * light.color.g as f32 / 255.0 * light_contribution;
            total_b += base_color.b as f32 * light.color.b as f32 / 255.0 * light_contribution;
        }
    }
    
    Color::new(
        total_r.min(255.0) as u8,
        total_g.min(255.0) as u8,
        total_b.min(255.0) as u8,
    )
}