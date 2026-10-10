use crate::{
    grid::SpatialGrid,
    metrics::StepTiming,
    mouse::MouseField,
    obstacles::{BIRD_CLEARANCE, Obstacle},
    settings::SimulationSettings,
};
use bevy::{
    prelude::*,
    tasks::{ComputeTaskPool, ParallelSliceMut},
};
use std::time::Instant;

pub const TRAIL_LENGTH: usize = 64;
#[cfg(test)]
const NEIGHBOR_RADIUS: f32 = 64.0;
#[cfg(test)]
const MAX_SPEED: f32 = 180.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Bird {
    pub position: Vec2,
    pub previous_position: Vec2,
    pub velocity: Vec2,
    pub previous_velocity: Vec2,
    pub alarm: f32,
    pub escape: Vec2,
    pub individuality: f32,
    pub wing_phase: f32,
    pub trail: [Vec2; TRAIL_LENGTH],
}

impl Bird {
    fn new(position: Vec2, velocity: Vec2) -> Self {
        let phase = position.dot(Vec2::new(0.13, 0.27));
        Self {
            position,
            previous_position: position,
            velocity,
            previous_velocity: velocity,
            alarm: 0.0,
            escape: Vec2::ZERO,
            individuality: phase.sin(),
            wing_phase: phase,
            trail: [position; TRAIL_LENGTH],
        }
    }
}

#[derive(Default)]
pub struct RuleForces {
    pub separation: Vec2,
    pub alignment: Vec2,
    pub cohesion: Vec2,
    pub neighbor_count: usize,
}

#[derive(Default)]
pub struct BirdInspection {
    pub rules: RuleForces,
    pub escape: Vec2,
    pub neighbors: Vec<usize>,
}

#[derive(Clone, Copy, Default)]
struct FlightForces {
    steering: Vec2,
    threat: Vec2,
}

#[derive(Default, Resource)]
pub struct Flock {
    pub birds: Vec<Bird>,
    pub trail_head: usize,
    pub ticks: usize,
    pub elapsed: f32,
    grid: SpatialGrid,
    forces: Vec<FlightForces>,
    next_bird: usize,
}

impl Flock {
    pub fn set_count(&mut self, count: usize, half_size: Vec2, cruise_speed: f32) {
        self.birds.truncate(count);
        while self.birds.len() < count {
            // Irrational strides make a reproducible distribution without storing a random generator.
            let seed = self.next_bird as f32 + 1.0;
            let position = Vec2::new((seed * 0.754_877_7).fract(), (seed * 0.569_840_3).fract());
            let position = (position * 2.0 - Vec2::ONE) * (half_size - Vec2::splat(70.0));
            self.birds.push(Bird::new(
                position,
                Vec2::from_angle(seed * 2.399_963_1) * cruise_speed,
            ));
            self.next_bird += 1;
        }
    }

    pub fn reset(&mut self, count: usize, half_size: Vec2, cruise_speed: f32) {
        *self = Self::default();
        self.set_count(count, half_size, cruise_speed);
    }

    pub fn resize_field(&mut self, half_size: Vec2, obstacles: &[Obstacle]) {
        let inside = half_size - Vec2::splat(12.0);
        for bird in &mut self.birds {
            let position = bird.position.clamp(-inside, inside);
            if position != bird.position {
                bird.position = position;
                bird.previous_position = position;
                bird.trail.fill(position);
                bird.velocity = -position.normalize_or_zero() * bird.velocity.length();
                bird.previous_velocity = bird.velocity;
            }
        }
        self.exclude_obstacles(obstacles, half_size);
    }

    pub fn exclude_obstacles(&mut self, obstacles: &[Obstacle], bounds: Vec2) {
        for (index, bird) in self.birds.iter_mut().enumerate() {
            for obstacle in obstacles {
                let away = bird.position - obstacle.center;
                let radius = obstacle.radius + BIRD_CLEARANCE + 0.01;
                if away.length_squared() < radius * radius {
                    let direction = if away == Vec2::ZERO {
                        Vec2::from_angle(index as f32 * 2.399_963_1)
                    } else {
                        away.normalize()
                    };
                    let position = bird.position;
                    bird.position = obstacle.center + direction * radius;
                    if bird.position.abs().cmpgt(bounds).any() {
                        // When an old obstacle extends past a resized field, escape toward the field interior.
                        let inward = -position.normalize();
                        let toward = obstacle.center - position;
                        let forward = toward.dot(inward);
                        let exit = forward
                            + (radius * radius - toward.length_squared() + forward * forward)
                                .sqrt();
                        bird.position = position + inward * exit;
                    }
                    bird.previous_position = bird.position;
                    bird.trail.fill(bird.position);
                    bird.velocity =
                        (bird.position - obstacle.center).normalize() * bird.velocity.length();
                    bird.previous_velocity = bird.velocity;
                }
            }
        }
    }

