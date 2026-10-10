use super::*;
use std::path::PathBuf;

fn test_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("bevy-birds-{}-{name}.json", std::process::id()))
}

#[test]
fn saved_settings_restore_all_simulation_and_appearance_fields() {
    let path = test_path("round-trip");
    let document = SettingsDocument {
        simulation: SimulationSettings {
            cruise_speed: 60.0,
            wind: bevy::prelude::Vec2::new(10.0, -5.0),
            mouse_enabled: false,
            ..Default::default()
        },
        appearance: Appearance {
            trail_seconds: 1.5,
            bird_size: 1.8,
            ..Default::default()
        },
    };
    document.write(&path).unwrap();
    let loaded = SettingsDocument::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(loaded.simulation, document.simulation);
    assert_eq!(loaded.appearance, document.appearance);
}

#[test]
fn loading_rejects_neighbor_limits_that_exceed_the_search_buffer() {
    let path = test_path("invalid-neighbors");
    let document = SettingsDocument {
        simulation: SimulationSettings {
            neighbor_limit: 17,
            ..Default::default()
        },
        appearance: Appearance::default(),
    };
    std::fs::write(&path, serde_json::to_string(&document).unwrap()).unwrap();
    let loaded = SettingsDocument::read(&path);
    std::fs::remove_file(&path).unwrap();
    assert!(loaded.is_err());
}

#[test]
fn loading_rejects_trails_longer_than_the_history_buffer() {
    let path = test_path("invalid-trail");
    let document = SettingsDocument {
        simulation: SimulationSettings::default(),
        appearance: Appearance {
            trail_seconds: 3.0,
            ..Default::default()
        },
    };
    std::fs::write(&path, serde_json::to_string(&document).unwrap()).unwrap();
    let loaded = SettingsDocument::read(&path);
    std::fs::remove_file(&path).unwrap();
    assert!(loaded.is_err());
}

#[test]
fn a_failed_load_preserves_live_settings_and_appearance() {
    use bevy::ecs::system::RunSystemOnce;
    let path = test_path("malformed");
    std::fs::write(&path, "{ broken json }").unwrap();
    let mut world = World::new();
    let settings = SimulationSettings {
        cruise_speed: 60.0,
        ..Default::default()
    };
    let appearance = Appearance {
        trail_seconds: 1.5,
        ..Default::default()
    };
    world.insert_resource(settings.clone());
    world.insert_resource(appearance.clone());
    world.init_resource::<MouseField>();
    world.insert_resource(PreferenceFile {
        path: path.clone(),
        load_requested: true,
        ..Default::default()
    });
    world.run_system_once(apply).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(*world.resource::<SimulationSettings>(), settings);
    assert_eq!(*world.resource::<Appearance>(), appearance);
    assert!(
        world
            .resource::<PreferenceFile>()
            .message
            .as_ref()
            .unwrap()
            .starts_with("Load failed:")
    );
}
