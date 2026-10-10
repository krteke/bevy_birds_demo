use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Default)]
pub struct StepTiming {
    pub grid: f64,
    pub steering: f64,
    pub motion: f64,
}

impl StepTiming {
    pub fn total(self) -> f64 {
        self.grid + self.steering + self.motion
    }
}

#[derive(Resource)]
pub struct SimulationMetrics {
    samples: VecDeque<StepTiming>,
    frames: VecDeque<f64>,
    pub draw_ms: f64,
}

impl Default for SimulationMetrics {
    fn default() -> Self {
        Self {
            samples: VecDeque::with_capacity(120),
            frames: VecDeque::with_capacity(120),
            draw_ms: 0.0,
        }
    }
}

#[derive(Default)]
pub struct TimingSummary {
    pub mean: StepTiming,
    pub p95: f64,
}

impl SimulationMetrics {
    pub fn record(&mut self, timing: StepTiming) {
        if self.samples.len() == 120 {
            self.samples.pop_front();
        }
        self.samples.push_back(timing);
    }

    pub fn record_frame(&mut self, ms: f64) {
        if self.frames.len() == 120 {
            self.frames.pop_front();
        }
        self.frames.push_back(ms);
    }

    pub fn frame_summary(&self) -> (f64, f64) {
        if self.frames.is_empty() {
            return (0.0, 0.0);
        }
        let mut values: Vec<_> = self.frames.iter().copied().collect();
        summarize_frame_times(&mut values)
    }

    pub fn summary(&self) -> TimingSummary {
        if self.samples.is_empty() {
            return TimingSummary::default();
        }
        let mut summary = TimingSummary::default();
        let mut totals = Vec::with_capacity(self.samples.len());
        for timing in &self.samples {
            summary.mean.grid += timing.grid;
            summary.mean.steering += timing.steering;
            summary.mean.motion += timing.motion;
            totals.push(timing.total());
        }
        let n = totals.len();
        summary.mean.grid /= n as f64;
        summary.mean.steering /= n as f64;
        summary.mean.motion /= n as f64;
        totals.sort_unstable_by(f64::total_cmp);
        summary.p95 = totals[(n * 95).div_ceil(100) - 1];
        summary
    }
}

pub fn summarize_frame_times(values: &mut [f64]) -> (f64, f64) {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.sort_unstable_by(f64::total_cmp);
    (mean, values[(values.len() * 95).div_ceil(100) - 1])
}

pub fn record_frame(time: Res<Time<Real>>, mut metrics: ResMut<SimulationMetrics>) {
    if !time.delta().is_zero() {
        metrics.record_frame(time.delta_secs_f64() * 1000.0);
    }
}
