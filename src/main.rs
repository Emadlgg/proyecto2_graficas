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
    
    pub fn create_stone_texture() -> Self {
        let size = 64;
        let mut data = Vec::with_capacity((size * size * 3) as usize);
        for y in 0..size {
            for x in 0..size {
                let noise = ((x * 31 + y * 11) % 50) as f32 / 50.0;
                let gray = 105 + (noise * 30.0) as u8;
                data.extend_from_slice(&[gray, gray, gray]);
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

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        BoundingBox { min, max }
    }
    
    pub fn contains_point(&self, point: &Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    
    pub fn intersects_ray(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> bool {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        
        for i in 0..3 {
            let ray_dir_component = ray_direction[i];
            let ray_origin_component = ray_origin[i];
            let box_min = self.min[i];
            let box_max = self.max[i];
            
            if ray_dir_component.abs() < 1e-6 {
                if ray_origin_component < box_min || ray_origin_component > box_max {
                    return false;
                }
            } else {
                let t1 = (box_min - ray_origin_component) / ray_dir_component;
                let t2 = (box_max - ray_origin_component) / ray_dir_component;
                let t_near = t1.min(t2);
                let t_far = t1.max(t2);
                t_min = t_min.max(t_near);
                t_max = t_max.min(t_far);
                if t_min > t_max { return false; }
            }
        }
        t_max > 0.0
    }
}

pub struct Octree {
    pub bounds: BoundingBox,
    pub cubes: Vec<usize>,
    pub children: Option<Box<[Octree; 8]>>,
    pub max_depth: u32,
    pub current_depth: u32,
}

impl Octree {
    pub fn new(bounds: BoundingBox, max_depth: u32) -> Self {
        Octree {
            bounds,
            cubes: Vec::new(),
            children: None,
            max_depth,
            current_depth: 0,
        }
    }
    
    fn subdivide_with_cubes(&mut self, cubes: &[Cube], center_cube: &Cube) {
        if self.current_depth >= self.max_depth || self.cubes.len() <= 4 {
            return;
        }
        
        let center = Vec3::new(
            (self.bounds.min.x + self.bounds.max.x) * 0.5,
            (self.bounds.min.y + self.bounds.max.y) * 0.5,
            (self.bounds.min.z + self.bounds.max.z) * 0.5,
        );
        
        let mut children = Box::new([
            Octree::new_with_depth(BoundingBox::new(self.bounds.min, center), self.max_depth, self.current_depth + 1),
            Octree::new_with_depth(BoundingBox::new(Vec3::new(center.x, self.bounds.min.y, self.bounds.min.z), Vec3::new(self.bounds.max.x, center.y, center.z)), self.max_depth, self.current_depth + 1),
            Octree::new_with_depth(BoundingBox::new(Vec3::new(self.bounds.min.x, center.y, self.bounds.min.z), Vec3::new(center.x, self.bounds.max.y, center.z)), self.max_depth, self.current_depth + 1),
            Octree::new_with_depth(BoundingBox::new(Vec3::new(center.x, center.y, self.bounds.min.z), Vec3::new(self.bounds.max.x, self.bounds.max.y, center.z)), self.max_depth, self.current_depth + 1),
            Octree::new_with_depth(BoundingBox::new(Vec3::new(self.bounds.min.x, self.bounds.min.y, center.z), Vec3::new(center.x, center.y, self.bounds.max.z)), self.max_depth, self.current_depth + 1),
            Octree::new_with_depth(BoundingBox::new(Vec3::new(center.x, self.bounds.min.y, center.z), Vec3::new(self.bounds.max.x, center.y, self.bounds.max.z)), self.max_depth, self.current_depth + 1),
            Octree::new_with_depth(BoundingBox::new(Vec3::new(self.bounds.min.x, center.y, center.z), Vec3::new(center.x, self.bounds.max.y, self.bounds.max.z)), self.max_depth, self.current_depth + 1),
            Octree::new_with_depth(BoundingBox::new(center, self.bounds.max), self.max_depth, self.current_depth + 1),
        ]);
        
        for &cube_index in &self.cubes {
            if cube_index == cubes.len() {
                let cube_center = (center_cube.min + center_cube.max) * 0.5;
                for child in children.iter_mut() {
                    if child.bounds.contains_point(&cube_center) {
                        child.cubes.push(cube_index);
                        break;
                    }
                }
            } else {
                let cube = &cubes[cube_index];
                let cube_center = (cube.min + cube.max) * 0.5;
                for child in children.iter_mut() {
                    if child.bounds.contains_point(&cube_center) {
                        child.cubes.push(cube_index);
                        break;
                    }
                }
            }
        }
        
        for child in children.iter_mut() {
            child.subdivide_with_cubes(cubes, center_cube);
        }
        
        self.children = Some(children);
        self.cubes.clear();
    }
    
    fn new_with_depth(bounds: BoundingBox, max_depth: u32, current_depth: u32) -> Self {
        Octree { bounds, cubes: Vec::new(), children: None, max_depth, current_depth }
    }
    
    pub fn query_ray(&self, ray_origin: &Vec3, ray_direction: &Vec3, results: &mut Vec<usize>) {
        if !self.bounds.intersects_ray(ray_origin, ray_direction) {
            return;
        }
        
        if self.children.is_none() {
            results.extend(&self.cubes);
            return;
        }
        
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_ray(ray_origin, ray_direction, results);
            }
        }
    }
}

pub struct CubeGrid {
    pub cubes: Vec<Cube>,
    pub size: usize,
    pub octree: Octree,
    pub center_cube: Cube,
}

impl CubeGrid {
    pub fn new(center: Vec3, cube_size: f32) -> Self {
        let size = 5;
        let mut cubes = Vec::new();
        let grid_size = size as f32 * cube_size;
        let offset = grid_size / 2.0 - cube_size / 2.0;
        
        for y in 0..size {
            for z in 0..size {
                for x in 0..size {
                    let pos = Vec3::new(
                        center.x + x as f32 * cube_size - offset,
                        center.y - y as f32 * cube_size,
                        center.z + z as f32 * cube_size - offset,
                    );
                    
                    let material = match y {
                        0 => Material::grass_top(),
                        1..=2 => Material::dirt_layer(),
                        3..=4 => Material::stone_layer(),
                        _ => Material::stone_layer(),
                    };
                    
                    cubes.push(Cube::new(pos, cube_size, material));
                }
            }
        }
        
        let center_cube = Cube::new(
            Vec3::new(center.x, center.y + cube_size, center.z),
            cube_size * 2.0,
            Material::center_block()
        );
        
        let bounds = BoundingBox::new(
            center - Vec3::new(grid_size * 0.6, grid_size * 0.6, grid_size * 0.6),
            center + Vec3::new(grid_size * 0.6, grid_size * 0.6, grid_size * 0.6),
        );
        
        let mut octree = Octree::new(bounds, 3);
        
        for i in 0..cubes.len() {
            octree.cubes.push(i);
        }
        octree.cubes.push(cubes.len());
        
        let grid = CubeGrid { cubes, size, octree, center_cube };
        Self::build_octree_for_grid(grid)
    }
    
    fn build_octree_for_grid(mut grid: CubeGrid) -> CubeGrid {
        grid.octree.subdivide_with_cubes(&grid.cubes, &grid.center_cube);
        grid
    }
    
    pub fn ray_intersect_optimized(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<(usize, f32, bool)> {
        let mut candidate_cubes = Vec::new();
        self.octree.query_ray(ray_origin, ray_direction, &mut candidate_cubes);
        
        if candidate_cubes.is_empty() {
            for i in 0..self.cubes.len() {
                candidate_cubes.push(i);
            }
            candidate_cubes.push(self.cubes.len());
        }
        
        let mut closest_distance = f32::INFINITY;
        let mut closest_index = None;
        let mut is_center_cube = false;
        
        for &cube_index in &candidate_cubes {
            let (distance, is_center) = if cube_index == self.cubes.len() {
                (self.center_cube.ray_intersect(ray_origin, ray_direction), true)
            } else {
                (self.cubes[cube_index].ray_intersect(ray_origin, ray_direction), false)
            };
            
            if let Some(distance) = distance {
                if distance > 0.001 && distance < closest_distance {
                    closest_distance = distance;
                    closest_index = Some(cube_index);
                    is_center_cube = is_center;
                }
            }
        }
        
        closest_index.map(|idx| (idx, closest_distance, is_center_cube))
    }
    
    pub fn ray_intersect_optimized_shadow(&self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        let mut candidate_cubes = Vec::new();
        self.octree.query_ray(ray_origin, ray_direction, &mut candidate_cubes);
        
        for &cube_index in &candidate_cubes {
            let distance = if cube_index == self.cubes.len() {
                self.center_cube.ray_intersect(ray_origin, ray_direction)
            } else {
                self.cubes[cube_index].ray_intersect(ray_origin, ray_direction)
            };
            
            if let Some(distance) = distance {
                if distance > 0.001 && distance < max_distance {
                    return true;
                }
            }
        }
        false
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    
    println!("Loading textures...");
    
    let grass_texture = match Texture::load_from_file("grass.png") {
        Ok(tex) => { println!("Grass texture loaded: {}x{}", tex.width, tex.height); tex }
        Err(_) => { println!("Could not load grass.png, using procedural grass texture"); Texture::create_grass_texture() }
    };
    
    let dirt_texture = match Texture::load_from_file("dirt.png") {
        Ok(tex) => { println!("Dirt texture loaded: {}x{}", tex.width, tex.height); tex }
        Err(_) => { println!("Could not load dirt.png, using procedural dirt texture"); Texture::create_dirt_texture() }
    };
    
    let stone_texture = match Texture::load_from_file("stone.png") {
        Ok(tex) => { println!("Stone texture loaded: {}x{}", tex.width, tex.height); tex }
        Err(_) => { println!("Could not load stone.png, using procedural stone texture"); Texture::create_stone_texture() }
    };
    
    let center_texture = match Texture::load_from_file("center.png") {
        Ok(tex) => { println!("Center texture loaded: {}x{}", tex.width, tex.height); tex }
        Err(_) => { println!("Could not load center.png, using procedural center texture"); Texture::create_special_texture() }
    };
    
    let mut camera = OrbitCamera::new(Vec3::new(0.0, 0.0, -5.0), 8.0);
    camera.orbit(0.7, 0.3);
    
    println!("Building octree...");
    let cube_grid = CubeGrid::new(Vec3::new(0.0, 0.0, -5.0), 0.8);
    println!("Octree built successfully!");
    
    let floor = Plane::new(Vec3::new(0.0, -4.0, -5.0), Vec3::new(0.0, 1.0, 0.0), Material::stone_wall());
    
    let lights = vec![
        Light::new(Vec3::new(-2.0, 4.0, -3.0), Color::new(255, 200, 150), 1.2),
        Light::new(Vec3::new(3.0, 3.0, -4.0), Color::new(200, 220, 255), 0.8),
        Light::new(Vec3::new(0.0, 6.0, -2.0), Color::new(100, 120, 150), 0.4),
    ];
    
    let mut window = Window::new("5x5x5 Grid + Center Cube", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.set_target_fps(60);
    
    println!("\n=== CONTROLS ===");
    println!("Arrow Keys: Orbit around grid");
    println!("W/S: Zoom in/out");
    println!("Escape: Exit");
    println!("================\n");
    println!("Grid: 5x5x5 cubes = {} total", cube_grid.cubes.len());
    println!("Center cube: 2x2 added");
    
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
        render(&mut framebuffer, &cube_grid, &floor, &lights, &camera, &grass_texture, &dirt_texture, &stone_texture, &center_texture, &mut stats);
        
        if frame_count % 120 == 0 {
            println!("\n--- Frame {} ---", frame_count);
            println!("5x5x5 Grid + Center Cube rendering");
            println!("Camera: distance {:.1}, yaw {:.2}, pitch {:.2}", camera.distance, camera.yaw, camera.pitch);
            stats.print_summary();
        }
        frame_count += 1;
        
        window.update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn render(framebuffer: &mut Framebuffer, cube_grid: &CubeGrid, floor: &Plane, lights: &[Light], camera: &OrbitCamera, grass_texture: &Texture, dirt_texture: &Texture, stone_texture: &Texture, center_texture: &Texture, stats: &mut RenderStats) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;
            let screen_x = screen_x * aspect_ratio;
            
            let ray_direction = camera.get_ray_direction(screen_x, screen_y);
            let pixel_color = cast_ray_optimized(&camera.eye, &ray_direction, cube_grid, floor, lights, grass_texture, dirt_texture, stone_texture, center_texture, stats);
            
            framebuffer.set_current_color(pixel_color);
            framebuffer.point(x, y);
        }
    }
}

fn cast_ray_optimized(ray_origin: &Vec3, ray_direction: &Vec3, cube_grid: &CubeGrid, floor: &Plane, lights: &[Light], grass_texture: &Texture, dirt_texture: &Texture, stone_texture: &Texture, center_texture: &Texture, stats: &mut RenderStats) -> Color {
    let background_color = Color::new(135, 206, 235);
    let mut closest_distance = f32::INFINITY;
    let mut hit_material: Option<&Material> = None;
    let mut hit_point = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_normal = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_object = 0;
    let mut hit_cube: Option<&Cube> = None;
    let mut cube_layer = 0;
    let mut is_center_cube = false;
    
    stats.rays_cast += 1;
    
    if let Some((cube_index, distance, is_center)) = cube_grid.ray_intersect_optimized(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance {
            closest_distance = distance;
            
            if is_center {
                hit_material = Some(&cube_grid.center_cube.material);
                hit_point = ray_origin + ray_direction * distance;
                hit_normal = cube_grid.center_cube.get_normal(&hit_point);
                hit_cube = Some(&cube_grid.center_cube);
                is_center_cube = true;
            } else {
                let cube = &cube_grid.cubes[cube_index];
                hit_material = Some(&cube.material);
                hit_point = ray_origin + ray_direction * distance;
                hit_normal = cube.get_normal(&hit_point);
                hit_cube = Some(cube);
                cube_layer = cube_index / 25;
            }
            
            hit_object = 1;
            stats.hits += 1;
        }
    }
    
    stats.objects_tested += 1;
    if let Some(distance) = floor.ray_intersect(ray_origin, ray_direction) {
        if distance > 0.001 && distance < closest_distance {
            hit_material = Some(&floor.material);
            hit_point = ray_origin + ray_direction * distance;
            hit_normal = floor.get_normal(&hit_point);
            hit_object = 2;
            hit_cube = None;
            stats.hits += 1;
        }
    }
    
    if hit_object == 0 {
        stats.misses += 1;
        return background_color;
    }
    
    if let Some(material) = hit_material {
        calculate_lighting_optimized(&hit_point, &hit_normal, material, lights, cube_grid, floor, grass_texture, dirt_texture, stone_texture, center_texture, hit_object, hit_cube, cube_layer, is_center_cube)
    } else {
        background_color
    }
}

fn calculate_lighting_optimized(hit_point: &Vec3, normal: &Vec3, material: &Material, lights: &[Light], cube_grid: &CubeGrid, floor: &Plane, grass_texture: &Texture, dirt_texture: &Texture, stone_texture: &Texture, center_texture: &Texture, hit_object: u8, hit_cube: Option<&Cube>, cube_layer: usize, is_center_cube: bool) -> Color {
    let mut total_r: f32 = 0.0;
    let mut total_g: f32 = 0.0;
    let mut total_b: f32 = 0.0;
    
    let base_color = if hit_object == 1 && material.has_texture {
        if let Some(cube) = hit_cube {
            let (u, v) = cube.get_uv_coordinates(hit_point);
            if is_center_cube {
                center_texture.sample(u, v)
            } else {
                match cube_layer {
                    0 => grass_texture.sample(u, v),
                    1..=2 => dirt_texture.sample(u, v),
                    3..=4 => stone_texture.sample(u, v),
                    _ => stone_texture.sample(u, v),
                }
            }
        } else {
            material.diffuse
        }
    } else {
        material.diffuse
    };
    
    let ambient_strength = 0.2;
    total_r += base_color.r as f32 * ambient_strength;
    total_g += base_color.g as f32 * ambient_strength;
    total_b += base_color.b as f32 * ambient_strength;
    
    for (i, light) in lights.iter().enumerate() {
        let light_dir = nalgebra_glm::normalize(&(light.position - hit_point));
        let light_distance = nalgebra_glm::distance(&light.position, hit_point);
        
        let mut in_shadow = false;
        
        if i == 0 {
            let shadow_ray_origin = hit_point + normal * 0.001;
            in_shadow = cube_grid.ray_intersect_optimized_shadow(&shadow_ray_origin, &light_dir, light_distance);
            
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
            let light_contribution = diff * light.intensity * attenuation;
            
            total_r += base_color.r as f32 * light.color.r as f32 / 255.0 * light_contribution;
            total_g += base_color.g as f32 * light.color.g as f32 / 255.0 * light_contribution;
            total_b += base_color.b as f32 * light.color.b as f32 / 255.0 * light_contribution;
        }
    }
    
    Color::new(total_r.min(255.0) as u8, total_g.min(255.0) as u8, total_b.min(255.0) as u8)
}