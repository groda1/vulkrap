use std::sync::{Mutex, MutexGuard};

const SAMPLE_WINDOW: f32 = 0.2;

lazy_static! {
    static ref RENDERSTATS: Mutex<RenderStats> = Mutex::new(RenderStats::new());
}

pub(crate) fn get() -> MutexGuard<'static, RenderStats> {
    RENDERSTATS.lock().unwrap()
}

pub struct RenderStats {
    fps: u32,
    frame_time: f32,

    frame_time_samples: f32,
    frame_time_sample_count: u32,

    index_count: u64,
    draw_count: u32,
}

impl RenderStats {
    pub fn new() -> Self {
        RenderStats {
            fps: 0,
            frame_time: 0.0,
            frame_time_samples: 0.0,
            frame_time_sample_count: 0,
            index_count: 0,
            draw_count: 0,
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

    pub fn set_index_count(&mut self, index_count: u64) {
        self.index_count = index_count;
    }
    pub fn set_draw_count(&mut self, draw_count: u32) {
        self.draw_count = draw_count;
    }

    pub fn get_fps(&self) -> u32 {
        self.fps
    }

    pub fn get_frametime(&self) -> f32 {
        self.frame_time
    }

    pub fn get_index_count(&self) -> u64 {
        self.index_count
    }
    pub fn get_draw_count(&self) -> u32 {
        self.draw_count
    }
}
