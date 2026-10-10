use super::*;
use bevy::ecs::system::RunSystemOnce;

fn world() -> World {
    bevy::tasks::ComputeTaskPool::get_or_init(|| {
        bevy::tasks::TaskPoolBuilder::new().num_threads(4).build()
    });
    let mut world = World::new();
    world.init_resource::<SimulationSettings>();
    world.init_resource::<Appearance>();
    world.init_resource::<Playback>();
    world.init_resource::<Field>();
    world.init_resource::<Flock>();
    world.init_resource::<MouseField>();
    world.init_resource::<Observation>();
    world.init_resource::<Obstacles>();
    world.init_resource::<SimulationMetrics>();
    let mut time = Time::<Fixed>::from_hz(60.0);
    time.advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
    world.insert_resource(time);
    world.resource_mut::<Flock>().set_count(
        1000,
        Vec2::new(640.0, 400.0),
        SimulationSettings::default().cruise_speed,
    );
    world
}

#[test]
fn pausing_preserves_positions_and_trails() {
    let mut world = world();
    let bird = world.resource::<Flock>().birds[0].clone();
    world.resource_mut::<Playback>().paused = true;
    world.run_system_once(simulate).unwrap();
    assert_eq!(world.resource::<Flock>().birds[0], bird);
}

#[test]
fn population_control_preserves_surviving_birds() {
    let mut world = world();
    let bird = world.resource::<Flock>().birds[0].clone();
    world.resource_mut::<SimulationSettings>().count = 300;
    world.run_system_once(apply_controls).unwrap();
    assert_eq!(world.resource::<Flock>().birds.len(), 300);
    assert_eq!(world.resource::<Flock>().birds[0], bird);
}

#[test]
fn reset_restarts_flight_without_changing_settings_or_pause() {
    let mut world = world();
    world.run_system_once(simulate).unwrap();
    let moved = world.resource::<Flock>().birds[0].clone();
    world.resource_mut::<SimulationSettings>().cruise_speed = 170.0;
    let settings = world.resource::<SimulationSettings>().clone();
    world.resource_mut::<Playback>().paused = true;
    world.resource_mut::<Playback>().reset_requested = true;
    world.run_system_once(apply_controls).unwrap();
    assert_ne!(world.resource::<Flock>().birds[0], moved);
    assert!(*world.resource::<SimulationSettings>() == settings);
    assert!(world.resource::<Playback>().paused);
    assert!(!world.resource::<Playback>().reset_requested);
}

#[test]
fn restoring_defaults_restores_mouse_and_visual_controls() {
    let mut world = world();
    world.resource_mut::<SimulationSettings>().mouse_enabled = false;
    world.resource_mut::<SimulationSettings>().count = 5000;
    world.resource_mut::<Appearance>().trail_seconds = 0.0;
    world.resource_mut::<Playback>().defaults_requested = true;
    world.run_system_once(apply_controls).unwrap();
    assert!(*world.resource::<SimulationSettings>() == SimulationSettings::default());
    assert_eq!(world.resource::<Appearance>().trail_seconds, 0.4);
    assert_eq!(world.resource::<Flock>().birds.len(), 1000);
}

#[test]
fn pausing_keeps_the_escape_response_frozen() {
    let mut world = world();
    world.resource_mut::<Flock>().birds[0].alarm = 0.4;
    world.resource_mut::<Flock>().birds[0].escape = Vec2::Y;
    world.resource_mut::<Playback>().paused = true;
    world.run_system_once(apply_controls).unwrap();
    assert_eq!(world.resource::<Flock>().birds[0].alarm, 0.4);
    assert_eq!(world.resource::<Flock>().birds[0].escape, Vec2::Y);
}

#[test]
fn reset_spawns_birds_at_the_selected_cruise_speed() {
    let mut world = world();
    world.resource_mut::<SimulationSettings>().cruise_speed = 60.0;
    world.resource_mut::<Playback>().reset_requested = true;
    world.run_system_once(apply_controls).unwrap();
    for bird in &world.resource::<Flock>().birds {
        assert!((bird.velocity.length() - 60.0).abs() < 0.001);
    }
}

#[test]
fn adding_birds_uses_current_speed_without_changing_survivors() {
    let mut world = world();
    let survivor = world.resource::<Flock>().birds[0].clone();
    world.resource_mut::<SimulationSettings>().cruise_speed = 60.0;
    world.resource_mut::<SimulationSettings>().count = 1100;
    world.run_system_once(apply_controls).unwrap();
    let birds = &world.resource::<Flock>().birds;
    assert_eq!(birds[0], survivor);
    assert!((birds[1000].velocity.length() - 60.0).abs() < 0.001);
}
