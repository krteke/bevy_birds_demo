use crate::{
    interaction::{SceneTool, SceneTools},
    metrics::{SimulationMetrics, TimingSummary},
    mouse::MouseField,
    observation::Observation,
    obstacles::Obstacles,
    preferences::PreferenceFile,
    presets::Preset,
    settings::{Appearance, Playback, SettingGroup, SimulationSettings},
};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{EguiContexts, egui};

#[derive(Default)]
pub struct Readout {
    elapsed: f32,
    timings: TimingSummary,
    fps: f64,
    frame_mean: f64,
    frame_p95: f64,
}

#[expect(
    clippy::too_many_arguments,
    reason = "The panel edits simulation, playback and appearance and displays diagnostics"
)]
pub fn panel(
    mut contexts: EguiContexts,
    mut settings: ResMut<SimulationSettings>,
    mut appearance: ResMut<Appearance>,
    mut playback: ResMut<Playback>,
    mut file: ResMut<PreferenceFile>,
    mut tools: ResMut<SceneTools>,
    mut observation: ResMut<Observation>,
    mut obstacles: ResMut<Obstacles>,
    mouse: Res<MouseField>,
    diagnostics: Res<DiagnosticsStore>,
    metrics: Res<SimulationMetrics>,
    time: Res<Time>,
    mut readout: Local<Readout>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    readout.elapsed += time.delta_secs();
    if readout.elapsed >= 0.4 {
        readout.elapsed = 0.0;
        readout.timings = metrics.summary();
        (readout.frame_mean, readout.frame_p95) = metrics.frame_summary();
        readout.fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|value| value.smoothed())
            .unwrap_or(0.0);
    }
    ctx.set_visuals(egui::Visuals::dark());
    let color = egui::Color32::from_rgb(98, 217, 199);
    let status = if playback.paused {
        "PAUSED"
    } else if tools.tool == SceneTool::Inspect {
        "INSPECT"
    } else if tools.tool == SceneTool::Obstacles {
        "EDIT OBSTACLES"
    } else if !settings.mouse_enabled {
        "MOUSE OFF"
    } else if mouse.cursor.is_none() {
        "FREE FLIGHT"
    } else if mouse.repelling {
        "FALCON"
    } else {
        "FOLLOW"
    };
    egui::Window::new("MURMURATION")
        .default_pos([22.0, 22.0]).default_width(300.0)
        .default_height(600.0).resizable(false)
        .max_height((ctx.content_rect().height() - 44.0).max(220.0)).vscroll(true)
        .frame(egui::Frame::window(&ctx.global_style()).fill(egui::Color32::from_rgb(7, 12, 17))
            .inner_margin(16).corner_radius(10))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 9.0);
            ui.spacing_mut().slider_width = 150.0;
            ui.colored_label(color, "A living field of motion");
            ui.horizontal(|ui| {
                if ui.button(if playback.paused { "Resume" } else { "Pause" }).clicked() {
                    playback.paused = !playback.paused;
                }
                if ui.button("Reset flock").clicked() { playback.reset_requested = true; }
                if ui.button("Defaults").clicked() { playback.defaults_requested = true; }
            });
            ui.horizontal(|ui| {
                ui.label("Preset");
                for (name, preset) in [("Tight", Preset::Tight), ("Loose", Preset::Loose), ("Evasive", Preset::Evasive)] {
                    if ui.button(name).clicked() { preset.apply(&mut settings); }
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Save settings").on_hover_text("Save simulation and appearance to boids-settings.json; restored at startup.").clicked() { file.save_requested = true; }
                if ui.button("Load settings").clicked() { file.load_requested = true; }
            });
            if let Some(message) = &file.message { ui.small(message); }
            ui.horizontal(|ui| {
                ui.selectable_value(&mut tools.tool, SceneTool::Fly, "1 Fly");
                ui.selectable_value(&mut tools.tool, SceneTool::Inspect, "2 Inspect");
                ui.selectable_value(&mut tools.tool, SceneTool::Obstacles, "3 Obstacles");
            });
            ui.checkbox(&mut settings.mouse_enabled, "Mouse influence");
            ui.colored_label(if mouse.repelling { egui::Color32::from_rgb(250, 125, 92) } else { color }, status);
            match tools.tool {
                SceneTool::Fly => { ui.small("Move: falcon scatters / cuts\nRight drag: attract and follow"); },
                SceneTool::Inspect => { ui.small("Click a bird to inspect; click empty space to clear.\nReturn to Fly to observe live escape forces."); },
                SceneTool::Obstacles => { ui.small("Click: add / select   Drag: move\nDelete: remove selected obstacle"); },
            }
            ui.small("Space: pause   R: reset   M: mouse on/off");
            ui.separator();
            ui.colored_label(color, format!("{:.0} FPS   /   SIM {:.2} ms", readout.fps, readout.timings.mean.total()));
            ui.small(format!("Frame {:.2} ms / p95 {:.2} ms", readout.frame_mean, readout.frame_p95));
            ui.add(egui::Slider::new(&mut settings.count, 100..=5000).step_by(100.0).text("Birds"));
            egui::CollapsingHeader::new("Flock behavior").default_open(true).show(ui, |ui| {
                if ui.small_button("Reset flock rules").clicked() { settings.reset_group(SettingGroup::Flock); }
                ui.add(egui::Slider::new(&mut settings.separation_weight, 0.0..=3.0).text("Separation")).on_hover_text("Push apart nearby birds; stronger values spread the flock.");
                ui.add(egui::Slider::new(&mut settings.alignment_weight, 0.0..=4.0).text("Alignment")).on_hover_text("Match neighbor headings; stronger values synchronize flight.");
                ui.add(egui::Slider::new(&mut settings.cohesion_weight, 0.0..=2.0).text("Cohesion")).on_hover_text("Turn toward the local center; stronger values tighten groups.");
                ui.add(egui::Slider::new(&mut settings.regroup_weight, 0.0..=0.8).text("Regroup")).on_hover_text("Pull separated groups toward the whole flock; zero allows independent flocks.");
                ui.add(egui::Slider::new(&mut settings.neighbor_radius, 40.0..=140.0).text("View radius")).on_hover_text("How far a bird looks for neighbors; larger values cost more CPU time.");
                ui.add(egui::Slider::new(&mut settings.separation_distance, 8.0..=32.0).text("Spacing"));
                ui.checkbox(&mut settings.limit_neighbors, "Use nearest neighbors").on_hover_text("Use the nearest K birds for alignment and cohesion. Separation still checks every nearby bird.");
                ui.add_enabled(settings.limit_neighbors,
                    egui::Slider::new(&mut settings.neighbor_limit, 1..=16).text("Neighbors"));
                ui.small("Spacing checks all nearby birds in both modes.");
            });
            egui::CollapsingHeader::new("Observation").show(ui, |ui| {
                ui.checkbox(&mut observation.color_alarm, "Color escape response");
                ui.checkbox(&mut observation.show_metrics, "Measure flock behavior").on_hover_text("Refresh twice per second, including neighbors outside the perception radius. Extra CPU cost is shown separately.");
                if observation.show_metrics {
                    ui.monospace(format!("Alignment    {:.0}%\nNearest bird {}\nMeasure CPU  {:.2} ms", observation.alignment * 100.0,
                        observation.mean_nearest.map_or_else(|| "—".into(), |value| format!("{value:.1} px")), observation.measurement_ms));
                    ui.small("Alignment: 0 = mixed headings, 100% = same heading.");
                }
                if let Some(index) = observation.selected {
                    ui.label(format!("Selected bird #{}", index + 1));
                    if let Some(details) = &observation.details { ui.label(format!("{} alignment / cohesion neighbors", details.neighbors.len())); }
                    if ui.small_button("Clear selection").clicked() { observation.selected = None; }
                    ui.colored_label(egui::Color32::from_rgb(247, 97, 117), "Red: separation");
                    ui.colored_label(egui::Color32::from_rgb(89, 166, 255), "Blue: alignment");
                    ui.colored_label(egui::Color32::from_rgb(128, 242, 128), "Green: cohesion");
                    ui.colored_label(egui::Color32::from_rgb(255, 171, 59), "Orange: escape");
                    ui.small("Weighted rule contributions, before steering limits.\nCyan marks alignment / cohesion neighbors.\nSpacing checks every bird inside the red circle.");
                }
            });
            egui::CollapsingHeader::new("Obstacles").show(ui, |ui| {
                ui.add(egui::Slider::new(&mut tools.radius, 20.0..=100.0).text("New radius"));
                ui.label(format!("{} circular obstacles", obstacles.circles.len()));
                ui.horizontal(|ui| {
                    if ui.add_enabled(obstacles.selected.is_some(), egui::Button::new("Delete selected")).clicked() { tools.delete_selected(&mut obstacles); }
                    if ui.button("Clear obstacles").clicked() { tools.clear_obstacles(&mut obstacles); }
                });
                if let Some(message) = &tools.message { ui.small(message); }
                ui.small("Choose 3 Obstacles to edit. Circles need a gap.\nWorks while paused; mouse flight effects are disabled.\nObstacles are kept when resetting the flock.");
            });
            egui::CollapsingHeader::new("Flight").show(ui, |ui| {
                if ui.small_button("Reset flight").clicked() { settings.reset_group(SettingGroup::Flight); }
                ui.add(egui::Slider::new(&mut settings.cruise_speed, 40.0..=240.0).text("Cruise speed"));
                ui.add(egui::Slider::new(&mut settings.turn_rate, 60.0..=360.0).suffix("°/s").text("Turn rate")).on_hover_text("Maximum heading change per second; slow turns need more space and may trigger braking.");
                ui.add(egui::Slider::new(&mut settings.individuality, 0.0..=0.25).text("Variation")).on_hover_text("Fixed speed differences between birds; larger values make flight less uniform.");
                let wind_limit = settings.cruise_speed * 0.35;
                ui.add(egui::Slider::new(&mut settings.wind.x, -wind_limit..=wind_limit).text("Wind X"));
                ui.add(egui::Slider::new(&mut settings.wind.y, -wind_limit..=wind_limit).text("Wind Y"));
            });
            egui::CollapsingHeader::new("Falcon and attraction").show(ui, |ui| {
                if ui.small_button("Reset falcon").clicked() { settings.reset_group(SettingGroup::Falcon); }
                ui.add(egui::Slider::new(&mut settings.predator_radius, 40.0..=220.0).text("Threat radius"));
                ui.add(egui::Slider::new(&mut settings.predator_strength, 0.0..=1400.0).text("Threat strength"));
                ui.add(egui::Slider::new(&mut settings.prediction, 0.0..=0.5).suffix("s").text("Prediction")).on_hover_text("Look ahead along the falcon motion; zero disables prediction.");
                ui.add(egui::Slider::new(&mut settings.alarm_duration, 0.0..=1.5).suffix("s").text("Escape memory")).on_hover_text("Continue escaping after a falcon pass while the falcon remains in the scene.");
                ui.add(egui::Slider::new(&mut settings.attraction_strength, 0.0..=300.0).text("Attraction"));
            });
            egui::CollapsingHeader::new("Appearance").show(ui, |ui| {
                if ui.small_button("Reset appearance").clicked() { *appearance = Appearance::default(); }
                ui.add(egui::Slider::new(&mut appearance.trail_seconds, 0.0..=2.0).suffix("s").text("Trail length"));
                ui.add(egui::Slider::new(&mut appearance.trail_opacity, 0.0..=0.8).text("Trail opacity"));
                ui.add(egui::Slider::new(&mut appearance.trail_width, 0.5..=3.0).text("Trail width"));
                ui.add(egui::Slider::new(&mut appearance.bird_size, 0.5..=2.0).text("Bird size"));
                ui.checkbox(&mut appearance.animate_wings, "Animate wings");
                ui.checkbox(&mut appearance.show_influence, "Show mouse field");
            });
            ui.separator();
            egui::CollapsingHeader::new("Performance details").show(ui, |ui| {
                let mean = &readout.timings.mean;
                ui.monospace(format!("Grid       {:.2} ms\nSteering   {:.2} ms\nMotion     {:.2} ms\nSim p95    {:.2} ms\nCPU draw   {:.2} ms",
                    mean.grid, mean.steering, mean.motion, readout.timings.p95, metrics.draw_ms));
                ui.small("Simulation: last 120 steps. CPU draw excludes GPU time.");
            });
        });
    Ok(())
}

#[cfg(test)]
mod tests;
