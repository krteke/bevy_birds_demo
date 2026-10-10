use crate::{
    mouse::MouseField,
    settings::{Appearance, SimulationSettings},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsDocument {
    pub simulation: SimulationSettings,
    pub appearance: Appearance,
}

impl SettingsDocument {
    pub fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        let document: Self = serde_json::from_str(&text).map_err(|error| error.to_string())?;
        document.validate()?;
        Ok(document)
    }

    fn validate(&self) -> Result<(), String> {
        let s = &self.simulation;
        let a = &self.appearance;
        if !(100..=5000).contains(&s.count) || !s.count.is_multiple_of(100) {
            return Err("Bird count must be 100–5000, in steps of 100".into());
        }
        if !(1..=16).contains(&s.neighbor_limit) {
            return Err("Neighbor limit must be 1–16".into());
        }
        for (name, value, min, max) in [
            ("View radius", s.neighbor_radius, 40.0, 140.0),
            ("Spacing", s.separation_distance, 8.0, 32.0),
            ("Separation", s.separation_weight, 0.0, 3.0),
            ("Alignment", s.alignment_weight, 0.0, 4.0),
            ("Cohesion", s.cohesion_weight, 0.0, 2.0),
            ("Regroup", s.regroup_weight, 0.0, 0.8),
            ("Cruise speed", s.cruise_speed, 40.0, 240.0),
            ("Turn rate", s.turn_rate, 60.0, 360.0),
            ("Variation", s.individuality, 0.0, 0.25),
            ("Threat radius", s.predator_radius, 40.0, 220.0),
            ("Threat strength", s.predator_strength, 0.0, 1400.0),
            ("Prediction", s.prediction, 0.0, 0.5),
            ("Escape memory", s.alarm_duration, 0.0, 1.5),
            ("Attraction", s.attraction_strength, 0.0, 300.0),
            (
                "Wind X",
                s.wind.x,
                -s.cruise_speed * 0.35,
                s.cruise_speed * 0.35,
            ),
            (
                "Wind Y",
                s.wind.y,
                -s.cruise_speed * 0.35,
                s.cruise_speed * 0.35,
            ),
            ("Trail length", a.trail_seconds, 0.0, 2.0),
            ("Trail opacity", a.trail_opacity, 0.0, 0.8),
            ("Trail width", a.trail_width, 0.5, 3.0),
            ("Bird size", a.bird_size, 0.5, 2.0),
        ] {
            if !(min..=max).contains(&value) {
                return Err(format!("{name} must be between {min} and {max}"));
            }
        }
        Ok(())
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let text = serde_json::to_string_pretty(self).map_err(|error| error.to_string())?;
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, text).map_err(|error| error.to_string())?;
        if let Err(error) = std::fs::rename(&temporary, path) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error.to_string());
        }
        Ok(())
    }
}

#[derive(Resource)]
pub struct PreferenceFile {
    pub path: PathBuf,
    pub save_requested: bool,
    pub load_requested: bool,
    pub message: Option<String>,
}

impl Default for PreferenceFile {
    fn default() -> Self {
        Self {
            path: "boids-settings.json".into(),
            save_requested: false,
            load_requested: false,
            message: None,
        }
    }
}

pub fn restore(
    mut file: ResMut<PreferenceFile>,
    mut settings: ResMut<SimulationSettings>,
    mut appearance: ResMut<Appearance>,
) {
    match file.path.try_exists() {
        Ok(false) => {}
        Ok(true) => match SettingsDocument::read(&file.path) {
            Ok(document) => {
                *settings = document.simulation;
                *appearance = document.appearance;
                file.message = Some("Restored saved settings".into());
            }
            Err(error) => file.message = Some(format!("Restore failed: {error}")),
        },
        Err(error) => file.message = Some(format!("Restore failed: {error}")),
    }
}

pub fn apply(
    mut file: ResMut<PreferenceFile>,
    mut settings: ResMut<SimulationSettings>,
    mut appearance: ResMut<Appearance>,
    mut mouse: ResMut<MouseField>,
) {
    if std::mem::take(&mut file.load_requested) {
        match SettingsDocument::read(&file.path) {
            Ok(document) => {
                *settings = document.simulation;
                *appearance = document.appearance;
                mouse.update(None, false, 0.0);
                file.message = Some("Loaded settings".into());
            }
            Err(error) => file.message = Some(format!("Load failed: {error}")),
        }
    }
    if std::mem::take(&mut file.save_requested) {
        let document = SettingsDocument {
            simulation: settings.clone(),
            appearance: appearance.clone(),
        };
        file.message = Some(match document.write(&file.path) {
            Ok(()) => "Saved settings".into(),
            Err(error) => format!("Save failed: {error}"),
        });
    }
}

#[cfg(test)]
mod tests;