    pub fn step(
        &mut self,
        dt: f32,
        half_size: Vec2,
        mouse: &mut MouseField,
        settings: &SimulationSettings,
        obstacles: &[Obstacle],
    ) -> StepTiming {
        let start = Instant::now();
        mouse.advance(dt);
        self.grid.rebuild(&self.birds, settings.neighbor_radius);
        let center =
            self.birds.iter().map(|bird| bird.position).sum::<Vec2>() / self.birds.len() as f32;
        let grid_ms = start.elapsed().as_secs_f64() * 1000.0;
        let mut forces = std::mem::take(&mut self.forces);
        forces.resize(self.birds.len(), FlightForces::default());
        let pool = ComputeTaskPool::get();
        let chunk_size = self.birds.len().div_ceil(pool.thread_num()).max(256);
        forces.par_chunk_map_mut(pool, chunk_size, |chunk_index, chunk| {
            for (offset, force) in chunk.iter_mut().enumerate() {
                *force = self.flight_forces(
                    chunk_index * chunk_size + offset,
                    center,
                    half_size,
                    mouse,
                    settings,
                    obstacles,
                );
            }
        });
        self.forces = forces;
        let steering_ms = start.elapsed().as_secs_f64() * 1000.0 - grid_ms;
        self.ticks += 1;
        self.elapsed += dt;
        let record_trail = self.ticks.is_multiple_of(2);
        if record_trail {
            self.trail_head = (self.trail_head + 1) % TRAIL_LENGTH;
        }
        for (index, (bird, force)) in self.birds.iter_mut().zip(&self.forces).enumerate() {
            bird.previous_position = bird.position;
            bird.previous_velocity = bird.velocity;
            if settings.mouse_enabled && mouse.repelling && mouse.cursor.is_some() {
                if force.threat != Vec2::ZERO {
                    bird.escape = force.threat.normalize();
                    bird.alarm = settings.alarm_duration;
                } else {
                    bird.alarm = (bird.alarm - dt).max(0.0);
                }
            } else {
                bird.alarm = 0.0;
                bird.escape = Vec2::ZERO;
            }
            let heading = bird.velocity.normalize_or_zero();
            let desired = (heading * settings.cruise_speed + force.steering).normalize_or_zero();
            let speed = bird.velocity.length();
            let turn_limit = (settings.turn_rate.to_radians() * dt).min(510.0 * dt / speed);
            let turn = heading.angle_to(desired).clamp(-turn_limit, turn_limit);
            let heading = Rot2::radians(turn) * heading;
            let cruise =
                settings.cruise_speed * (1.0 + bird.individuality * settings.individuality);
            let mut goal = cruise * if bird.alarm > 0.0 { 1.12 } else { 1.0 };
            for axis in 0..2 {
                if heading[axis] * bird.position[axis] > 0.0 {
                    let clearance = (half_size[axis] - bird.position[axis].abs() - 12.0).max(0.0);
                    let safe_speed = (clearance * settings.turn_rate.to_radians() * 0.7
                        / heading[axis].abs())
                    .max(1.0);
                    goal = goal.min(safe_speed);
                }
            }
            for obstacle in obstacles {
                if let Some(limit) =
                    obstacle.speed_limit(bird.position, heading, settings.turn_rate.to_radians())
                {
                    goal = goal.min(limit);
                }
            }
            let speed_change =
                ((goal - speed) * (1.0 - (-3.0 * dt).exp())).clamp(-510.0 * dt, 510.0 * dt);
            let speed = (speed + speed_change).min(settings.cruise_speed * (4.0 / 3.0));
            bird.velocity = heading * speed;
            bird.position += bird.velocity * dt;
            for obstacle in obstacles {
                (bird.position, bird.velocity) = obstacle.constrain_motion(
                    bird.previous_position,
                    bird.position,
                    bird.velocity,
                    index,
                );
            }
            if record_trail {
                bird.trail[self.trail_head] = bird.position;
            }
        }
        mouse.segments.clear();
        StepTiming {
            grid: grid_ms,
            steering: steering_ms,
            motion: start.elapsed().as_secs_f64() * 1000.0 - grid_ms - steering_ms,
        }
    }

    pub fn inspect(
        &self,
        index: usize,
        mouse: &MouseField,
        settings: &SimulationSettings,
        grid: &SpatialGrid,
    ) -> BirdInspection {
        let mut neighbors = Vec::new();
        let rules = self.flock_rules(index, settings, grid, Some(&mut neighbors));
        let threat = mouse.threat_at(self.birds[index].position, index, settings);
        BirdInspection {
            rules,
            escape: self.escape_force(index, threat, mouse, settings),
            neighbors,
        }
    }

