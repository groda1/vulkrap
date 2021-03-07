use std::time::Instant;

const SAMPLE_COUNT: usize = 5;

pub struct FrameTimer {
    counter: Instant,
    samples: [u32; SAMPLE_COUNT],
    current_frame: usize,
    delta_frame: u32,
}

impl FrameTimer {
    pub fn new() -> FrameTimer {
        FrameTimer {
            counter: Instant::now(),
            samples: [0; SAMPLE_COUNT],
            current_frame: 0,
            delta_frame: 0,
        }
    }

    pub fn tick_frame(&mut self) {
        let time_elapsed = self.counter.elapsed();
        self.counter = Instant::now();

        self.delta_frame = time_elapsed.subsec_micros();
        self.samples[self.current_frame] = self.delta_frame;
        self.current_frame = (self.current_frame + 1) % SAMPLE_COUNT;
    }

    pub fn _get_framerate(&self) -> f32 {
        let mut sum = 0_u32;
        self.samples.iter().for_each(|val| {
            sum += val;
        });

        1000_000.0_f32 / (sum as f32 / SAMPLE_COUNT as f32)
    }

    pub fn _get_frametime_ms(&self) -> f32 {
        // TODO
        0.0
    }

    pub fn delta_time_sec(&self) -> f32 {
        self.delta_frame as f32 / 1000_000.0_f32 // time in second
    }
}
