fn render(framebuffer: &mut Framebuffer, cube_grid: &SimpleCubeGrid, floor: &Plane, lights: &[Light], camera: &OrbitCamera, grass_texture: &Texture, dirt_texture: &Texture, center_texture: &Texture, water_texture: &Texture, lava_texture: &Texture, stats: &mut RenderStats) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;
            let screen_x = screen_x * aspect_ratio;
            
            let ray_direction = camera.get_ray_direction(screen_x, screen_y);
            let pixel_color = cast_ray(&camera.eye, &ray_direction, cube_grid, floor, lights, grass_texture, dirt_texture, center_texture, water_texture, lava_texture, stats);
            
            framebuffer.set_current_color(pixel_color);
            framebuffer.point(x, y);
        }
    }
}

fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, cube_grid: &SimpleCubeGrid, floor: &Plane, lights: &[Light], grass_texture: &Texture, dirt_texture: &Texture, center_texture: &Texture, water_texture: &Texture, lava_texture: &Texture, stats: &mut RenderStats) -> Color {
    let background_color = Color::new(135, 206, 235); // Cielo azul
    let mut closest_distance = f32::INFINITY;
    let mut hit_material: Option<&Material> = None;
    let mut hit_point = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_normal = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_object = 0; // 0=none, 1=cube, 2=center_cube, 3=water, 4=lava, 5=floor
    let mut hit_cube: Option<&Cube> = None;
    let mut cube_layer = 0;
    
    stats.rays_cast += 1;
    
    // Probar intersección con cubos del grid
    if let Some((cube_index, distance, object_type)) = cube_grid.ray_intersect(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance {
            closest_distance = distance;
            
            match object_type {
                1 => { // Cubo normal del grid
                    let cube = &cube_grid.cubes[cube_index];
                    hit_material = Some(&cube.material);
                    hit_point = ray_origin + ray_direction * distance;
                    hit_normal = cube.get_normal(&hit_point);
                    hit_cube = Some(cube);
                    // Determinar capa: 0-20=hierba, 21-45=tierra, 46+=piedra
                    cube_layer = if cube_index < 21 { 0 } else if cube_index < 46 { 1 } else { 2 };
                    hit_object = 1;
                }
                2 => { // Cubo central
                    hit_material = Some(&cube_grid.center_cube.material);
                    hit_point = ray_origin + ray_direction * distance;
                    hit_normal = cube_grid.center_cube.get_normal(&hit_point);
                    hit_cube = Some(&cube_grid.center_cube);
                    hit_object = 2;
                }
                3 => { // Agua
                    let water_plane = &cube_grid.water_planes[cube_index];
                    hit_material = Some(&water_plane.material);
                    hit_point = ray_origin + ray_direction * distance;
                    hit_normal = water_plane.get_normal(&hit_point);
                    hit_object = 3;
                }
                4 => { // Lava
                    let lava_plane = &cube_grid.lava_planes[cube_index];
                    hit_material = Some(&lava_plane.material);
                    hit_point = ray_origin + ray_direction * distance;
                    hit_normal = lava_plane.get_normal(&hit_point);
                    hit_object = 4;
                }
                _ => {}
            }
            stats.hits += 1;
        }
    }
    
    // Probar intersección con suelo
    stats.objects_tested += 1;
    if let Some(distance) = floor.ray_intersect(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance {
            hit_material = Some(&floor.material);
            hit_point = ray_origin + ray_direction * distance;
            hit_normal = floor.get_normal(&hit_point);
            hit_object = 5;
            hit_cube = None;
            stats.hits += 1;
        }
    }
    
    if hit_object == 0 {
        stats.misses += 1;
        return background_color;
    }
    
    if let Some(material) = hit_material {
        calculate_lighting(&hit_point, &hit_normal, material, lights, cube_grid, floor, grass_texture, dirt_texture, center_texture, water_texture, lava_texture, hit_object, hit_cube, cube_layer)
    } else {
        background_color
    }
}

