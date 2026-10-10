use crate::{
    boids::Flock,
    metrics::{SimulationMetrics, summarize_frame_times},
    mouse::MouseField,
    obstacles::Obstacle,
    settings::{Appearance, Field, SimulationSettings},
};
use bevy::prelude::*;

pub fn run() {
    App::new().add_plugins(TaskPoolPlugin::default());
    println!("birds,scene,view_radius,grid_ms,steering_ms,motion_ms,total_ms,p95_ms");
    for count in [1000, 3000, 5000] {
        for scene in [
            "free",
            "falcon",
            "attraction",
            "obstacles",
            "dense-attraction",
        ] {
            if scene == "dense-attraction" && count != 5000 {
                continue;
            }
            let settings = SimulationSettings {
                count,
                neighbor_radius: if scene == "dense-attraction" {
                    140.0
                } else {
                    64.0
                },
                ..default()
            };
            let field = Vec2::new(640.0, 400.0);
            let mut flock = Flock::default();
            flock.set_count(count, field, settings.cruise_speed);
            let obstacles = if scene == "obstacles" {
                vec![
                    Obstacle {
                        center: Vec2::ZERO,
                        radius: 55.0,
                    },
                    Obstacle {
                        center: Vec2::new(200.0, 100.0),
                        radius: 45.0,
                    },
                    Obstacle {
                        center: Vec2::new(-200.0, -100.0),
                        radius: 45.0,
                    },
                ]
            } else {
                Vec::new()
            };
            flock.exclude_obstacles(&obstacles, field);
            let mut mouse = MouseField::default();
            let mut metrics = SimulationMetrics::default();
            for tick in 0..480 {
                let t = tick as f32 / 60.0;
                match scene {
                    "falcon" => mouse.update(
                        Some(Vec2::new((t * 2.2).sin() * 450.0, (t * 3.1).cos() * 250.0)),
                        true,
                        1.0 / 60.0,
                    ),
                    "attraction" | "dense-attraction" => {
                        mouse.update(Some(Vec2::ZERO), false, 1.0 / 60.0)
                    }
                    _ => {}
                }
                let timing = flock.step(1.0 / 60.0, field, &mut mouse, &settings, &obstacles);
                if tick >= 180 {
                    metrics.record(timing);
                }
            }
            let summary = metrics.summary();
            let mean = summary.mean;
            println!(
                "{count},{scene},{:.0},{:.3},{:.3},{:.3},{:.3},{:.3}",
                settings.neighbor_radius,
                mean.grid,
                mean.steering,
                mean.motion,
                mean.total(),
                summary.p95
            );
        }
    }
}

#[derive(Default, Resource)]
pub struct GraphicsProfile {
    pub enabled: bool,
    phase: usize,
    elapsed: f64,
    samples: Vec<f64>,
}

pub fn prepare_graphics(
    profile: Res<GraphicsProfile>,
    mut settings: ResMut<SimulationSettings>,
    mut appearance: ResMut<Appearance>,
    mut mouse: ResMut<MouseField>,
    time: Res<Time>,
    field: Res<Field>,
    mut flock: ResMut<Flock>,
) {
    if !profile.enabled {
        return;
    }
    if profile.elapsed == 0.0 || field.is_changed() {
        flock.reset(5000, field.half_size, 135.0);
    }
    *settings = SimulationSettings {
        count: 5000,
        neighbor_radius: 140.0,
        ..default()
    };
    *appearance = Appearance {
        trail_seconds: if profile.phase == 0 { 2.0 } else { 0.0 },
        ..default()
    };
    mouse.update(Some(Vec2::ZERO), false, time.delta_secs());
}

pub fn collect_graphics(
    time: Res<Time<Real>>,
    mut profile: ResMut<GraphicsProfile>,
    metrics: Res<SimulationMetrics>,
    mut exit: MessageWriter<AppExit>,
    field: Res<Field>,
) {
    if !profile.enabled {
        return;
    }
    profile.elapsed += time.delta_secs_f64();
    if profile.elapsed > 3.0 {
        profile.samples.push(time.delta_secs_f64() * 1000.0);
    }
    if profile.elapsed >= 6.0 {
        let (mean, p95) = summarize_frame_times(&mut profile.samples);
        let sim = metrics.summary();
        println!(
            "graphics,birds=5000,view=140,scene=attraction,width={:.0},height={:.0},trail={},frames={},frame_mean_ms={mean:.3},frame_p95_ms={p95:.3},sim_mean_ms={:.3},cpu_draw_smoothed_ms={:.3}",
            field.half_size.x * 2.0,
            field.half_size.y * 2.0,
            if profile.phase == 0 { 2 } else { 0 },
            profile.samples.len(),
            sim.mean.total(),
            metrics.draw_ms
        );
        profile.phase += 1;
        profile.elapsed = 0.0;
        profile.samples.clear();
        if profile.phase == 2 {
            exit.write(AppExit::Success);
        }
    }
}
