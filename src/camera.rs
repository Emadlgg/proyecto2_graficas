use nalgebra_glm::Vec3;
use std::f32::consts::PI;

pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub eye: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub forward: Vec3,
}

impl OrbitCamera {
    pub fn new(target: Vec3, distance: f32) -> Self {
        let mut camera = OrbitCamera {
            target,
            distance,
            yaw: 0.0,
            pitch: 0.0,
            eye: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            forward: Vec3::new(0.0, 0.0, -1.0),
        };
        camera.update();
        camera
    }
    
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch += delta_pitch;
        
        self.pitch = self.pitch.clamp(-PI * 0.49, PI * 0.49);
        self.yaw = self.yaw % (2.0 * PI);
        
        self.update();
    }
    
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta).max(1.0).min(20.0);
        self.update();
    }
    
    fn update(&mut self) {
        let x = self.distance * self.pitch.cos() * self.yaw.cos();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.sin();
        
        self.eye = self.target + Vec3::new(x, y, z);
        
        self.forward = nalgebra_glm::normalize(&(self.target - self.eye));
        self.right = nalgebra_glm::normalize(&nalgebra_glm::cross(&self.forward, &Vec3::new(0.0, 1.0, 0.0)));
        self.up = nalgebra_glm::cross(&self.right, &self.forward);
    }
    
    pub fn get_ray_direction(&self, screen_x: f32, screen_y: f32) -> Vec3 {
        let direction = screen_x * self.right + screen_y * self.up + self.forward;
        nalgebra_glm::normalize(&direction)
    }
}