    fn flock_rules(
        &self,
        index: usize,
        settings: &SimulationSettings,
        grid: &SpatialGrid,
        mut observed: Option<&mut Vec<usize>>,
    ) -> RuleForces {
        let bird = &self.birds[index];
        let mut separation = Vec2::ZERO;
        let mut headings = Vec2::ZERO;
        let mut positions = Vec2::ZERO;
        let mut neighbors = 0;
        let mut nearest = [(f32::INFINITY, 0usize); 16];
        let mut nearest_count = 0;
        grid.for_each_neighbor(&self.birds, index, |other, offset, distance_squared| {
            if distance_squared < settings.separation_distance * settings.separation_distance {
                let distance = distance_squared.sqrt();
                separation += if distance > 0.0 {
                    offset / distance * (1.0 - distance / settings.separation_distance)
                } else {
                    let direction = Vec2::from_angle(
                        (index.min(other) * 31 + index.max(other)) as f32 * 2.399_963_1,
                    );
                    if index < other { direction } else { -direction }
                };
            }
            if settings.limit_neighbors {
                let slot = nearest[..nearest_count]
                    .partition_point(|&(distance, _)| distance <= distance_squared);
                if slot < settings.neighbor_limit {
                    nearest_count = (nearest_count + 1).min(settings.neighbor_limit);
                    nearest.copy_within(slot..nearest_count - 1, slot + 1);
                    nearest[slot] = (distance_squared, other);
                }
            } else {
                headings += self.birds[other].velocity;
                positions += self.birds[other].position;
                neighbors += 1;
                if let Some(indices) = observed.as_mut() {
                    indices.push(other);
                }
            }
        });
        if settings.limit_neighbors {
            for &(_, other) in &nearest[..nearest_count] {
                headings += self.birds[other].velocity;
                positions += self.birds[other].position;
                if let Some(indices) = observed.as_mut() {
                    indices.push(other);
                }
            }
            neighbors = nearest_count;
        }
        let mut alignment = Vec2::ZERO;
        let mut cohesion = Vec2::ZERO;
        if neighbors > 0 {
            alignment = (headings.normalize_or_zero() * settings.cruise_speed - bird.velocity)
                * settings.alignment_weight;
            let toward = positions / neighbors as f32 - bird.position;
            cohesion = (toward.normalize_or_zero() * settings.cruise_speed - bird.velocity)
                * settings.cohesion_weight
                * (toward.length() / settings.neighbor_radius);
        }
        RuleForces {
            separation: (separation * 420.0).clamp_length_max(480.0) * settings.separation_weight,
            alignment,
            cohesion,
            neighbor_count: neighbors,
        }
    }

    fn escape_force(
        &self,
        index: usize,
        threat: Vec2,
        mouse: &MouseField,
        settings: &SimulationSettings,
    ) -> Vec2 {
        let bird = &self.birds[index];
        if threat != Vec2::ZERO {
            return threat;
        }
        if settings.mouse_enabled
            && mouse.repelling
            && mouse.cursor.is_some()
            && bird.alarm > 0.0
            && settings.alarm_duration > 0.0
        {
            bird.escape
                * settings.predator_strength
                * 0.3
                * (bird.alarm / settings.alarm_duration).min(1.0)
        } else {
            Vec2::ZERO
        }
    }

    fn flight_forces(
        &self,
        index: usize,
        center: Vec2,
        half_size: Vec2,
        mouse: &MouseField,
        settings: &SimulationSettings,
        obstacles: &[Obstacle],
    ) -> FlightForces {
        let bird = &self.birds[index];
        let rules = self.flock_rules(index, settings, &self.grid, None);
        let mut steering = rules.separation + rules.alignment + rules.cohesion;
        let to_center = center - bird.position;
        // Connected birds need less global pull; isolated groups retain a route back to the flock.
        let regroup = if rules.neighbor_count == 0 { 1.0 } else { 0.45 };
        steering += (to_center.normalize_or_zero() * settings.cruise_speed - bird.velocity)
            * settings.regroup_weight
            * regroup
            * (to_center.length() / 140.0).min(1.0);
        steering += settings.wind * 3.0;
        steering = steering.clamp_length_max(260.0);
        let threat = mouse.threat_at(bird.position, index, settings);
        steering += self.escape_force(index, threat, mouse, settings);
        if settings.mouse_enabled
            && !mouse.repelling
            && let Some(cursor) = mouse.cursor
        {
            let toward = cursor - bird.position;
            steering += toward.normalize_or_zero()
                * settings.attraction_strength
                * (toward.length() / 140.0).min(1.0);
        }
        let turn_radius = settings.cruise_speed * (4.0 / 3.0) / settings.turn_rate.to_radians();
        let margin = (turn_radius * 1.8).max(95.0);
        let turn_at = (half_size - Vec2::splat(margin)).max(half_size * 0.25);
        let ahead = bird.position + bird.velocity * 0.35;
        let edge = (ahead.abs() - turn_at).max(Vec2::ZERO);
        steering -= ahead.signum() * edge * 24.0;
        for obstacle in obstacles {
            steering += obstacle.avoidance(
                bird.position,
                bird.velocity,
                settings.turn_rate.to_radians(),
                index,
            );
        }
        FlightForces {
            steering: steering.clamp_length_max(850.0),
            threat,
        }
    }
}

#[cfg(test)]
mod tests;
