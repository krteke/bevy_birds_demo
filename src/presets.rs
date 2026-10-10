use crate::settings::SimulationSettings;

#[derive(Clone, Copy)]
pub enum Preset {
    Tight,
    Loose,
    Evasive,
}

impl Preset {
    pub fn apply(self, settings: &mut SimulationSettings) {
        let mut next = SimulationSettings {
            count: settings.count,
            mouse_enabled: settings.mouse_enabled,
            ..Default::default()
        };
        match self {
            Self::Tight => {
                next.alignment_weight = 2.4;
                next.cohesion_weight = 0.9;
                next.neighbor_radius = 78.0;
                next.separation_distance = 16.0;
                next.cruise_speed = 120.0;
                next.turn_rate = 280.0;
                next.regroup_weight = 0.22;
            }
            Self::Loose => {
                next.alignment_weight = 1.1;
                next.cohesion_weight = 0.15;
                next.regroup_weight = 0.05;
                next.separation_distance = 25.0;
                next.neighbor_radius = 52.0;
                next.individuality = 0.18;
                next.cruise_speed = 100.0;
                next.turn_rate = 150.0;
            }
            Self::Evasive => {
                next.alignment_weight = 2.0;
                next.cohesion_weight = 0.5;
                next.limit_neighbors = true;
                next.cruise_speed = 170.0;
                next.turn_rate = 320.0;
                next.predator_strength = 1000.0;
                next.prediction = 0.28;
                next.alarm_duration = 1.1;
            }
        }
        *settings = next;
    }
}
