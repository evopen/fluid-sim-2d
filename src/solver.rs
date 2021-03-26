use once_cell::sync::Lazy;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::f32::consts::PI;

use glam::Vec2;

use bytemuck::{Pod, Zeroable};

use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

const VIEW_WIDTH: f32 = WINDOW_WIDTH as f32 * 1.5;
const VIEW_HEIGHT: f32 = WINDOW_HEIGHT as f32 * 1.5;
const REST_DENS: f32 = 2000.0;
const H: f32 = 25.0;
const MASS: f32 = 65.0;
const GAS_CONST: f32 = 4000.0;
const SPIKY_GRAD: Lazy<f32> = Lazy::new(|| -45.0 / (std::f32::consts::PI * H.powi(6)));
const VISC_LAP: Lazy<f32> = Lazy::new(|| 45.0 / (std::f32::consts::PI * H.powi(6)));
const VISC: f32 = 250.0;
const G: Vec2 = glam::const_vec2!([0.0, 12000.0 * -9.8]);
const DT: f32 = 0.0004;
const SIGMA_3: f32 = 8.0 / (PI * H * H * H);
const BOUNDRY_DENSITY: f32 = H / 4.0;
const BOUNDRY_DEPTH: usize = 3;

#[repr(C)]
#[derive(Debug, Default, Zeroable, Pod, Clone, Copy)]
pub struct Particle {
    pos: Vec2,
    v: Vec2,
    f: Vec2,
    rho: f32,
    p: f32,
    is_dynamic: u32,
}

impl Particle {
    pub fn new(pos: Vec2, is_dynamic: u32) -> Self {
        Self {
            pos,
            is_dynamic,
            ..Default::default()
        }
    }
}

pub struct Solver {
    pub particles: Vec<Particle>,
    rng: rand::rngs::SmallRng,
}

fn cubic_spline(r: f32) -> f32 {
    let q = (1.0 / H) * r;
    if q.clamp(0.0, 0.5) == q {
        return SIGMA_3 * (6.0 * (q.powi(3) - q.powi(2)) + 1.0);
    } else if q.clamp(0.5, 1.0) == q {
        return SIGMA_3 * (2.0 * (1.0 - q).powi(3));
    } else {
        return 0.0;
    }
}

fn check_pos_identical(particles: &[Particle]) {
    for (i, p1) in particles.iter().enumerate() {
        for (j, p2) in particles.iter().enumerate() {
            if i == j {
                continue;
            }
            assert!(p1.pos != p2.pos);
        }
    }
}

impl Solver {
    pub fn new(count: u32) -> Self {
        let mut particles = Vec::with_capacity(count as usize);

        for y in std::iter::successors(Some(H), |y| Some(y + H - 0.01))
            .take_while(|y| y < &(VIEW_HEIGHT - 2.0 * H))
        {
            for x in std::iter::successors(Some(VIEW_WIDTH / 4.0), |x| Some(x + H - 0.01))
                .take_while(|x| x < &(VIEW_WIDTH / 2.0))
            {
                if particles.len() >= count as usize {
                    break;
                }
                particles.push(Particle::new(Vec2::new(x, y), 1));
            }
        }
        println!("initialize dam break with {} particles", particles.len());

        for y in std::iter::successors(Some(0.0), |y| Some(y - BOUNDRY_DENSITY)).take(BOUNDRY_DEPTH)
        {
            for x in std::iter::successors(Some(0.0), |x| Some(x + BOUNDRY_DENSITY))
                .take_while(|x| x < &VIEW_WIDTH)
            {
                particles.push(Particle::new(Vec2::new(x, y), 0));
            }
        }
        for y in std::iter::successors(Some(0.0), |y| Some(y + BOUNDRY_DENSITY))
            .take_while(|y| y < &VIEW_HEIGHT)
        {
            for x in std::iter::successors(Some(VIEW_WIDTH), |x| Some(x + BOUNDRY_DENSITY))
                .take(BOUNDRY_DEPTH)
            {
                particles.push(Particle::new(Vec2::new(x, y), 0));
            }
        }
        for y in std::iter::successors(Some(VIEW_HEIGHT), |y| Some(y + BOUNDRY_DENSITY))
            .take(BOUNDRY_DEPTH)
        {
            for x in std::iter::successors(Some(0.0), |x| Some(x + BOUNDRY_DENSITY))
                .take_while(|x| x < &VIEW_WIDTH)
            {
                particles.push(Particle::new(Vec2::new(x, y), 0));
            }
        }
        for y in std::iter::successors(Some(0.0), |y| Some(y + BOUNDRY_DENSITY))
            .take_while(|y| y < &VIEW_HEIGHT)
        {
            for x in
                std::iter::successors(Some(0.0), |x| Some(x - BOUNDRY_DENSITY)).take(BOUNDRY_DEPTH)
            {
                particles.push(Particle::new(Vec2::new(x, y), 0));
            }
        }

        // pipe
        let pipe_width = 15;
        let density = BOUNDRY_DENSITY;
        let depth = BOUNDRY_DEPTH;
        for y in std::iter::successors(Some(VIEW_HEIGHT / 4.0 * 3.0), |y| Some(y + density))
            .take(pipe_width)
        {
            for x in
                std::iter::successors(Some(VIEW_WIDTH / 4.0), |x| Some(x - density)).take(depth)
            {
                particles.push(Particle::new(Vec2::new(x, y), 0));
            }
        }
        for y in
            std::iter::successors(Some(VIEW_HEIGHT / 4.0 * 3.0), |y| Some(y - density)).take(depth)
        {
            for x in std::iter::successors(Some(VIEW_WIDTH / 4.0), |x| Some(x + density)).take(100)
            {
                particles.push(Particle::new(Vec2::new(x, y), 0));
            }
        }
        for y in std::iter::successors(
            Some(VIEW_HEIGHT / 4.0 * 3.0 + density * pipe_width as f32),
            |y| Some(y + density),
        )
        .take(depth)
        {
            for x in std::iter::successors(Some(VIEW_WIDTH / 4.0), |x| Some(x + density)).take(100)
            {
                particles.push(Particle::new(Vec2::new(x, y), 0));
            }
        }

        Self {
            particles,
            rng: SmallRng::from_entropy(),
        }
    }

