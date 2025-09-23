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
use material::{Material, MaterialType};
use stats::RenderStats;
use nalgebra_glm::Vec3;
use minifb::{Key, Window, WindowOptions};
use image::open;

const WIDTH: usize = 300;
const HEIGHT: usize = 300;

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
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise1 = ((x * 17 + y * 13) % 16) as f32 / 16.0;
                let noise2 = ((x * 7 + y * 11) % 8) as f32 / 8.0;
                let combined_noise = (noise1 + noise2 * 0.3).clamp(0.0, 1.0);
                
                let base_green = 160 + (combined_noise * 60.0) as u8;
                let r = (25.0 + combined_noise * 35.0) as u8;
                let b = (25.0 + combined_noise * 30.0) as u8;
                
                data.extend_from_slice(&[r, base_green, b]);
            }
        }
        Texture { width: size, height: size, data }
    }
    
    pub fn create_stone_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise1 = ((x * 23 + y * 19) % 32) as f32 / 32.0;
                let noise2 = ((x * 7 + y * 13) % 16) as f32 / 16.0;
                let combined_noise = (noise1 + noise2 * 0.4).clamp(0.0, 1.0);
                
                let base_gray = (70.0 + combined_noise * 40.0) as u8;
                let variation = (combined_noise * 15.0) as u8;
                
                data.extend_from_slice(&[
                    base_gray + variation,
                    base_gray + (variation / 2),
                    base_gray
                ]);
            }
        }
        Texture { width: size, height: size, data }
    }
    
    pub fn create_dirt_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise1 = ((x * 13 + y * 17) % 24) as f32 / 24.0;
                let noise2 = ((x * 29 + y * 7) % 16) as f32 / 16.0;
                let combined_noise = (noise1 + noise2 * 0.5).clamp(0.0, 1.0);
                
                let brown_r = (140.0 + combined_noise * 50.0) as u8;
                let brown_g = (85.0 + combined_noise * 35.0) as u8;
                let brown_b = (35.0 + combined_noise * 25.0) as u8;
                
                data.extend_from_slice(&[brown_r, brown_g, brown_b]);
            }
        }
        Texture { width: size, height: size, data }
    }
    
    pub fn create_water_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let wave1 = ((x as f32 * 0.4).sin() + (y as f32 * 0.3).sin()) * 0.3;
                let wave2 = ((x as f32 * 0.2 + y as f32 * 0.2).sin()) * 0.2;
                let wave_intensity = (wave1 + wave2).clamp(-0.5, 0.5);
                
                let base_blue = 40 + (wave_intensity * 20.0) as i32;
                let base_green = 140 + (wave_intensity * 30.0) as i32;
                let base_alpha = 200;
                
                data.extend_from_slice(&[
                    base_blue.clamp(20, 80) as u8,
                    base_green.clamp(120, 180) as u8,
                    base_alpha as u8
                ]);
            }
        }
        Texture { width: size, height: size, data }
    }
    
    pub fn create_lava_texture() -> Self {
        let size = 32;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise1 = ((x * 31 + y * 17) % 32) as f32 / 32.0;
                let noise2 = ((x * 13 + y * 29) % 16) as f32 / 16.0;
                let intensity = (noise1 + noise2 * 0.6).clamp(0.0, 1.0);
                
                if intensity > 0.7 {
                    data.extend_from_slice(&[255, 255, (150.0 + intensity * 105.0) as u8]);
                } else if intensity > 0.4 {
                    data.extend_from_slice(&[255, (120.0 + intensity * 135.0) as u8, 30]);
                } else {
                    data.extend_from_slice(&[(180.0 + intensity * 75.0) as u8, 20, 0]);
                }
            }
        }
        Texture { width: size, height: size, data }
    }
}

pub struct OptimizedDiorama {
    pub cubes: Vec<Cube>,
    pub water_planes: Vec<Plane>,
    pub lava_planes: Vec<Plane>,
    pub bounding_box_min: Vec3,
    pub bounding_box_max: Vec3,
}

