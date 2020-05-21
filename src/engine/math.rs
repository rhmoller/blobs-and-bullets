#[derive(Clone)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&mut self) {
        let length = self.length();
        self.x /= length;
        self.y /= length;
    }

    pub fn add(&mut self, v: &Vec2) {
        self.x += v.x;
        self.y += v.y;
    }

    pub fn scale(&mut self, f: f64) {
        self.x *= f;
        self.y *= f;
    }
}

pub fn vec2_distance(v1: &Vec2, v2: &Vec2) -> f64 {
    Vec2::new(v2.x - v1.x, v2.y - v1.y).length()
}