fn calculate_lighting(hit_point: &Vec3, normal: &Vec3, material: &Material, lights: &[Light], cube_grid: &SimpleCubeGrid, floor: &Plane, grass_texture: &Texture, dirt_texture: &Texture, center_texture: &Texture, water_texture: &Texture, lava_texture: &Texture, hit_object: u8, hit_cube: Option<&Cube>, cube_layer: usize) -> Color {
    let mut total_r: f32 = 0.0;
    let mut total_g: f32 = 0.0;
    let mut total_b: f32 = 0.0;
    
    // Obtener color base
    let base_color = match hit_object {
        1 => { // Cubo normal del grid
            if material.has_texture {
                if let Some(cube) = hit_cube {
                    let (u, v) = cube.get_uv_coordinates(hit_point);
                    match cube_layer {
                        0 => grass_texture.sample(u, v), // Capa superior = hierba
                        1 => dirt_texture.sample(u, v),  // Capa media = tierra
                        2 => dirt_texture.sample(u, v),  // Capa inferior = piedra (usar dirt por ahora)
                        _ => grass_texture.sample(u, v),
                    }
                } else {
                    material.diffuse
                }
            } else {
                material.diffuse
            }
        }
        2 => { // Cubo central
            if material.has_texture {
                if let Some(cube) = hit_cube {
                    let (u, v) = cube.get_uv_coordinates(hit_point);
                    center_texture.sample(u, v)
                } else {
                    material.diffuse
                }
            } else {
                material.diffuse
            }
        }
        3 => { // Agua - usar textura procedural animada
            let (u, v) = ((hit_point.x * 0.5) % 1.0, (hit_point.z * 0.5) % 1.0);
            water_texture.sample(u, v)
        }
        4 => { // Lava - usar textura procedural animada
            let (u, v) = ((hit_point.x * 0.3) % 1.0, (hit_point.z * 0.3) % 1.0);
            lava_texture.sample(u, v)
        }
        _ => material.diffuse // Suelo y otros
    };
    
    // Luz ambiental (lava brilla más)
    let ambient_strength = match hit_object {
        4 => 0.6, // Lava emite luz propia
        3 => 0.3, // Agua refleja más luz ambiente
        _ => 0.2  // Normal
    };
    
    total_r += base_color.r as f32 * ambient_strength;
    total_g += base_color.g as f32 * ambient_strength;
    total_b += base_color.b as f32 * ambient_strength;
    
    // Iluminación directa
    for light in lights {
        let light_dir = nalgebra_glm::normalize(&(light.position - hit_point));
        let light_distance = nalgebra_glm::distance(&light.position, hit_point);
        
        // Verificar sombras (agua y lava no proyectan sombras sólidas)
        let shadow_ray_origin = hit_point + normal * 0.001;
        let mut in_shadow = false;
        
        if hit_object != 3 && hit_object != 4 { // No calcular sombras para agua/lava
            in_shadow = cube_grid.ray_intersect_shadow(&shadow_ray_origin, &light_dir, light_distance);
            
            if !in_shadow {
                if let Some(shadow_distance) = floor.ray_intersect(&shadow_ray_origin, &light_dir) {
                    if shadow_distance > 0.001 && shadow_distance < light_distance {
                        in_shadow = true;
                    }
                }
            }
        }
        
        if !in_shadow {
            let diff = nalgebra_glm::dot(normal, &light_dir).max(0.0);
            let attenuation = 1.0 / (1.0 + 0.05 * light_distance + 0.005 * light_distance * light_distance);
            
            // Multiplicadores especiales por tipo
            let light_contribution = match hit_object {
                3 => diff * light.intensity * attenuation * 1.2, // Agua más brillante
                4 => diff * light.intensity * attenuation * 0.8, // Lava menos afectada por luz externa
                _ => diff * light.intensity * attenuation
            };
            
            total_r += base_color.r as f32 * light.color.r as f32 / 255.0 * light_contribution;
            total_g += base_color.g as f32 * light.color.g as f32 / 255.0 * light_contribution;
            total_b += base_color.b as f32 * light.color.b as f32 / 255.0 * light_contribution;
        }
    }
    
    Color::new(total_r.min(255.0) as u8, total_g.min(255.0) as u8, total_b.min(255.0) as u8)
}mod framebuffer;
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

pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light { position, color, intensity }
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
        if denom.abs() < 1e-6 { return None; }
        let t = nalgebra_glm::dot(&(self.point - ray_origin), &self.normal) / denom;
        if t > 0.001 { Some(t) } else { None }
    }
    
    pub fn get_normal(&self, _point: &Vec3) -> Vec3 {
        self.normal
    }
}

#[derive(Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Texture {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let img = open(path)?;
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        let data = rgb_img.into_raw();
        Ok(Texture { width, height, data })
    }
    
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let x = ((u * self.width as f32) as u32).min(self.width - 1);
        let y = ((v * self.height as f32) as u32).min(self.height - 1);
        let index = ((y * self.width + x) * 3) as usize;
        
        if index + 2 < self.data.len() {
            Color::new(self.data[index], self.data[index + 1], self.data[index + 2])
        } else {
            Color::new(255, 0, 255)
        }
    }
    
    pub fn create_grass_texture() -> Self {
        let size = 64;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise = ((x * 17 + y * 13) % 32) as f32 / 32.0;
                let base_green = 34 + (noise * 30.0) as u8;
                data.extend_from_slice(&[0, base_green + 105, 0]);
            }
        }
        Texture { width: size, height: size, data }
    }
    
    pub fn create_dirt_texture() -> Self {
        let size = 64;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise = ((x * 23 + y * 19) % 40) as f32 / 40.0;
                let brown_r = 139 + (noise * 20.0) as u8;
                let brown_g = 90 + (noise * 15.0) as u8;
                let brown_b = 43 + (noise * 10.0) as u8;
                data.extend_from_slice(&[brown_r, brown_g, brown_b]);
            }
        }
        Texture { width: size, height: size, data }
    }
    
    pub fn create_water_texture() -> Self {
        let size = 64;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                // Crear ondas de agua procedurales
                let wave1 = ((x as f32 * 0.3).sin() + (y as f32 * 0.4).sin()) * 15.0;
                let wave2 = ((x as f32 * 0.1 + y as f32 * 0.2).sin()) * 10.0;
                let base_blue = 64 + wave1 as i32 + wave2 as i32;
                let base_green = 164 + (wave1 * 0.5) as i32;
                
                data.extend_from_slice(&[
                    base_blue.clamp(40, 100) as u8,
                    base_green.clamp(140, 200) as u8,
                    223
                ]);
            }
        }
        Texture { width: size, height: size, data }
    }
    
    pub fn create_lava_texture() -> Self {
        let size = 64;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                // Crear efecto de lava procedural
                let noise1 = ((x * 31 + y * 17) % 64) as f32 / 64.0;
                let noise2 = ((x * 13 + y * 29) % 32) as f32 / 32.0;
                let intensity = (noise1 + noise2 * 0.5).clamp(0.0, 1.0);
                
                if intensity > 0.7 {
                    // Zonas muy calientes - amarillo/blanco
                    data.extend_from_slice(&[255, 255, (100 + intensity * 155.0) as u8]);
                } else if intensity > 0.4 {
                    // Zonas calientes - rojo/naranja
                    data.extend_from_slice(&[255, (50 + intensity * 150.0) as u8, 0]);
                } else {
                    // Zonas más frías - rojo oscuro
                    data.extend_from_slice(&[(100 + intensity * 100.0) as u8, 0, 0]);
                }
            }
        }
        Texture { width: size, height: size, data }
    }
    
    pub fn create_special_texture() -> Self {
        let size = 64;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let checker = ((x / 8) + (y / 8)) % 2;
                if checker == 0 {
                    data.extend_from_slice(&[50, 100, 200]); // Azul
                } else {
                    data.extend_from_slice(&[200, 200, 255]); // Azul claro
                }
            }
        }
        Texture { width: size, height: size, data }
    }
}

pub struct SimpleCubeGrid {
    pub cubes: Vec<Cube>,
    pub center_cube: Cube,
    pub water_planes: Vec<Plane>,
    pub lava_planes: Vec<Plane>,
}

