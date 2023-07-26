use std::time::Duration;

pub struct RenderStats {
    pub draw_command_count: u32,
    pub triangle_count: u64,

    pub transfer_commands_bake_time: Duration,
    pub draw_commands_bake_time: Duration,
}

impl RenderStats {
    pub fn new() -> Self {
        RenderStats {
            draw_command_count: 0,
            triangle_count: 0,
            transfer_commands_bake_time: Duration::ZERO,
            draw_commands_bake_time: Duration::ZERO,
        }
    }

    pub fn add_draw_command(&mut self, draw_stats: DrawCommandStats) {
        self.draw_command_count += 1;
        self.triangle_count += draw_stats.triangle_count as u64;
    }
}

impl Default for RenderStats {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DrawCommandStats {
    pub triangle_count: u32,
}

impl DrawCommandStats {
    pub fn new(triangle_count: u32) -> Self {
        DrawCommandStats { triangle_count }
    }
}
