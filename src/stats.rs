#[derive(Debug, Default)]
pub struct RenderStats {
    pub rays_cast: u32,
    pub hits: u32,
    pub misses: u32,
    pub objects_tested: u32,
}

impl RenderStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    pub fn print_summary(&self) {
        println!("=== Render Stats ===");
        println!("Rays cast: {}", self.rays_cast);
        println!("Hits: {}", self.hits);
        println!("Misses: {}", self.misses);
        println!("Hit rate: {:.1}%", 
            if self.rays_cast > 0 { 
                (self.hits as f32 / self.rays_cast as f32) * 100.0 
            } else { 
                0.0 
            }
        );
        println!("Objects tested: {}", self.objects_tested);
        println!("==================");
    }
}