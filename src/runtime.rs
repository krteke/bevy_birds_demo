use crate::{
    boids::Flock,
    metrics::SimulationMetrics,
    mouse::MouseField,
    observation::Observation,
    obstacles::Obstacles,
    settings::{Appearance, Field, Playback, SimulationSettings},
};
use bevy::prelude::*;

pub fn setup(
    mut commands: Commands,
    mut flock: ResMut<Flock>,
    settings: Res<SimulationSettings>,
    field: Res<Field>,
) {
    commands.spawn(Camera2d);
    flock.set_count(settings.count, field.half_size, settings.cruise_speed);
}

#[expect(
    clippy::too_many_arguments,
    reason = "Controls reconcile simulation, population, input and selection resources"
)]
pub fn apply_controls(
    mut settings: ResMut<SimulationSettings>,
    mut appearance: ResMut<Appearance>,
    mut playback: ResMut<Playback>,
    field: Res<Field>,
    mut flock: ResMut<Flock>,
    mut mouse: ResMut<MouseField>,
    mut observation: ResMut<Observation>,
    obstacles: Res<Obstacles>,
) {
    if playback.defaults_requested {
        *settings = SimulationSettings::default();
        *appearance = Appearance::default();
        playback.defaults_requested = false;
        mouse.update(None, false, 0.0);
    }
    if playback.reset_requested {
        flock.reset(settings.count, field.half_size, settings.cruise_speed);
        flock.exclude_obstacles(&obstacles.circles, field.half_size);
        observation.selected = None;
        observation.details = None;
        playback.reset_requested = false;
        mouse.update(None, false, 0.0);
    } else if flock.birds.len() != settings.count {
        flock.set_count(settings.count, field.half_size, settings.cruise_speed);
        flock.exclude_obstacles(&obstacles.circles, field.half_size);
    }
    if !settings.mouse_enabled || playback.paused {
        mouse.update(None, false, 0.0);
    }
    if !settings.mouse_enabled {
        for bird in &mut flock.birds {
            bird.alarm = 0.0;
            bird.escape = Vec2::ZERO;
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Fixed simulation reads field, controls and obstacles and writes flock, input and metrics"
)]
pub fn simulate(
    time: Res<Time<Fixed>>,
    settings: Res<SimulationSettings>,
    playback: Res<Playback>,
    field: Res<Field>,
    mut flock: ResMut<Flock>,
    mut mouse: ResMut<MouseField>,
    mut metrics: ResMut<SimulationMetrics>,
    obstacles: Res<Obstacles>,
) {
    if playback.paused {
        return;
    }
    let timing = flock.step(
        time.delta_secs(),
        field.half_size,
        &mut mouse,
        &settings,
        &obstacles.circles,
    );
    metrics.record(timing);
}

#[cfg(test)]
mod tests;
