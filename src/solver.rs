use glam::Vec2;
use glam::Vec3A as Vec3;

struct Particle {
    pos: Vec2,
    v: Vec2,
    f: Vec2,
    rho: f32,
    p: f32,
}

impl Particle {
    pub fn new() -> Self {}
}

struct Solver {
    particles: Vec<Particle>,
}

impl Solver {
    pub fn iterate() {}

    pub fn new() -> Self {
        let particles = Vec::new();

        Self { particles }
    }

    pub fn integrate(&self) {}

    fn compute_density_pressure(&self) {}

    fn compute_forces(&self) {}

    pub fn update(&self) {
        self.compute_density_pressure();
        self.compute_forces();
        self.integrate();
    }
}
