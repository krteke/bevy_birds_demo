use bevy::prelude::*;

pub const BIRD_CLEARANCE: f32 = 5.0;

#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub center: Vec2,
    pub radius: f32,
}

impl Obstacle {
    pub fn avoidance(&self, position: Vec2, velocity: Vec2, turn_rate: f32, index: usize) -> Vec2 {
        let speed = velocity.length();
        let heading = velocity / speed;
        let toward = self.center - position;
        let forward = toward.dot(heading);
        let lookahead = speed * 0.8 + speed / turn_rate;
        let clearance = self.radius + BIRD_CLEARANCE + 30.0;
        let lateral = (toward - heading * forward).length();
        if forward < -self.radius || forward > lookahead || lateral >= clearance {
            return Vec2::ZERO;
        }
        let cross = heading.perp_dot(toward);
        let side = if cross.abs() > 0.01 {
            -cross.signum()
        } else if index.is_multiple_of(2) {
            1.0
        } else {
            -1.0
        };
        let urgency = (1.0 - lateral / clearance) * (1.0 - forward.max(0.0) / lookahead);
        let away = (position - self.center).normalize_or_zero();
        (heading.perp() * side * 1100.0 + away * 450.0) * urgency
    }

    pub fn speed_limit(&self, position: Vec2, heading: Vec2, turn_rate: f32) -> Option<f32> {
        let toward = self.center - position;
        let forward = toward.dot(heading);
        let lateral_squared = toward.length_squared() - forward * forward;
        let radius = self.radius + BIRD_CLEARANCE + 12.0;
        if forward <= 0.0 || lateral_squared >= radius * radius {
            return None;
        }
        let distance = forward - (radius * radius - lateral_squared.max(0.0)).sqrt();
        Some((distance.max(0.0) * turn_rate * 0.7).max(1.0))
    }

    pub fn constrain_motion(
        &self,
        previous: Vec2,
        position: Vec2,
        velocity: Vec2,
        index: usize,
    ) -> (Vec2, Vec2) {
        let delta = position - previous;
        let distance = delta.length();
        if distance == 0.0 {
            return (position, velocity);
        }
        let direction = delta / distance;
        let toward = self.center - previous;
        let forward = toward.dot(direction);
        let lateral_squared = toward.length_squared() - forward * forward;
        let radius = self.radius + BIRD_CLEARANCE;
        if forward <= 0.0 || lateral_squared >= radius * radius {
            return (position, velocity);
        }
        let contact = forward - (radius * radius - lateral_squared.max(0.0)).sqrt();
        if contact < 0.0 || contact > distance {
            return (position, velocity);
        }
        let point = previous + direction * contact;
        let normal = (point - self.center).normalize();
        let tangent = normal.perp();
        let slide = velocity.dot(tangent);
        let side = if slide.abs() > 0.01 {
            slide.signum()
        } else if index.is_multiple_of(2) {
            1.0
        } else {
            -1.0
        };
        (
            self.center + normal * (radius + 0.01),
            tangent * side * velocity.length(),
        )
    }
}

#[derive(Default, Resource)]
pub struct Obstacles {
    pub circles: Vec<Obstacle>,
    pub selected: Option<usize>,
}

impl Obstacles {
    fn available(&self, center: Vec2, radius: f32, except: Option<usize>) -> Result<(), String> {
        if self.circles.iter().enumerate().any(|(index, circle)| {
            Some(index) != except
                && center.distance(circle.center)
                    < radius + circle.radius + BIRD_CLEARANCE * 2.0 + 2.0
        }) {
            Err("Leave space between obstacles".into())
        } else {
            Ok(())
        }
    }

    pub fn place(&mut self, center: Vec2, radius: f32, bounds: Vec2) -> Result<usize, String> {
        let inside = bounds - Vec2::splat(radius + 12.0);
        let center = center.clamp(-inside, inside);
        self.available(center, radius, None)?;
        self.circles.push(Obstacle { center, radius });
        Ok(self.circles.len() - 1)
    }

    pub fn move_circle(&mut self, index: usize, center: Vec2, bounds: Vec2) -> Result<(), String> {
        let radius = self.circles[index].radius;
        let inside = bounds - Vec2::splat(radius + 12.0);
        let center = center.clamp(-inside, inside);
        self.available(center, radius, Some(index))?;
        self.circles[index].center = center;
        Ok(())
    }

    pub fn delete_selected(&mut self) {
        if let Some(index) = self.selected.take() {
            self.circles.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlapping_obstacle_placement_is_rejected_without_changing_the_scene() {
        let mut obstacles = Obstacles::default();
        let bounds = Vec2::new(640.0, 400.0);
        obstacles.place(Vec2::ZERO, 40.0, bounds).unwrap();
        assert!(obstacles.place(Vec2::new(60.0, 0.0), 40.0, bounds).is_err());
        assert_eq!(obstacles.circles.len(), 1);
    }
    #[test]
    fn approaching_bird_starts_turning_before_contact() {
        let obstacle = Obstacle {
            center: Vec2::ZERO,
            radius: 40.0,
        };
        let steering = obstacle.avoidance(
            Vec2::new(-130.0, 0.0),
            Vec2::X * 135.0,
            220_f32.to_radians(),
            0,
        );
        assert!(steering.y.abs() > 0.0);
    }
    #[test]
    fn swept_collision_prevents_tunneling_through_a_circle() {
        let obstacle = Obstacle {
            center: Vec2::ZERO,
            radius: 40.0,
        };
        let (position, velocity) = obstacle.constrain_motion(
            Vec2::new(-100.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::X * 240.0,
            0,
        );
        assert!(position.x < -44.9, "{position:?}");
        assert!(velocity.is_finite() && velocity.length() > 0.0);
    }
}
