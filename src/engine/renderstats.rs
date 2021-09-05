use std::sync::Mutex;

const SAMPLE_WINDOW: f32 = 0.2;

lazy_static! {
    static ref BUFFER: Mutex<RenderStats> = Mutex::new(RenderStats::new());
}

pub(crate) fn update_delta_time(delta_time_s: f32) {
    BUFFER.lock().unwrap().update_delta_time(delta_time_s);
}

pub(crate) fn get_fps() -> u32 {
    BUFFER.lock().unwrap().get_fps()
}

pub(crate) fn get_frametime() -> f32 {
    //0.5
    BUFFER.lock().unwrap().get_frametime()
}

struct RenderStats {
    fps: u32,
    frame_time: f32,

    frame_time_samples: f32,
    frame_time_sample_count: u32,
}

impl RenderStats {
    pub fn new() -> Self {
        RenderStats {
            fps: 0,
            frame_time: 0.0,
            frame_time_samples: 0.0,
            frame_time_sample_count: 0,
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

    pub fn get_fps(&self) -> u32 {
        self.fps
    }

    pub fn get_frametime(&self) -> f32 {
        self.frame_time
    }
}
