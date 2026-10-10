use crate::{boids::Flock, observation::Observation, obstacles::Obstacles};
use bevy::prelude::*;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum SceneTool {
    #[default]
    Fly,
    Inspect,
    Obstacles,
}

#[derive(Resource)]
pub struct SceneTools {
    pub tool: SceneTool,
    pub radius: f32,
    pub message: Option<String>,
    dragging: Option<(usize, Vec2)>,
}
impl Default for SceneTools {
    fn default() -> Self {
        Self {
            tool: SceneTool::Fly,
            radius: 45.0,
            message: None,
            dragging: None,
        }
    }
}
impl SceneTools {
    pub fn handle(
        &mut self,
        cursor: Option<Vec2>,
        buttons: &ButtonInput<MouseButton>,
        bounds: Vec2,
        flock: &Flock,
        observation: &mut Observation,
        obstacles: &mut Obstacles,
    ) {
        if self.tool != SceneTool::Obstacles || !buttons.pressed(MouseButton::Left) {
            self.dragging = None;
        }
        let Some(cursor) = cursor else {
            return;
        };
        match self.tool {
            SceneTool::Fly => {}
            SceneTool::Inspect => {
                if buttons.just_pressed(MouseButton::Left) {
                    observation.selected = flock
                        .birds
                        .iter()
                        .enumerate()
                        .map(|(index, bird)| (index, bird.position.distance_squared(cursor)))
                        .filter(|&(_, distance)| distance <= 20.0 * 20.0)
                        .min_by(|a, b| a.1.total_cmp(&b.1))
                        .map(|(index, _)| index);
                }
            }
            SceneTool::Obstacles => {
                if buttons.just_pressed(MouseButton::Left) {
                    let hit = obstacles.circles.iter().position(|circle| {
                        cursor.distance_squared(circle.center) <= circle.radius * circle.radius
                    });
                    if let Some(index) = hit {
                        obstacles.selected = Some(index);
                        self.dragging = Some((index, obstacles.circles[index].center - cursor));
                        self.message = None;
                    } else {
                        match obstacles.place(cursor, self.radius, bounds) {
                            Ok(index) => {
                                obstacles.selected = Some(index);
                                self.dragging =
                                    Some((index, obstacles.circles[index].center - cursor));
                                self.message = None;
                            }
                            Err(error) => {
                                self.message = Some(error);
                                self.dragging = None;
                            }
                        }
                    }
                }
                if let Some((index, offset)) = self.dragging {
                    self.message = obstacles.move_circle(index, cursor + offset, bounds).err();
                }
            }
        }
    }

    pub fn delete_selected(&mut self, obstacles: &mut Obstacles) {
        obstacles.delete_selected();
        self.dragging = None;
    }

    pub fn clear_obstacles(&mut self, obstacles: &mut Obstacles) {
        obstacles.circles.clear();
        obstacles.selected = None;
        self.dragging = None;
        self.message = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn obstacle_tool_creates_a_circle_at_the_scene_click() {
        let mut tools = SceneTools {
            tool: SceneTool::Obstacles,
            ..Default::default()
        };
        let mut buttons = ButtonInput::default();
        buttons.press(MouseButton::Left);
        let mut obstacles = Obstacles::default();
        tools.handle(
            Some(Vec2::new(100.0, 30.0)),
            &buttons,
            Vec2::new(640.0, 400.0),
            &Flock::default(),
            &mut Observation::default(),
            &mut obstacles,
        );
        assert_eq!(obstacles.circles.len(), 1);
        assert_eq!(obstacles.circles[0].center, Vec2::new(100.0, 30.0));
    }
    #[test]
    fn inspect_click_selects_the_closest_bird() {
        let mut flock = Flock::default();
        flock.set_count(2, Vec2::new(640.0, 400.0), 135.0);
        flock.birds[0].position = Vec2::ZERO;
        flock.birds[1].position = Vec2::new(12.0, 0.0);
        let mut tools = SceneTools {
            tool: SceneTool::Inspect,
            ..Default::default()
        };
        let mut buttons = ButtonInput::default();
        buttons.press(MouseButton::Left);
        let mut observation = Observation::default();
        tools.handle(
            Some(Vec2::new(10.0, 0.0)),
            &buttons,
            Vec2::new(640.0, 400.0),
            &flock,
            &mut observation,
            &mut Obstacles::default(),
        );
        assert_eq!(observation.selected, Some(1));
    }
    #[test]
    fn scene_drag_moves_the_selected_circle_and_delete_clears_the_drag() {
        let mut tools = SceneTools {
            tool: SceneTool::Obstacles,
            ..Default::default()
        };
        let mut buttons = ButtonInput::default();
        let mut obstacles = Obstacles::default();
        let bounds = Vec2::new(640.0, 400.0);
        let flock = Flock::default();
        let mut observation = Observation::default();
        buttons.press(MouseButton::Left);
        tools.handle(
            Some(Vec2::new(100.0, 30.0)),
            &buttons,
            bounds,
            &flock,
            &mut observation,
            &mut obstacles,
        );
        buttons.clear();
        tools.handle(
            Some(Vec2::new(180.0, 50.0)),
            &buttons,
            bounds,
            &flock,
            &mut observation,
            &mut obstacles,
        );
        assert_eq!(obstacles.circles[0].center, Vec2::new(180.0, 50.0));
        tools.delete_selected(&mut obstacles);
        tools.handle(
            Some(Vec2::new(200.0, 50.0)),
            &buttons,
            bounds,
            &flock,
            &mut observation,
            &mut obstacles,
        );
        assert!(obstacles.circles.is_empty());
        assert!(obstacles.selected.is_none());
    }
}
