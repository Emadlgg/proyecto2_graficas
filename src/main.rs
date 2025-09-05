mod framebuffer;
mod color;
mod sphere;
mod cube;
mod camera;
mod material;
mod ray_intersect;
mod stats;

use framebuffer::Framebuffer;
use color::Color;
use sphere::Sphere;
use cube::Cube;
use camera::OrbitCamera;
use material::Material;
use stats::RenderStats;
use nalgebra_glm::Vec3;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn test_intersections(spheres: &[Sphere], cubes: &[Cube]) {
    println!("\n=== Testing object intersections ===");
    let test_ray_origin = Vec3::new(0.0, 0.0, 0.0);
    let test_ray_direction = Vec3::new(0.0, 0.0, -1.0);
    
    for (i, sphere) in spheres.iter().enumerate() {
        if let Some(distance) = sphere.ray_intersect(&test_ray_origin, &test_ray_direction) {
            println!("Sphere {}: HIT at distance {:.2}", i, distance);
        } else {
            println!("Sphere {}: MISS", i);
        }
    }
    
    for (i, cube) in cubes.iter().enumerate() {
        if let Some(distance) = cube.ray_intersect(&test_ray_origin, &test_ray_direction) {
            println!("Cube {}: HIT at distance {:.2}", i, distance);
        } else {
            println!("Cube {}: MISS", i);
        }
    }
    println!("====================================\n");
}

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    
    // Crear cámara orbital centrada en la escena del laboratorio
    let mut camera = OrbitCamera::new(
        Vec3::new(0.0, -0.5, -6.0), // Target: centro del laboratorio
        8.0 // Distancia inicial
    );
    
    // Crear escena del laboratorio de alquimia
    let spheres = vec![
        // Frasco redondo grande (esfera de vidrio)
        Sphere::new(Vec3::new(-1.0, 0.0, -5.0), 0.6, Material::clear_glass()),
        
        // Cristal mágico (esfera emisiva)
        Sphere::new(Vec3::new(0.0, 0.5, -4.5), 0.3, Material::magic_crystal()),
        
        // Poción pequeña (esfera pequeña)
        Sphere::new(Vec3::new(0.8, -0.7, -4.8), 0.2, Material::rusty_metal()),
    ];
    
    let cubes = vec![
        // Mesa del laboratorio (cubo grande, madera)
        Cube::new(Vec3::new(0.0, -1.5, -6.0), 3.0, Material::old_wood()),
        
        // Libro/grimorio (cubo pequeño)
        Cube::new(Vec3::new(1.5, -0.8, -5.5), 0.4, Material::old_wood()),
        
        // Caldero (cubo metálico)
        Cube::new(Vec3::new(-1.8, -0.5, -7.0), 0.8, Material::rusty_metal()),
        
        // Pared trasera (cubo grande de piedra)
        Cube::new(Vec3::new(0.0, 1.0, -10.0), 6.0, Material::stone_wall()),
    ];
    
    // Crear ventana
    let mut window = Window::new(
        "Alchemy Lab Raytracer - Orbital Camera",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });
    
    window.set_target_fps(60);
    
    // Test inicial
    test_intersections(&spheres, &cubes);
    
    println!("\n=== CONTROLS ===");
    println!("Arrow Keys: Orbit around scene");
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
        
        render(&mut framebuffer, &spheres, &cubes, &camera, &mut stats);
        
        if frame_count % 120 == 0 {
            println!("\n--- Frame {} ---", frame_count);
            println!("Spheres: {}, Cubes: {}", spheres.len(), cubes.len());
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

fn render(framebuffer: &mut Framebuffer, spheres: &[Sphere], cubes: &[Cube], camera: &OrbitCamera, stats: &mut RenderStats) {
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
            let pixel_color = cast_ray(&camera.eye, &ray_direction, spheres, cubes, stats);
            
            framebuffer.set_current_color(pixel_color);
            framebuffer.point(x, y);
        }
    }
}

fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, spheres: &[Sphere], cubes: &[Cube], stats: &mut RenderStats) -> Color {
    let mut closest_distance = f32::INFINITY;
    let mut hit_material: Option<&Material> = None;
    let background_color = Color::new(15, 25, 35);
    
    stats.rays_cast += 1;
    
    // Probar intersección con esferas
    for sphere in spheres.iter() {
        stats.objects_tested += 1;
        
        if let Some(distance) = sphere.ray_intersect(ray_origin, ray_direction) 
            && distance < closest_distance 
            && distance > 0.001 
        {
            closest_distance = distance;
            hit_material = Some(&sphere.material);
        }
    }
    
    // Probar intersección con cubos
    for cube in cubes.iter() {
        stats.objects_tested += 1;
        
        if let Some(distance) = cube.ray_intersect(ray_origin, ray_direction) 
            && distance < closest_distance 
            && distance > 0.001 
        {
            closest_distance = distance;
            hit_material = Some(&cube.material);
        }
    }
    
    if let Some(material) = hit_material {
        stats.hits += 1;
        material.diffuse
    } else {
        stats.misses += 1;
        background_color
    }
}