impl OptimizedDiorama {
    pub fn new(center: Vec3, cube_size: f32) -> Self {
        let mut cubes = Vec::new();
        let mut water_planes = Vec::new();
        let mut lava_planes = Vec::new();
        
        let grid_size = 12;
        let spacing = cube_size;
        let offset = (grid_size as f32 * spacing) / 2.0 - spacing / 2.0;
        
        let terrain_heights = Self::generate_terrain_heights(grid_size);
        
        let mut min_pos = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max_pos = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        
        for z in 0..grid_size {
            for x in 0..grid_size {
                let height = terrain_heights[z][x];
                
                for y_level in 0..=height {
                    let pos = Vec3::new(
                        center.x + x as f32 * spacing - offset,
                        center.y + (y_level as f32) * spacing,
                        center.z + z as f32 * spacing - offset,
                    );
                    
                    min_pos = Vec3::new(min_pos.x.min(pos.x), min_pos.y.min(pos.y), min_pos.z.min(pos.z));
                    max_pos = Vec3::new(max_pos.x.max(pos.x), max_pos.y.max(pos.y), max_pos.z.max(pos.z));
                    
                    let material = Self::determine_material(x, z, y_level, height);
                    
                    if Self::should_place_cube(x, z, y_level, height, grid_size) {
                        cubes.push(Cube::new(pos, cube_size, material));
                    }
                }
            }
        }
        
        Self::add_water_areas(&mut water_planes, center, cube_size, spacing, offset);
        Self::add_lava_areas(&mut lava_planes, center, cube_size, spacing, offset);
        
        OptimizedDiorama { 
            cubes, 
            water_planes, 
            lava_planes,
            bounding_box_min: min_pos - Vec3::new(2.0, 2.0, 2.0),
            bounding_box_max: max_pos + Vec3::new(2.0, 2.0, 2.0),
        }
    }
    
    fn generate_terrain_heights(grid_size: usize) -> Vec<Vec<usize>> {
        let mut heights = vec![vec![0; grid_size]; grid_size];
        
        for z in 0..grid_size {
            for x in 0..grid_size {
                let mut height = 1;
                
                if z <= 2 { height = 6; }
                if x <= 2 { height = 6; }
                if x >= grid_size - 3 && !(z >= 5 && z <= 7) { height = 6; }
                if z >= grid_size - 3 && !(x >= 3 && x <= 8) { height = 4; }
                
                if x >= 4 && x <= 7 && z >= 4 && z <= 7 { height = 3; }
                if x >= 5 && x <= 6 && z >= 5 && z <= 6 { height = 4; }
                
                if x >= 8 && x <= 10 && z >= 8 && z <= 10 { height = 2; }
                
                heights[z][x] = height;
            }
        }
        heights
    }
    
    fn determine_material(x: usize, z: usize, y_level: usize, max_height: usize) -> Material {
        if y_level == max_height {
            if x >= 4 && x <= 7 && z >= 4 && z <= 7 {
                Material::grass_top()
            } else {
                Material::stone_layer()
            }
        } else if y_level >= max_height - 1 {
            Material::dirt_layer()
        } else {
            Material::stone_layer()
        }
    }
    
    fn should_place_cube(x: usize, z: usize, y_level: usize, _max_height: usize, grid_size: usize) -> bool {
        if y_level >= 1 && y_level <= 2 && z >= 5 && z <= 7 && x >= 3 && x <= grid_size - 3 {
            return false;
        }
        
        if y_level >= 1 && y_level <= 3 && x >= 3 && x <= 5 && z >= 3 && z <= 5 {
            return false;
        }
        
        if y_level == 1 && x >= 8 && x <= 10 && z >= 8 && z <= 10 {
            return false;
        }
        
        true
    }
    
    // FUNCIONES VACÍAS - NO CREAR PLANOS GIGANTES
    fn add_water_areas(_water_planes: &mut Vec<Plane>, _center: Vec3, _cube_size: f32, _spacing: f32, _offset: f32) {
        // VACÍA - No agregar agua que cause paneles gigantes
    }
    
    fn add_lava_areas(_lava_planes: &mut Vec<Plane>, _center: Vec3, _cube_size: f32, _spacing: f32, _offset: f32) {
        // VACÍA - No agregar lava que cause paneles gigantes  
    }
    