impl SimpleCubeGrid {
    pub fn new(center: Vec3, cube_size: f32) -> Self {
        let mut cubes = Vec::new();
        let grid_size = 5; // 5x5 para más espacio
        let spacing = cube_size * 0.99; // Muy poco espacio para eliminar "entrecortado"
        let offset = (grid_size as f32 * spacing) / 2.0 - spacing / 2.0;
        
        // Capa 1: Hierba (superior) 5x5 con hueco central
        for z in 0..grid_size {
            for x in 0..grid_size {
                // Crear hueco 2x2 en el centro para el cubo central
                if (x == 2 || x == 1) && (z == 2 || z == 1) {
                    continue; // Saltar cubos del centro
                }
                
                let pos = Vec3::new(
                    center.x + x as f32 * spacing - offset,
                    center.y,
                    center.z + z as f32 * spacing - offset,
                );
                
                cubes.push(Cube::new(pos, cube_size, Material::grass_top()));
            }
        }
        
        // Capa 2: Tierra (media) 5x5 completa
        for z in 0..grid_size {
            for x in 0..grid_size {
                let pos = Vec3::new(
                    center.x + x as f32 * spacing - offset,
                    center.y - cube_size * 1.01, // Muy poca separación
                    center.z + z as f32 * spacing - offset,
                );
                
                cubes.push(Cube::new(pos, cube_size, Material::dirt_layer()));
            }
        }
        
        // Capa 3: Piedra (inferior) 5x5 completa
        for z in 0..grid_size {
            for x in 0..grid_size {
                let pos = Vec3::new(
                    center.x + x as f32 * spacing - offset,
                    center.y - cube_size * 2.02, // Muy poca separación
                    center.z + z as f32 * spacing - offset,
                );
                
                cubes.push(Cube::new(pos, cube_size, Material::stone_layer()));
            }
        }
        
        // Cubo central prominente
        let center_cube = Cube::new(
            Vec3::new(center.x, center.y + cube_size * 0.7, center.z),
            cube_size * 1.4,
            Material::center_block()
        );
        
        // Agua en el lado izquierdo (-X)
        let mut water_planes = Vec::new();
        for i in 0..3 {
            water_planes.push(Plane::new(
                Vec3::new(center.x - cube_size * 3.5, center.y - cube_size * 0.3 + i as f32 * 0.1, center.z + (i as f32 - 1.0) * cube_size),
                Vec3::new(0.0, 1.0, 0.0),
                Material::water_surface()
            ));
        }
        
        // Lava en el lado derecho (+X)
        let mut lava_planes = Vec::new();
        for i in 0..3 {
            lava_planes.push(Plane::new(
                Vec3::new(center.x + cube_size * 3.5, center.y - cube_size * 0.3 + i as f32 * 0.1, center.z + (i as f32 - 1.0) * cube_size),
                Vec3::new(0.0, 1.0, 0.0),
                Material::lava_surface()
            ));
        }
        
        SimpleCubeGrid { cubes, center_cube, water_planes, lava_planes }
    }
    
    pub fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<(usize, f32, u8)> {
        let mut closest_distance = f32::INFINITY;
        let mut closest_index = None;
        let mut object_type = 0; // 0=none, 1=cube, 2=center, 3=water, 4=lava
        
        // Probar cubo central
        if let Some(distance) = self.center_cube.ray_intersect(ray_origin, ray_direction) {
            if distance > 0.001 && distance < closest_distance {
                closest_distance = distance;
                closest_index = Some(0);
                object_type = 2;
            }
        }
        
        // Probar cubos del grid
        for (i, cube) in self.cubes.iter().enumerate() {
            if let Some(distance) = cube.ray_intersect(ray_origin, ray_direction) {
                if distance > 0.001 && distance < closest_distance {
                    closest_distance = distance;
                    closest_index = Some(i);
                    object_type = 1;
                }
            }
        }
        
        // Probar planos de agua
        for (i, water) in self.water_planes.iter().enumerate() {
            if let Some(distance) = water.ray_intersect(ray_origin, ray_direction) {
                if distance > 0.001 && distance < closest_distance {
                    closest_distance = distance;
                    closest_index = Some(i);
                    object_type = 3;
                }
            }
        }
        
        // Probar planos de lava
        for (i, lava) in self.lava_planes.iter().enumerate() {
            if let Some(distance) = lava.ray_intersect(ray_origin, ray_direction) {
                if distance > 0.001 && distance < closest_distance {
                    closest_distance = distance;
                    closest_index = Some(i);
                    object_type = 4;
                }
            }
        }
        
        closest_index.map(|idx| (idx, closest_distance, object_type))
    }
    
