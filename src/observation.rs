use crate::{
    boids::{BirdInspection, Flock},
    grid::SpatialGrid,
    interaction::{SceneTool, SceneTools},
    mouse::MouseField,
    settings::SimulationSettings,
};
use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct Observation {
    pub selected: Option<usize>,
    pub details: Option<BirdInspection>,
    pub color_alarm: bool,
    pub show_metrics: bool,
    pub alignment: f32,
    pub mean_nearest: Option<f32>,
    pub measurement_ms: f64,
    pub(super) elapsed: f32,
    pub(super) grid: SpatialGrid,
}

impl Observation {
    pub fn measure(&mut self, flock: &Flock) {
        let start = std::time::Instant::now();
        self.grid.rebuild(&flock.birds, 64.0);
        self.alignment = (flock
            .birds
            .iter()
            .map(|bird| bird.velocity.normalize_or_zero())
            .sum::<Vec2>()
            / flock.birds.len() as f32)
            .length();
        let (sum, count) = (0..flock.birds.len())
            .filter_map(|index| self.grid.nearest_distance(&flock.birds, index))
            .fold((0.0, 0usize), |(sum, count), distance| {
                (sum + distance, count + 1)
            });
        self.mean_nearest = (count > 0).then(|| sum / count as f32);
        self.measurement_ms = start.elapsed().as_secs_f64() * 1000.0;
    }
}

pub fn update(
    time: Res<Time>,
    flock: Res<Flock>,
    settings: Res<SimulationSettings>,
    mouse: Res<MouseField>,
    tools: Res<SceneTools>,
    mut observation: ResMut<Observation>,
) {
    observation.elapsed += time.delta_secs();
    if observation.show_metrics && observation.elapsed >= 0.5 {
        observation.elapsed = 0.0;
        observation.measure(&flock);
    }
    if observation
        .selected
        .is_some_and(|index| index >= flock.birds.len())
    {
        observation.selected = None;
    }
    observation.details = if tools.tool != SceneTool::Obstacles
        && let Some(index) = observation.selected
    {
        observation
            .grid
            .rebuild(&flock.birds, settings.neighbor_radius);
        Some(flock.inspect(index, &mouse, &settings, &observation.grid))
    } else {
        None
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn measurements_include_isolated_birds_and_opposite_headings() {
        let mut flock = Flock::default();
        flock.set_count(2, Vec2::new(640.0, 400.0), 135.0);
        flock.birds[0].position = Vec2::ZERO;
        flock.birds[0].velocity = Vec2::X * 100.0;
        flock.birds[1].position = Vec2::new(300.0, 0.0);
        flock.birds[1].velocity = -Vec2::X * 200.0;
        let mut observation = Observation::default();
        observation.measure(&flock);
        assert_eq!(observation.alignment, 0.0);
        assert_eq!(observation.mean_nearest, Some(300.0));
    }
}
