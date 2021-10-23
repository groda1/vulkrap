use crate::renderer::stats::RenderStats;
use std::sync::{Mutex, MutexGuard};

const SAMPLE_WINDOW: f32 = 0.2;

lazy_static! {
    static ref ENGINE_STATS: Mutex<EngineStatistics> = Mutex::new(EngineStatistics::new());
}

pub(crate) fn get() -> MutexGuard<'static, EngineStatistics> {
    ENGINE_STATS.lock().unwrap()
}

pub struct EngineStatistics {
    fps: u32,
    frame_time: f32,

    frame_time_samples: f32,
    frame_time_sample_count: u32,

    render_stats: RenderStats,
}

impl EngineStatistics {
    pub fn new() -> Self {
        EngineStatistics {
            fps: 0,
            frame_time: 0.0,
            frame_time_samples: 0.0,
            frame_time_sample_count: 0,
            render_stats: RenderStats::new(),
        }
    }

    pub fn update_delta_time(&mut self, delta_time_s: f32) {
        self.frame_time_samples += delta_time_s;
        self.frame_time_sample_count += 1;

        if self.frame_time_samples >= SAMPLE_WINDOW {
            self.frame_time = self.frame_time_samples / self.frame_time_sample_count as f32;
            self.fps = (1.0 / self.frame_time) as u32;

            self.frame_time_samples = 0.0;
            self.frame_time_sample_count = 0;
        }
    }

    pub fn set_render_stats(&mut self, stats: RenderStats) {
        self.render_stats = stats;
    }

    pub fn get_fps(&self) -> u32 {
        self.fps
    }

    pub fn get_frametime(&self) -> f32 {
        self.frame_time
    }

    pub fn get_render_stats(&self) -> &RenderStats {
        &self.render_stats
    }
}