    pub fn ray_intersect_shadow(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        // Cubo central
        if let Some(distance) = self.center_cube.ray_intersect(ray_origin, ray_direction) {
            if distance > 0.001 && distance < max_distance {
                return true;
            }
        }
        
        // Cubos del grid
        for cube in &self.cubes {
            if let Some(distance) = cube.ray_intersect(ray_origin, ray_direction) {
                if distance > 0.001 && distance < max_distance {
                    return true;
                }
            }
        }
        
        false // Agua y lava no proyectan sombras sólidas
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    
    println!("Loading textures...");
    
    let grass_texture = match Texture::load_from_file("grass.png") {
        Ok(tex) => { println!("Grass texture loaded: {}x{}", tex.width, tex.height); tex }
        Err(_) => { println!("Using procedural grass texture"); Texture::create_grass_texture() }
    };
    
    let dirt_texture = match Texture::load_from_file("dirt.png") {
        Ok(tex) => { println!("Dirt texture loaded: {}x{}", tex.width, tex.height); tex }
        Err(_) => { println!("Using procedural dirt texture"); Texture::create_dirt_texture() }
    };
    
    let center_texture = match Texture::load_from_file("center.png") {
        Ok(tex) => { println!("Center texture loaded: {}x{}", tex.width, tex.height); tex }
        Err(_) => { println!("Using procedural center texture"); Texture::create_special_texture() }
    };
    
    let water_texture = match Texture::load_from_file("water.png") {
        Ok(tex) => { println!("Water texture loaded: {}x{}", tex.width, tex.height); tex }
        Err(_) => { println!("Using procedural water texture"); Texture::create_water_texture() }
    };
    
    let lava_texture = match Texture::load_from_file("lava.png") {
        Ok(tex) => { println!("Lava texture loaded: {}x{}", tex.width, tex.height); tex }
        Err(_) => { println!("Using procedural lava texture"); Texture::create_lava_texture() }
    };
    
    let mut camera = OrbitCamera::new(Vec3::new(0.0, 0.0, -5.0), 6.0);
    camera.orbit(0.7, 0.3);
    
    println!("Building 3-layer cube grid...");
    let cube_grid = SimpleCubeGrid::new(Vec3::new(0.0, 0.0, -5.0), 0.8);
    println!("Grid built: {} cubes + 1 center cube + {} water + {} lava planes", 
             cube_grid.cubes.len(), cube_grid.water_planes.len(), cube_grid.lava_planes.len());
    
    let floor = Plane::new(Vec3::new(0.0, -4.0, -5.0), Vec3::new(0.0, 1.0, 0.0), Material::stone_wall());
    
    let lights = vec![
        Light::new(Vec3::new(-2.0, 4.0, -3.0), Color::new(255, 200, 150), 1.2),
        Light::new(Vec3::new(3.0, 3.0, -4.0), Color::new(200, 220, 255), 0.8),
        Light::new(Vec3::new(0.0, 6.0, -2.0), Color::new(100, 120, 150), 0.4),
    ];
    
    let mut window = Window::new("Simple Cube Grid", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.set_target_fps(60);
    
    println!("\n=== CONTROLS ===");
    println!("Arrow Keys: Orbit around grid");
    println!("W/S: Zoom in/out");
    println!("Escape: Exit");
    println!("================\n");
    
    let mut frame_count = 0;
    let mut stats = RenderStats::new();
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let orbit_speed = 0.05;
        let zoom_speed = 0.2;
        
        if window.is_key_down(Key::Left) { camera.orbit(-orbit_speed, 0.0); }
        if window.is_key_down(Key::Right) { camera.orbit(orbit_speed, 0.0); }
        if window.is_key_down(Key::Up) { camera.orbit(0.0, orbit_speed); }
        if window.is_key_down(Key::Down) { camera.orbit(0.0, -orbit_speed); }
        if window.is_key_down(Key::W) { camera.zoom(-zoom_speed); }
        if window.is_key_down(Key::S) { camera.zoom(zoom_speed); }
        
        stats.reset();
        render(&mut framebuffer, &cube_grid, &floor, &lights, &camera, &grass_texture, &dirt_texture, &center_texture, &water_texture, &lava_texture, &mut stats);
        
        if frame_count % 120 == 0 {
            println!("\n--- Frame {} ---", frame_count);
            println!("Simple Grid rendering - {} objects", cube_grid.cubes.len() + 1);
            println!("Camera: distance {:.1}, yaw {:.2}, pitch {:.2}", camera.distance, camera.yaw, camera.pitch);
            stats.print_summary();
        }
        frame_count += 1;
        
        window.update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn render(framebuffer: &mut Framebuffer, cube_grid: &SimpleCubeGrid, floor: &Plane, lights: &[Light], camera: &OrbitCamera, grass_texture: &Texture, dirt_texture: &Texture, center_texture: &Texture, stats: &mut RenderStats) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;
            let screen_x = screen_x * aspect_ratio;
            
            let ray_direction = camera.get_ray_direction(screen_x, screen_y);
            let pixel_color = cast_ray(&camera.eye, &ray_direction, cube_grid, floor, lights, grass_texture, dirt_texture, center_texture, stats);
            
            framebuffer.set_current_color(pixel_color);
            framebuffer.point(x, y);
        }
    }
}

fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, cube_grid: &SimpleCubeGrid, floor: &Plane, lights: &[Light], grass_texture: &Texture, dirt_texture: &Texture, center_texture: &Texture, stats: &mut RenderStats) -> Color {
    let background_color = Color::new(135, 206, 235); // Cielo azul
    let mut closest_distance = f32::INFINITY;
    let mut hit_material: Option<&Material> = None;
    let mut hit_point = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_normal = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_object = 0; // 0=none, 1=cube, 2=center_cube, 3=water, 4=floor
    let mut hit_cube: Option<&Cube> = None;
    let mut cube_layer = 0;
    
    stats.rays_cast += 1;
    
    // Probar intersección con cubos del grid
    if let Some((cube_index, distance, object_type)) = cube_grid.ray_intersect(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance {
            closest_distance = distance;
            
            match object_type {
                1 => { // Cubo normal del grid
                    let cube = &cube_grid.cubes[cube_index];
                    hit_material = Some(&cube.material);
                    hit_point = ray_origin + ray_direction * distance;
                    hit_normal = cube.get_normal(&hit_point);
                    hit_cube = Some(cube);
                    cube_layer = if cube_index < 12 { 0 } else { 1 }; // 12 cubos superiores = hierba
                    hit_object = 1;
                }
                2 => { // Cubo central
                    hit_material = Some(&cube_grid.center_cube.material);
                    hit_point = ray_origin + ray_direction * distance;
                    hit_normal = cube_grid.center_cube.get_normal(&hit_point);
                    hit_cube = Some(&cube_grid.center_cube);
                    hit_object = 2;
                }
                3 => { // Agua
                    let water_plane = &cube_grid.water_planes[cube_index];
                    hit_material = Some(&water_plane.material);
                    hit_point = ray_origin + ray_direction * distance;
                    hit_normal = water_plane.get_normal(&hit_point);
                    hit_object = 3;
                }
                _ => {}
            }
            stats.hits += 1;
        }
    }
    
    // Probar intersección con suelo
    stats.objects_tested += 1;
    if let Some(distance) = floor.ray_intersect(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance {
            hit_material = Some(&floor.material);
            hit_point = ray_origin + ray_direction * distance;
            hit_normal = floor.get_normal(&hit_point);
            hit_object = 4;
            hit_cube = None;
            stats.hits += 1;
        }
    }
    
    if hit_object == 0 {
        stats.misses += 1;
        return background_color;
    }
    
    if let Some(material) = hit_material {
        calculate_lighting(&hit_point, &hit_normal, material, lights, cube_grid, floor, grass_texture, dirt_texture, center_texture, hit_object, hit_cube, cube_layer)
    } else {
        background_color
    }
}

