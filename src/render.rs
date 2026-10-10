use crate::{
    boids::{Bird, Flock, TRAIL_LENGTH},
    interaction::{SceneTool, SceneTools},
    metrics::SimulationMetrics,
    mouse::MouseField,
    observation::Observation,
    obstacles::Obstacles,
    settings::{Appearance, Playback, SimulationSettings},
};
use bevy::{
    gizmos::config::{GizmoConfigGroup, GizmoConfigStore},
    prelude::*,
};
use std::time::Instant;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct Trails;

pub fn configure_trails(appearance: Res<Appearance>, mut configs: ResMut<GizmoConfigStore>) {
    if appearance.is_changed() {
        configs.config_mut::<Trails>().0.line.width = appearance.trail_width;
    }
}

fn trail_points(
    bird: &Bird,
    head: usize,
    samples: usize,
    just_recorded: bool,
    alpha: f32,
) -> impl Iterator<Item = Vec2> + '_ {
    let skip = usize::from(just_recorded && alpha < 1.0);
    let start = (head + TRAIL_LENGTH + 1 - samples - skip) % TRAIL_LENGTH;
    (0..samples)
        .map(move |age| bird.trail[(start + age) % TRAIL_LENGTH])
        .chain(std::iter::once(
            bird.previous_position.lerp(bird.position, alpha),
        ))
}

#[expect(
    clippy::too_many_arguments,
    reason = "Rendering reads simulation, appearance and fixed time and writes gizmos and CPU timing"
)]
pub fn draw_flock(
    flock: Res<Flock>,
    time: Res<Time<Fixed>>,
    mouse: Res<MouseField>,
    settings: Res<SimulationSettings>,
    appearance: Res<Appearance>,
    observation: Res<Observation>,
    playback: Res<Playback>,
    mut metrics: ResMut<SimulationMetrics>,
    mut trails: Gizmos<Trails>,
    mut birds: Gizmos,
) {
    let start = Instant::now();
    let alpha = if playback.paused {
        1.0
    } else {
        time.overstep_fraction()
    };
    let flight_time = (flock.elapsed - (1.0 - alpha) / 60.0).max(0.0);
    let samples = (appearance.trail_seconds * 30.0).ceil() as usize;
    let color = Color::srgb(0.32, 0.68, 0.72).to_linear();
    let trail_colors: [Color; TRAIL_LENGTH] = std::array::from_fn(|age| {
        let opacity = (age as f32 / samples.max(1) as f32).powi(2) * appearance.trail_opacity;
        Color::linear_rgba(color.red, color.green, color.blue, opacity)
    });
    let ink = Color::LinearRgba(Color::srgb(0.79, 0.92, 0.94).to_linear());
    for bird in &flock.birds {
        let position = bird.previous_position.lerp(bird.position, alpha);
        if samples > 0 && appearance.trail_opacity > 0.0 {
            trails.linestrip_gradient_2d(
                trail_points(
                    bird,
                    flock.trail_head,
                    samples,
                    flock.ticks.is_multiple_of(2),
                    alpha,
                )
                .enumerate()
                .map(|(age, point)| (point, trail_colors[age])),
            );
        }
        let forward = bird
            .previous_velocity
            .lerp(bird.velocity, alpha)
            .normalize_or_zero();
        let flap = if appearance.animate_wings {
            (flight_time * 9.0 + bird.wing_phase).sin()
        } else {
            0.0
        };
        let wing = forward.perp() * (3.8 + flap * 0.75) * appearance.bird_size;
        let sweep = forward * (3.0 + flap * 0.5) * appearance.bird_size;
        birds.linestrip_2d(
            [
                position - sweep + wing,
                position + forward * 5.0 * appearance.bird_size,
                position - sweep - wing,
            ],
            if observation.color_alarm {
                let alarm = bird.alarm.clamp(0.0, 1.0);
                Color::srgb(
                    0.79 + 0.19 * alarm,
                    0.92 - 0.43 * alarm,
                    0.94 - 0.58 * alarm,
                )
            } else {
                ink
            },
        );
    }
    if appearance.show_influence
        && let Some(cursor) = mouse.cursor
    {
        let color = if mouse.repelling {
            Color::srgb(0.98, 0.49, 0.36)
        } else {
            Color::srgb(0.38, 0.85, 0.78)
        };
        birds.circle_2d(cursor, 4.0, color.with_alpha(0.8));
        trails.circle_2d(
            cursor,
            if mouse.repelling {
                settings.predator_radius
            } else {
                42.0
            },
            color.with_alpha(0.28),
        );
        if mouse.repelling && mouse.velocity != Vec2::ZERO {
            let prediction = (mouse.velocity * settings.prediction)
                .clamp_length_max(settings.predator_radius * 2.0);
            trails.line_2d(cursor, cursor + prediction, color.with_alpha(0.35));
        }
    }
    metrics.draw_ms += (start.elapsed().as_secs_f64() * 1000.0 - metrics.draw_ms) * 0.08;
}

