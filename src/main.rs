mod boids;
mod grid;
mod input;
mod interaction;
mod metrics;
mod mouse;
mod observation;
mod obstacles;
mod preferences;
mod presets;
mod profiling;
mod render;
mod runtime;
mod settings;
mod ui;

use crate::{
    boids::Flock,
    metrics::SimulationMetrics,
    mouse::MouseField,
    render::Trails,
    settings::{Appearance, Field, Playback, SimulationSettings},
};
use bevy::{
    app::RunFixedMainLoopSystems,
    diagnostic::FrameTimeDiagnosticsPlugin,
    gizmos::config::{GizmoConfig, GizmoLineConfig},
    prelude::*,
    window::WindowResizeConstraints,
};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

fn main() {
    if std::env::args().any(|argument| argument == "--profile") {
        profiling::run();
        return;
    }
    let mut graphics_profile = profiling::GraphicsProfile::default();
    graphics_profile.enabled = std::env::args().any(|argument| argument == "--profile-graphics");
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Murmuration — Boids".into(),
                    resolution: (1280, 800).into(),
                    resize_constraints: WindowResizeConstraints {
                        min_width: 640.0,
                        min_height: 420.0,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            EguiPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(0.018, 0.032, 0.049)))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<Flock>()
        .init_resource::<MouseField>()
        .init_resource::<Field>()
        .init_resource::<SimulationSettings>()
        .init_resource::<Appearance>()
        .init_resource::<Playback>()
        .init_resource::<observation::Observation>()
        .init_resource::<obstacles::Obstacles>()
        .init_resource::<interaction::SceneTools>()
        .init_resource::<preferences::PreferenceFile>()
        .init_resource::<SimulationMetrics>()
        .insert_resource(graphics_profile)
        .insert_gizmo_config(
            Trails,
            GizmoConfig {
                line: GizmoLineConfig {
                    width: 1.0,
                    ..default()
                },
                ..default()
            },
        )
        .add_systems(Startup, (preferences::restore, runtime::setup).chain())
        .add_systems(
            RunFixedMainLoop,
            (
                input::keyboard_controls,
                runtime::apply_controls,
                input::sample_mouse,
                profiling::prepare_graphics,
            )
                .chain()
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(FixedUpdate, runtime::simulate)
        .add_systems(
            Last,
            (metrics::record_frame, profiling::collect_graphics).chain(),
        )
        .add_systems(Update, (render::configure_trails, render::draw_flock))
        .add_systems(
            Update,
            (observation::update, render::draw_environment).chain(),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (ui::panel, preferences::apply, runtime::apply_controls).chain(),
        )
        .run();
}
