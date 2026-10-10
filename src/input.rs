use crate::{
    boids::Flock,
    interaction::{SceneTool, SceneTools},
    mouse::MouseField,
    observation::Observation,
    obstacles::Obstacles,
    settings::{Field, Playback, SimulationSettings},
};
use bevy::prelude::*;
use bevy_egui::{EguiContext, PrimaryEguiContext};

#[derive(Default)]
pub struct DragCapture {
    captured: bool,
}

impl DragCapture {
    fn scene_cursor(
        &mut self,
        cursor: Option<Vec2>,
        over_ui: bool,
        buttons: &ButtonInput<MouseButton>,
        enabled: bool,
    ) -> Option<Vec2> {
        let pressed = buttons.pressed(MouseButton::Left) || buttons.pressed(MouseButton::Right);
        if !pressed {
            self.captured = false;
        } else if over_ui
            && (buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right))
        {
            self.captured = true;
        }
        cursor.filter(|_| enabled && !over_ui && !self.captured)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects camera, input and simulation resources"
)]
pub fn sample_mouse(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut context: Single<&mut EguiContext, With<PrimaryEguiContext>>,
    buttons: Res<ButtonInput<MouseButton>>,
    settings: Res<SimulationSettings>,
    playback: Res<Playback>,
    time: Res<Time>,
    mut capture: Local<DragCapture>,
    mut field: ResMut<Field>,
    mut flock: ResMut<Flock>,
    mut mouse: ResMut<MouseField>,
    mut tools: ResMut<SceneTools>,
    mut observation: ResMut<Observation>,
    mut obstacles: ResMut<Obstacles>,
) {
    let half_size = Vec2::new(window.width(), window.height()) * 0.5;
    if half_size != field.half_size {
        field.half_size = half_size;
        flock.resize_field(half_size, &obstacles.circles);
    }
    let cursor = window.cursor_position().filter(|_| window.focused);
    let ctx = context.get_mut();
    let over_ui = cursor.is_some_and(|point| {
        ctx.layer_id_at(bevy_egui::egui::pos2(
            point.x / ctx.zoom_factor(),
            point.y / ctx.zoom_factor(),
        ))
        .is_some_and(|layer| layer.order != bevy_egui::egui::Order::Background)
    });
    let cursor = capture.scene_cursor(cursor, over_ui, &buttons, true);
    let (camera, transform) = *camera;
    let cursor = cursor.and_then(|position| camera.viewport_to_world_2d(transform, position).ok());
    tools.handle(
        cursor,
        &buttons,
        half_size,
        &flock,
        &mut observation,
        &mut obstacles,
    );
    if tools.tool == SceneTool::Obstacles && buttons.pressed(MouseButton::Left) {
        flock.exclude_obstacles(&obstacles.circles, half_size);
    }
    mouse.update(
        cursor
            .filter(|_| settings.mouse_enabled && !playback.paused && tools.tool == SceneTool::Fly),
        !buttons.pressed(MouseButton::Right),
        time.delta_secs(),
    );
}

pub fn keyboard_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut playback: ResMut<Playback>,
    mut settings: ResMut<SimulationSettings>,
    mut context: Single<&mut EguiContext, With<PrimaryEguiContext>>,
    mut tools: ResMut<SceneTools>,
    mut obstacles: ResMut<Obstacles>,
) {
    if context.get_mut().egui_wants_keyboard_input() {
        return;
    }
    for (key, tool) in [
        (KeyCode::Digit1, SceneTool::Fly),
        (KeyCode::Digit2, SceneTool::Inspect),
        (KeyCode::Digit3, SceneTool::Obstacles),
    ] {
        if keys.just_pressed(key) {
            tools.tool = tool;
        }
    }
    if keys.just_pressed(KeyCode::Delete) && tools.tool == SceneTool::Obstacles {
        tools.delete_selected(&mut obstacles);
    }
    if keys.just_pressed(KeyCode::Space) {
        playback.paused = !playback.paused;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        playback.reset_requested = true;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        settings.mouse_enabled = !settings.mouse_enabled;
    }
}

#[cfg(test)]
mod tests;