    // SIMPLIFICADO - SOLO CUBOS
    pub fn ray_intersect_fast(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<(usize, f32, u8)> {
        if !self.ray_intersects_bbox(ray_origin, ray_direction) {
            return None;
        }
        
        let mut closest_distance = f32::INFINITY;
        let mut closest_index = None;
        
        // SOLO CUBOS - sin agua ni lava
        for (i, cube) in self.cubes.iter().enumerate() {
            if let Some(distance) = cube.ray_intersect(ray_origin, ray_direction) {
                if distance > 0.001 && distance < closest_distance {
                    closest_distance = distance;
                    closest_index = Some(i);
                    
                    if distance < 0.1 { break; }
                }
            }
        }
        
        closest_index.map(|idx| (idx, closest_distance, 1)) // Solo tipo 1 (cubos)
    }
    
    fn ray_intersects_bbox(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> bool {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        
        for i in 0..3 {
            if ray_direction[i].abs() < 1e-6 {
                if ray_origin[i] < self.bounding_box_min[i] || ray_origin[i] > self.bounding_box_max[i] {
                    return false;
                }
            } else {
                let t1 = (self.bounding_box_min[i] - ray_origin[i]) / ray_direction[i];
                let t2 = (self.bounding_box_max[i] - ray_origin[i]) / ray_direction[i];
                
                let t_near = t1.min(t2);
                let t_far = t1.max(t2);
                
                t_min = t_min.max(t_near);
                t_max = t_max.min(t_far);
                
                if t_min > t_max {
                    return false;
                }
            }
        }
        
        t_max > 0.0
    }
    
    pub fn ray_intersect_shadow_fast(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        for (i, cube) in self.cubes.iter().enumerate() {
            if i % 2 == 0 {
                if let Some(distance) = cube.ray_intersect(ray_origin, ray_direction) {
                    if distance > 0.001 && distance < max_distance {
                        return true;
                    }
                }
            }
        }
        false
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    
    let grass_texture = match Texture::load_from_file("grass.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_grass_texture()
    };
    
    let dirt_texture = match Texture::load_from_file("dirt.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_dirt_texture()
    };
    
    let stone_texture = match Texture::load_from_file("stone.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_stone_texture()
    };
    
    let water_texture = match Texture::load_from_file("water.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_water_texture()
    };
    
    let lava_texture = match Texture::load_from_file("lava.png") {
        Ok(tex) => tex,
        Err(_) => Texture::create_lava_texture()
    };
    
    let mut camera = OrbitCamera::new(Vec3::new(0.0, 2.0, 0.0), 10.0);
    camera.orbit(0.8, 0.4);
    
    let diorama = OptimizedDiorama::new(Vec3::new(0.0, 0.0, 0.0), 0.8);
    let floor = Plane::new(Vec3::new(0.0, -2.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Material::stone_wall());
    
    let lights = vec![
        Light::new(Vec3::new(-4.0, 8.0, -2.0), Color::new(255, 220, 180), 1.3),
        Light::new(Vec3::new(6.0, 6.0, 3.0), Color::new(180, 200, 255), 0.9),
    ];
    
    let mut window = Window::new("Minecraft Diorama - Raytracer", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.set_target_fps(60);
    
    let mut stats = RenderStats::new();
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let orbit_speed = 0.05;
        let zoom_speed = 0.4;
        
        if window.is_key_down(Key::Left) { camera.orbit(-orbit_speed, 0.0); }
        if window.is_key_down(Key::Right) { camera.orbit(orbit_speed, 0.0); }
        if window.is_key_down(Key::Up) { camera.orbit(0.0, orbit_speed); }
        if window.is_key_down(Key::Down) { camera.orbit(0.0, -orbit_speed); }
        if window.is_key_down(Key::W) { camera.zoom(-zoom_speed); }
        if window.is_key_down(Key::S) { camera.zoom(zoom_speed); }
        
        if window.is_key_down(Key::Space) {
            camera = OrbitCamera::new(Vec3::new(0.0, 2.0, 0.0), 10.0);
            camera.orbit(0.8, 0.4);
        }
        
        stats.reset();
        render_optimized(&mut framebuffer, &diorama, &floor, &lights, &camera, 
                        &grass_texture, &dirt_texture, &stone_texture, &water_texture, &lava_texture, &mut stats);
        
        window.update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn render_optimized(framebuffer: &mut Framebuffer, diorama: &OptimizedDiorama, floor: &Plane, 
                   lights: &[Light], camera: &OrbitCamera, grass_texture: &Texture, 
                   dirt_texture: &Texture, stone_texture: &Texture, water_texture: &Texture, 
                   lava_texture: &Texture, stats: &mut RenderStats) {
    
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    
    framebuffer.clear();
    
    let skip = 1;
    
    for y in (0..framebuffer.height).step_by(skip) {
        for x in (0..framebuffer.width).step_by(skip) {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;
            let screen_x = screen_x * aspect_ratio;
            
            let ray_direction = camera.get_ray_direction(screen_x, screen_y);
            let pixel_color = cast_ray_optimized(&camera.eye, &ray_direction, diorama, floor, 
                                               lights, grass_texture, dirt_texture, stone_texture, 
                                               water_texture, lava_texture, stats);
            
            framebuffer.set_current_color(pixel_color);
            
            for dy in 0..skip {
                for dx in 0..skip {
                    if x + dx < framebuffer.width && y + dy < framebuffer.height {
                        framebuffer.point(x + dx, y + dy);
                    }
                }
            }
        }
    }
}

fn cast_ray_optimized(ray_origin: &Vec3, ray_direction: &Vec3, diorama: &OptimizedDiorama, 
                     floor: &Plane, lights: &[Light], grass_texture: &Texture, dirt_texture: &Texture, 
                     stone_texture: &Texture, water_texture: &Texture, lava_texture: &Texture, 
                     stats: &mut RenderStats) -> Color {
    
    let background_color = if ray_direction.y > 0.3 {
        let sky_intensity = (ray_direction.y - 0.3).clamp(0.0, 0.7) / 0.7;
        Color::new(
            (100.0 + sky_intensity * 80.0) as u8,
            (180.0 + sky_intensity * 50.0) as u8,
            255
        )
    } else if ray_direction.y > -0.1 {
        Color::new(150, 200, 255)
    } else {
        Color::new(120, 160, 200)
    };
    
    let mut closest_distance = f32::INFINITY;
    let mut hit_material: Option<&Material> = None;
    let mut hit_point = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_normal = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_object = 0;
    let mut hit_cube: Option<&Cube> = None;
    
    stats.rays_cast += 1;
    
    // SOLO CUBOS - simplificado
    if let Some((object_index, distance, object_type)) = diorama.ray_intersect_fast(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance && object_type == 1 {
            closest_distance = distance;
            let cube = &diorama.cubes[object_index];
            hit_material = Some(&cube.material);
            hit_point = ray_origin + ray_direction * distance;
            hit_normal = cube.get_normal(&hit_point);
            hit_cube = Some(cube);
            hit_object = 1;
            stats.hits += 1;
        }
    }
    
    // Probar suelo
    if let Some(distance) = floor.ray_intersect(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance {
            hit_material = Some(&floor.material);
            hit_point = ray_origin + ray_direction * distance;
            hit_normal = floor.get_normal(&hit_point);
            hit_object = 5;
            stats.hits += 1;
        }
    }
    
    if hit_object == 0 {
        stats.misses += 1;
        return background_color;
    }
    
    if let Some(material) = hit_material {
        calculate_lighting_simple(&hit_point, &hit_normal, material, lights, diorama, 
                                 grass_texture, dirt_texture, stone_texture, 
                                 hit_object, hit_cube)
    } else {
        background_color
    }
}

fn calculate_lighting_simple(hit_point: &Vec3, normal: &Vec3, material: &Material, lights: &[Light], 
                            diorama: &OptimizedDiorama, grass_texture: &Texture, 
                            dirt_texture: &Texture, stone_texture: &Texture,
                            hit_object: u8, hit_cube: Option<&Cube>) -> Color {
    
    let mut total_r: f32 = 0.0;
    let mut total_g: f32 = 0.0;
    let mut total_b: f32 = 0.0;
    
    // SOLO CUBOS Y SUELO
    let base_color = if hit_object == 1 && material.has_texture && hit_cube.is_some() {
        let cube = hit_cube.unwrap();
        let (u, v) = cube.get_uv_coordinates(hit_point);
        
        match material.material_type {
            MaterialType::Grass => grass_texture.sample(u, v),
            MaterialType::Dirt => dirt_texture.sample(u, v),
            MaterialType::Stone => stone_texture.sample(u, v),
            _ => material.diffuse,
        }
    } else {
        material.diffuse
    };
    
    // Luz ambiental normal
    let ambient_strength = match material.material_type {
        MaterialType::Grass => 0.5,
        MaterialType::Stone => 0.25,
        MaterialType::Dirt => 0.35,
        _ => 0.3,
    };
    
    total_r += base_color.r as f32 * ambient_strength;
    total_g += base_color.g as f32 * ambient_strength;
    total_b += base_color.b as f32 * ambient_strength;
    
    // Iluminación directa simplificada
    for (i, light) in lights.iter().enumerate() {
        let light_dir = nalgebra_glm::normalize(&(light.position - hit_point));
        let light_distance = nalgebra_glm::distance(&light.position, hit_point);
        
        let mut in_shadow = false;
        if i == 0 && hit_object == 1 {
            let shadow_ray_origin = hit_point + normal * 0.001;
            in_shadow = diorama.ray_intersect_shadow_fast(&shadow_ray_origin, &light_dir, light_distance);
        }
        
        if !in_shadow {
            let diff = nalgebra_glm::dot(normal, &light_dir).max(0.0);
            let attenuation = 1.0 / (1.0 + 0.015 * light_distance + 0.0008 * light_distance * light_distance);
            
            let surface_multiplier = match material.material_type {
                MaterialType::Grass => 1.4,
                MaterialType::Stone => 0.8,
                MaterialType::Dirt => 1.0,
                _ => 1.0,
            };
            
            let light_contribution = diff * light.intensity * attenuation * surface_multiplier;
            
            total_r += base_color.r as f32 * light.color.r as f32 / 255.0 * light_contribution;
            total_g += base_color.g as f32 * light.color.g as f32 / 255.0 * light_contribution;
            total_b += base_color.b as f32 * light.color.b as f32 / 255.0 * light_contribution;
        }
    }
    
    Color::new(
        total_r.min(255.0) as u8, 
        total_g.min(255.0) as u8, 
        total_b.min(255.0) as u8
    )
}