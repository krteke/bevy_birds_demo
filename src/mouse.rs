use crate::settings::SimulationSettings;
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Debug)]
pub(super) struct CursorSpan {
    start: Vec2,
    end: Vec2,
    remaining: f32,
}

#[derive(Default, Resource)]
pub struct MouseField {
    pub cursor: Option<Vec2>,
    pub repelling: bool,
    pub velocity: Vec2,
    pub(super) segments: Vec<(Vec2, Vec2)>,
    pub(super) pending: VecDeque<CursorSpan>,
    pub(super) sampled_cursor: Option<Vec2>,
}

impl MouseField {
    pub fn update(&mut self, cursor: Option<Vec2>, repelling: bool, dt: f32) {
        let repelling = repelling && cursor.is_some();
        if cursor.is_none()
            || self.sampled_cursor.is_none()
            || repelling != self.repelling
            || dt == 0.0
        {
            self.pending.clear();
            self.segments.clear();
            self.velocity = Vec2::ZERO;
            self.cursor = cursor;
        } else if let (Some(start), Some(end)) = (self.sampled_cursor, cursor) {
            self.pending.push_back(CursorSpan {
                start,
                end,
                remaining: dt,
            });
        }
        self.sampled_cursor = cursor;
        self.repelling = repelling;
    }

    pub fn advance(&mut self, dt: f32) {
        self.segments.clear();
        self.velocity = Vec2::ZERO;
        let mut remaining = dt;
        let mut displacement = Vec2::ZERO;
        while remaining > 1e-7
            && let Some(span) = self.pending.front_mut()
        {
            let take = remaining.min(span.remaining);
            let next = span.start.lerp(span.end, take / span.remaining);
            if self.repelling && next != span.start {
                self.segments.push((span.start, next));
                displacement += next - span.start;
            }
            self.cursor = Some(next);
            span.start = next;
            span.remaining -= take;
            remaining -= take;
            if span.remaining <= 1e-7 {
                self.pending.pop_front();
            }
        }
        self.velocity = displacement / dt;
    }

    pub fn threat_at(&self, position: Vec2, index: usize, settings: &SimulationSettings) -> Vec2 {
        if !settings.mouse_enabled || !self.repelling {
            return Vec2::ZERO;
        }
        let Some(cursor) = self.cursor else {
            return Vec2::ZERO;
        };
        let mut closest = cursor;
        for &(start, end) in &self.segments {
            let point = closest_on_segment(position, start, end);
            if position.distance_squared(point) < position.distance_squared(closest) {
                closest = point;
            }
        }
        let predicted = cursor
            + (self.velocity * settings.prediction)
                .clamp_length_max(settings.predator_radius * 2.0);
        if predicted != cursor {
            let point = closest_on_segment(position, cursor, predicted);
            if position.distance_squared(point) < position.distance_squared(closest) {
                closest = point;
            }
        }
        let away = position - closest;
        let distance = away.length();
        if distance >= settings.predator_radius {
            return Vec2::ZERO;
        }
        let direction = if distance > 0.0 {
            away / distance
        } else {
            Vec2::from_angle(index as f32 * 2.399_963_1)
        };
        let urgency = 0.6 + 0.8 * (self.velocity.length() / 900.0).min(1.0);
        direction
            * settings.predator_strength
            * urgency
            * (1.0 - distance / settings.predator_radius)
    }
}

fn closest_on_segment(position: Vec2, start: Vec2, end: Vec2) -> Vec2 {
    let segment = end - start;
    let fraction = ((position - start).dot(segment) / segment.length_squared()).clamp(0.0, 1.0);
    start + segment * fraction
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prediction_warns_birds_ahead_of_the_current_cursor() {
        let settings = SimulationSettings {
            predator_radius: 25.0,
            prediction: 0.2,
            ..default()
        };
        let mut mouse = MouseField::default();
        mouse.update(Some(Vec2::new(-40.0, 0.0)), true, 0.1);
        mouse.update(Some(Vec2::ZERO), true, 0.1);
        mouse.advance(0.1);
        assert!(
            mouse
                .threat_at(Vec2::new(45.0, 10.0), 0, &settings)
                .length()
                > 0.0
        );
    }
    #[test]
    fn fast_mouse_motion_causes_stronger_escape_than_a_stationary_cursor() {
        let settings = SimulationSettings::default();
        let mut stationary = MouseField::default();
        stationary.update(Some(Vec2::ZERO), true, 0.1);
        let mut moving = MouseField::default();
        moving.update(Some(Vec2::new(-100.0, 0.0)), true, 0.1);
        moving.update(Some(Vec2::ZERO), true, 0.1);
        moving.advance(0.1);
        assert!(
            moving
                .threat_at(Vec2::new(0.0, 20.0), 0, &settings)
                .length()
                > stationary
                    .threat_at(Vec2::new(0.0, 20.0), 0, &settings)
                    .length()
        );
    }
}