    fn compute_density_pressure(&mut self) {
        let particles_cache = self.particles.clone();
        self.particles.par_iter_mut().for_each(|pi| {
            pi.rho = 0.0;
            for pj in &particles_cache {
                let r = pi.pos.distance(pj.pos);
                if r < H {
                    pi.rho += MASS * cubic_spline(r);
                }
            }
            pi.p = GAS_CONST * (pi.rho - REST_DENS);

            #[cfg(debug_assertions)]
            assert!(!pi.p.is_nan());
            #[cfg(debug_assertions)]
            assert!(!pi.rho.is_nan());
        });
    }

    fn compute_forces(&mut self) {
        let particles_cache = self.particles.clone();
        self.particles
            .par_iter_mut()
            .enumerate()
            .filter(|(i, p)| p.is_dynamic == 1)
            .for_each(|(i, pi)| {
                let mut fpress = Vec2::new(0.0, 0.0);
                let mut fvisc = Vec2::new(0.0, 0.0);

                for (j, pj) in particles_cache.iter().enumerate() {
                    if j.eq(&i) {
                        continue;
                    }
                    let rij: Vec2 = pj.pos - pi.pos;

                    let r = pi.pos.distance(pj.pos);

                    if r < H {
                        fpress += -rij.normalize() * MASS * (pi.p + pj.p) / (2.0 * pj.rho)
                            * *SPIKY_GRAD
                            * (H - r).powi(2);

                        fvisc += VISC * MASS * (pj.v - pi.v) / pj.rho * *VISC_LAP * (H - r);

                        if cfg!(debug_assertions) {
                            if fpress.is_nan() {
                                dbg!(&rij.normalize());
                                dbg!(&pi.p);
                                dbg!(&pj.p);
                                dbg!(&pi.pos);
                                dbg!(&pj.pos);
                                dbg!(&pj.rho);
                            }

                            assert!(!pi.p.is_nan());
                            assert!(!pj.p.is_nan());
                            assert!(!pj.rho.is_nan());
                            assert!(!r.is_nan());
                            assert!(!fpress.is_nan());
                            assert!(!fvisc.is_nan());
                        }
                    }
                }

                let fgrav = G * pi.rho;

                #[cfg(debug_assertions)]
                assert!(!fgrav.is_nan());

                pi.f = fgrav + fpress + fvisc;
            });
    }

    pub fn integrate(&mut self) {
        self.particles
            .par_iter_mut()
            .filter(|p| p.is_dynamic == 1)
            .for_each(|p| {
                p.v += DT * p.f / p.rho;
                p.pos += DT * p.v;
            });
    }

    pub fn update(&mut self) {
        self.compute_density_pressure();
        self.compute_forces();
        self.integrate();
        self.particles.push(Particle::new(
            Vec2::new(
                VIEW_WIDTH / 4.0 + self.rng.gen_range(10.0..200.0),
                VIEW_HEIGHT / 4.0 * 3.0 + self.rng.gen_range(20.0..100.0),
            ),
            1,
        ));
    }
}