#[expect(
    clippy::too_many_arguments,
    reason = "Scene overlays use selection, simulation and playback resources"
)]
pub fn draw_environment(
    time: Res<Time<Fixed>>,
    flock: Res<Flock>,
    settings: Res<SimulationSettings>,
    playback: Res<Playback>,
    tools: Res<SceneTools>,
    obstacles: Res<Obstacles>,
    observation: Res<Observation>,
    mut gizmos: Gizmos,
) {
    let alpha = if playback.paused {
        1.0
    } else {
        time.overstep_fraction()
    };
    for (index, circle) in obstacles.circles.iter().enumerate() {
        let color = if tools.tool == SceneTool::Obstacles && obstacles.selected == Some(index) {
            Color::srgb(0.98, 0.7, 0.34)
        } else {
            Color::srgb(0.36, 0.42, 0.51)
        };
        gizmos.circle_2d(circle.center, circle.radius, color);
        gizmos.circle_2d(
            circle.center,
            circle.radius + crate::obstacles::BIRD_CLEARANCE,
            color.with_alpha(0.15),
        );
    }
    if let (Some(index), Some(details)) = (observation.selected, &observation.details) {
        let bird = &flock.birds[index];
        let position = bird.previous_position.lerp(bird.position, alpha);
        let cyan = Color::srgb(0.38, 0.85, 0.78);
        gizmos.circle_2d(position, 9.0, cyan);
        gizmos.circle_2d(position, settings.neighbor_radius, cyan.with_alpha(0.3));
        gizmos.circle_2d(
            position,
            settings.separation_distance,
            Color::srgb(0.97, 0.38, 0.46).with_alpha(0.3),
        );
        for &other in &details.neighbors {
            let neighbor = &flock.birds[other];
            let neighbor_position = neighbor.previous_position.lerp(neighbor.position, alpha);
            gizmos.circle_2d(neighbor_position, 6.0, cyan.with_alpha(0.7));
            gizmos.line_2d(position, neighbor_position, cyan.with_alpha(0.12));
        }
        for (force, color) in [
            (details.rules.separation, Color::srgb(0.97, 0.38, 0.46)),
            (details.rules.alignment, Color::srgb(0.35, 0.65, 1.0)),
            (details.rules.cohesion, Color::srgb(0.5, 0.95, 0.5)),
            (details.escape, Color::srgb(1.0, 0.67, 0.23)),
        ] {
            if force.length_squared() > 0.01 {
                gizmos.arrow_2d(
                    position,
                    position + (force * 0.18).clamp_length_max(100.0),
                    color,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sampled_trail_never_runs_ahead_of_interpolated_bird() {
        let mut flock = crate::boids::Flock::default();
        flock.set_count(
            1,
            Vec2::new(640.0, 400.0),
            SimulationSettings::default().cruise_speed,
        );
        let bird = &mut flock.birds[0];
        bird.trail.fill(Vec2::ZERO);
        bird.trail[0] = Vec2::X;
        bird.trail[1] = Vec2::X * 2.0;
        bird.previous_position = Vec2::X;
        bird.position = Vec2::X * 2.0;
        let points: Vec<_> = trail_points(bird, 1, 3, true, 0.5).collect();
        assert_eq!(points.last().unwrap(), &Vec2::new(1.5, 0.0));
        assert!(points.iter().all(|point| point.x <= 1.5));
    }
}
