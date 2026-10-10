use super::*;
use bevy_egui::{EguiContext, EguiUserTextures, PrimaryEguiContext};

struct PanelHarness {
    world: World,
    ctx: egui::Context,
    panel: bevy::ecs::system::SystemId<(), Result>,
}

impl PanelHarness {
    fn new() -> Self {
        let mut world = World::new();
        world.init_resource::<SimulationSettings>();
        world.init_resource::<Appearance>();
        world.init_resource::<Playback>();
        world.init_resource::<PreferenceFile>();
        world.init_resource::<SceneTools>();
        world.init_resource::<Observation>();
        world.init_resource::<Obstacles>();
        world.init_resource::<MouseField>();
        world.init_resource::<DiagnosticsStore>();
        world.init_resource::<SimulationMetrics>();
        world.init_resource::<Time>();
        world.init_resource::<EguiUserTextures>();
        let entity = world.spawn(PrimaryEguiContext).id();
        let ctx = world
            .get_mut::<EguiContext>(entity)
            .unwrap()
            .get_mut()
            .clone();
        let panel = world.register_system(panel);
        Self { world, ctx, panel }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
        let mut output = self.ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1280.0, 800.0),
                )),
                events,
                ..default()
            },
            |_| {
                self.world.run_system(self.panel).unwrap().unwrap();
            },
        );
        output.textures_delta.clear();
        output
    }

    fn settle(&mut self) -> egui::FullOutput {
        self.frame(vec![]);
        self.frame(vec![]);
        self.frame(vec![])
    }

    fn click(&mut self, position: egui::Pos2) {
        self.frame(vec![
            egui::Event::PointerMoved(position),
            egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: default(),
            },
        ]);
        self.frame(vec![egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: default(),
        }]);
    }
}

fn text_center(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| {
            if let egui::epaint::Shape::Text(text) = &shape.shape
                && text.galley.job.text == label
            {
                Some(text.pos + text.galley.size() * 0.5)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("panel did not render {label}"))
}

#[test]
fn mouse_checkbox_changes_the_simulation_setting() {
    let mut panel = PanelHarness::new();
    let output = panel.settle();
    panel.click(text_center(&output, "Mouse influence"));
    assert!(!panel.world.resource::<SimulationSettings>().mouse_enabled);
}

#[test]
fn pause_button_changes_playback_without_requesting_a_reset() {
    let mut panel = PanelHarness::new();
    let output = panel.settle();
    panel.click(text_center(&output, "Pause"));
    assert!(panel.world.resource::<Playback>().paused);
    assert!(!panel.world.resource::<Playback>().reset_requested);
}

#[test]
fn separation_slider_edits_separation_without_changing_alignment() {
    let mut panel = PanelHarness::new();
    let output = panel.settle();
    let label = text_center(&output, "Separation");
    panel.click(egui::pos2(160.0, label.y));
    let settings = panel.world.resource::<SimulationSettings>();
    assert!(settings.separation_weight > 1.5);
    assert_eq!(settings.alignment_weight, 1.6);
}

#[test]
fn tight_preset_preserves_population_and_mouse_preference() {
    let mut panel = PanelHarness::new();
    panel.world.resource_mut::<SimulationSettings>().count = 300;
    panel
        .world
        .resource_mut::<SimulationSettings>()
        .mouse_enabled = false;
    let output = panel.settle();
    panel.click(text_center(&output, "Tight"));
    let settings = panel.world.resource::<SimulationSettings>();
    assert_eq!(settings.count, 300);
    assert!(!settings.mouse_enabled);
    assert!(settings.cohesion_weight > 0.45);
}

#[test]
fn group_reset_changes_flock_rules_without_changing_flight() {
    let mut panel = PanelHarness::new();
    panel
        .world
        .resource_mut::<SimulationSettings>()
        .separation_weight = 2.5;
    panel
        .world
        .resource_mut::<SimulationSettings>()
        .cruise_speed = 200.0;
    let output = panel.settle();
    panel.click(text_center(&output, "Reset flock rules"));
    let settings = panel.world.resource::<SimulationSettings>();
    assert_eq!(settings.separation_weight, 1.0);
    assert_eq!(settings.cruise_speed, 200.0);
}