fn calculate_lighting(hit_point: &Vec3, normal: &Vec3, material: &Material, lights: &[Light], cube_grid: &SimpleCubeGrid, floor: &Plane, grass_texture: &Texture, dirt_texture: &Texture, center_texture: &Texture, hit_object: u8, hit_cube: Option<&Cube>, cube_layer: usize) -> Color {
    let mut total_r: f32 = 0.0;
    let mut total_g: f32 = 0.0;
    let mut total_b: f32 = 0.0;
    
    // Obtener color base
    let base_color = match hit_object {
        1 => { // Cubo normal
            if material.has_texture {
                if let Some(cube) = hit_cube {
                    let (u, v) = cube.get_uv_coordinates(hit_point);
                    match cube_layer {
                        0 => grass_texture.sample(u, v), // Capa superior = hierba
                        1 => dirt_texture.sample(u, v),  // Capa inferior = tierra
                        _ => grass_texture.sample(u, v),
                    }
                } else {
                    material.diffuse
                }
            } else {
                material.diffuse
            }
        }
        2 => { // Cubo central
            if material.has_texture {
                if let Some(cube) = hit_cube {
                    let (u, v) = cube.get_uv_coordinates(hit_point);
                    center_texture.sample(u, v)
                } else {
                    material.diffuse
                }
            } else {
                material.diffuse
            }
        }
        3 => { // Agua - efecto especial
            // Agua con un poco de reflejo del cielo
            let sky_reflection = Color::new(135, 206, 235);
            let water_base = material.diffuse;
            
            // Mezclar agua con reflejo del cielo
            Color::new(
                ((water_base.r as f32 * 0.7) + (sky_reflection.r as f32 * 0.3)) as u8,
                ((water_base.g as f32 * 0.7) + (sky_reflection.g as f32 * 0.3)) as u8,
                ((water_base.b as f32 * 0.7) + (sky_reflection.b as f32 * 0.3)) as u8,
            )
        }
        _ => material.diffuse // Suelo y otros
    };
    
    // Luz ambiental
    let ambient_strength = if hit_object == 3 { 0.3 } else { 0.2 }; // Agua más brillante
    total_r += base_color.r as f32 * ambient_strength;
    total_g += base_color.g as f32 * ambient_strength;
    total_b += base_color.b as f32 * ambient_strength;
    
    // Iluminación directa
    for light in lights {
        let light_dir = nalgebra_glm::normalize(&(light.position - hit_point));
        let light_distance = nalgebra_glm::distance(&light.position, hit_point);
        
        // Verificar sombras (el agua no proyecta sombras sólidas)
        let shadow_ray_origin = hit_point + normal * 0.001;
        let mut in_shadow = false;
        
        if hit_object != 3 { // No calcular sombras para agua
            in_shadow = cube_grid.ray_intersect_shadow(&shadow_ray_origin, &light_dir, light_distance);
            
            if !in_shadow {
                if let Some(shadow_distance) = floor.ray_intersect(&shadow_ray_origin, &light_dir) {
                    if shadow_distance > 0.001 && shadow_distance < light_distance {
                        in_shadow = true;
                    }
                }
            }
        }
        
        if !in_shadow {
            let diff = nalgebra_glm::dot(normal, &light_dir).max(0.0);
            let attenuation = 1.0 / (1.0 + 0.05 * light_distance + 0.005 * light_distance * light_distance);
            
            // Agua tiene más brillo especular
            let light_contribution = if hit_object == 3 {
                diff * light.intensity * attenuation * 1.2 // Agua más brillante
            } else {
                diff * light.intensity * attenuation
            };
            
            total_r += base_color.r as f32 * light.color.r as f32 / 255.0 * light_contribution;
            total_g += base_color.g as f32 * light.color.g as f32 / 255.0 * light_contribution;
            total_b += base_color.b as f32 * light.color.b as f32 / 255.0 * light_contribution;
        }
    }
    
    Color::new(total_r.min(255.0) as u8, total_g.min(255.0) as u8, total_b.min(255.0) as u8)
}