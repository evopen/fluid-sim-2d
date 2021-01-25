use std::iter::successors;

use glam::Vec2;
use glam::Vec3A as Vec3;

use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

const VIEW_WIDTH: f32 = WINDOW_WIDTH as f32 * 1.5;
const VIEW_HEIGHT: f32 = WINDOW_HEIGHT as f32 * 1.5;

#[derive(Debug, Default)]
struct Particle {
    pos: Vec2,
    v: Vec2,
    f: Vec2,
    rho: f32,
    p: f32,
}

impl Particle {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }
}

pub struct Solver {
    particles: Vec<Particle>,
}

impl Solver {
    pub fn new(count: u32, radius: f32) -> Self {
        let mut particles = Vec::with_capacity(count as usize);
        println!("initialize dam break with {} particles", count);

        for y in std::iter::successors(Some(radius), |y| Some(y + radius))
            .take_while(|y| y < &(VIEW_HEIGHT - radius * 2.0))
        {
            for x in std::iter::successors(Some(VIEW_WIDTH / 4.0), |x| Some(x + radius))
                .take_while(|x| x < &(VIEW_WIDTH / 2.0))
            {
                particles.push(Particle::new(Vec2::new(x, y)));
            }
        }

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
