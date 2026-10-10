use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Resource, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationSettings {
    pub count: usize,
    pub neighbor_radius: f32,
    pub separation_distance: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub regroup_weight: f32,
    pub cruise_speed: f32,
    pub turn_rate: f32,
    pub individuality: f32,
    pub limit_neighbors: bool,
    pub neighbor_limit: usize,
    pub mouse_enabled: bool,
    pub predator_radius: f32,
    pub predator_strength: f32,
    pub prediction: f32,
    pub alarm_duration: f32,
    pub attraction_strength: f32,
    #[serde(with = "vector")]
    pub wind: Vec2,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            count: 1000,
            neighbor_radius: 64.0,
            separation_distance: 18.0,
            separation_weight: 1.0,
            alignment_weight: 1.6,
            cohesion_weight: 0.45,
            regroup_weight: 0.18,
            cruise_speed: 135.0,
            turn_rate: 220.0,
            individuality: 0.08,
            limit_neighbors: false,
            neighbor_limit: 7,
            mouse_enabled: true,
            predator_radius: 110.0,
            predator_strength: 750.0,
            prediction: 0.18,
            alarm_duration: 0.65,
            attraction_strength: 85.0,
            wind: Vec2::ZERO,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Resource, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub trail_seconds: f32,
    pub trail_opacity: f32,
    pub trail_width: f32,
    pub bird_size: f32,
    pub animate_wings: bool,
    pub show_influence: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            trail_seconds: 0.4,
            trail_opacity: 0.32,
            trail_width: 1.0,
            bird_size: 1.0,
            animate_wings: true,
            show_influence: true,
        }
    }
}

#[derive(Default, Resource)]
pub struct Playback {
    pub paused: bool,
    pub reset_requested: bool,
    pub defaults_requested: bool,
}

#[derive(Resource)]
pub struct Field {
    pub half_size: Vec2,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            half_size: Vec2::new(640.0, 400.0),
        }
    }
}

mod vector {
    use super::*;
    pub fn serialize<S: serde::Serializer>(value: &Vec2, serializer: S) -> Result<S::Ok, S::Error> {
        value.to_array().serialize(serializer)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec2, D::Error> {
        <[f32; 2]>::deserialize(deserializer).map(Vec2::from_array)
    }
}

#[derive(Clone, Copy)]
pub enum SettingGroup {
    Flock,
    Flight,
    Falcon,
}

impl SimulationSettings {
    pub fn reset_group(&mut self, group: SettingGroup) {
        let defaults = Self::default();
        match group {
            SettingGroup::Flock => {
                self.neighbor_radius = defaults.neighbor_radius;
                self.separation_distance = defaults.separation_distance;
                self.separation_weight = defaults.separation_weight;
                self.alignment_weight = defaults.alignment_weight;
                self.cohesion_weight = defaults.cohesion_weight;
                self.regroup_weight = defaults.regroup_weight;
                self.limit_neighbors = defaults.limit_neighbors;
                self.neighbor_limit = defaults.neighbor_limit;
            }
            SettingGroup::Flight => {
                self.cruise_speed = defaults.cruise_speed;
                self.turn_rate = defaults.turn_rate;
                self.individuality = defaults.individuality;
                self.wind = defaults.wind;
            }
            SettingGroup::Falcon => {
                self.predator_radius = defaults.predator_radius;
                self.predator_strength = defaults.predator_strength;
                self.prediction = defaults.prediction;
                self.alarm_duration = defaults.alarm_duration;
                self.attraction_strength = defaults.attraction_strength;
            }
        }
    }
}
