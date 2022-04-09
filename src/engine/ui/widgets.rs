use crate::engine::console::Console;
use crate::engine::datatypes::WindowExtent;
use crate::engine::stats;
use crate::engine::ui::colors::{
    COLOR_BLACK, COLOR_INPUT_TEXT, COLOR_TEXT, COLOR_TEXT_CVAR, COLOR_TEXT_DEBUG, COLOR_TEXT_ERROR, COLOR_TEXT_INFO,
    COLOR_WHITE,
};
use crate::engine::ui::draw::{draw_quad, draw_text, draw_text_shadowed};
use crate::log::logger;
use crate::log::logger::{LogMessage, MessageLevel};
use crate::renderer::context::Context;
use crate::renderer::types::BufferObjectHandle;
use crate::ENGINE_VERSION;

use cgmath::{Vector2, Vector4};
use std::ptr;

// Console
const BORDER_OFFSET: u32 = 4;
const CONSOLE_HEIGHT_FACTOR: f32 = 0.75;
const TEXT_SIZE_PX: u32 = 16;
const LINE_SPACING: u32 = 2;
const INPUT_BOX_OFFSET: u32 = 2;

pub struct TextRenderer {
    text : String
}

impl TextRenderer {
    pub fn new(text: String) -> Self {
        TextRenderer {
            text
        }
    }
    pub fn draw(&mut self, context: &mut Context, storage_buffer: BufferObjectHandle) -> u32 {
        draw_text(context, storage_buffer, &self.text, Vector2::new(0, 0), 128, COLOR_TEXT_CVAR)
    }
}

pub struct ConsoleRenderer {
    extent: WindowExtent,
}

impl ConsoleRenderer {
    pub fn new(window_extent: WindowExtent) -> ConsoleRenderer {
        ConsoleRenderer { extent: window_extent }
    }

    pub fn handle_window_resize(&mut self, new_extent: WindowExtent) {
        self.extent = new_extent;
    }

    pub fn draw(
        &mut self,
        context: &mut Context,
        text_sbo: BufferObjectHandle,
        quad_sbo: BufferObjectHandle,
        console: &Console,
    ) -> (u32, u32) {
        let height = (self.extent.height as f32 * CONSOLE_HEIGHT_FACTOR) as u32;
        let offset = (console.get_current_y_offset() * height as f32) as u32;

        let mut quad_instance_count = 0;
        let mut text_instance_count = 0;

        // Draw console bg
        quad_instance_count += draw_quad(
            context,
            quad_sbo,
            Vector2::new(0, self.extent.height - height + offset),
            Vector2::new(self.extent.width, height),
            //Vector4::new(0.02, 0.02, 0.02, 0.95),
            Vector4::new(0.02, 0.02, 0.02, 0.95),
        );

        // Draw prompt
        text_instance_count += draw_text(
            context,
            text_sbo,
            &*format!("> {}", console.get_current_input()),
            Vector2::new(BORDER_OFFSET, self.extent.height - height + offset + BORDER_OFFSET),
            TEXT_SIZE_PX,
            COLOR_INPUT_TEXT,
        );

        // Draw caret
        if console.is_caret_visible() && console.is_active() {
            quad_instance_count += draw_quad(
                context,
                quad_sbo,
                Vector2::new(
                    BORDER_OFFSET + console.get_input_index() * TEXT_SIZE_PX + (2 * TEXT_SIZE_PX),
                    self.extent.height - height + offset + BORDER_OFFSET,
                ),
                Vector2::new(4, TEXT_SIZE_PX),
                COLOR_INPUT_TEXT,
            );
        }

        // Draw history
        text_instance_count += self._draw_console_history(context, text_sbo, console, height, offset);

        (text_instance_count, quad_instance_count)
    }

    fn _draw_console_history(
        &mut self,
        context: &mut Context,
        storage_buffer: BufferObjectHandle,
        console: &Console,
        height: u32,
        offset: u32,
    ) -> u32 {
        let history_count_visible = height / (TEXT_SIZE_PX + LINE_SPACING) - 1;

        let mut __history_len = 0;
        let mut __history_ptr = ptr::null();

        let mut instance_count = 0;

        // Hack. To allow logging to occur when building the history log render data
        {
            let logger_mutex = logger::get();
            let history = logger_mutex.get_history(history_count_visible as usize, console.get_scroll());
            __history_len = history.len();
            __history_ptr = history.as_ptr();
        }
        let history = unsafe { std::slice::from_raw_parts(__history_ptr as *const LogMessage, __history_len) };

        for (i, line) in history.iter().rev().enumerate() {
            let (prefix_text, prefix_color) = match &line.level {
                MessageLevel::Input => (">", COLOR_TEXT),
                MessageLevel::Error => ("[error]", COLOR_TEXT_ERROR),
                MessageLevel::Info => ("[info]", COLOR_TEXT_INFO),
                MessageLevel::Debug => ("[dbg]", COLOR_TEXT_DEBUG),
                MessageLevel::Cvar => ("[cvar]", COLOR_TEXT_CVAR),
                _ => ("---", COLOR_TEXT),
            };

            instance_count += draw_text(
                context,
                storage_buffer,
                prefix_text,
                Vector2::new(
                    BORDER_OFFSET,
                    self.extent.height - height
                        + offset
                        + BORDER_OFFSET
                        + INPUT_BOX_OFFSET
                        + ((i + 1) as u32 * (TEXT_SIZE_PX + LINE_SPACING)),
                ),
                TEXT_SIZE_PX,
                prefix_color,
            );
            instance_count += draw_text(
                context,
                storage_buffer,
                &line.message,
                Vector2::new(
                    BORDER_OFFSET + ((1 + prefix_text.len()) as u32 * TEXT_SIZE_PX),
                    self.extent.height - height
                        + offset
                        + BORDER_OFFSET
                        + INPUT_BOX_OFFSET
                        + ((i + 1) as u32 * (TEXT_SIZE_PX + LINE_SPACING)),
                ),
                TEXT_SIZE_PX,
                COLOR_TEXT,
            );
        }
        instance_count
    }
}

pub struct RenderStatsRenderer {
    position: Vector2<u32>,
    active: bool,
}

impl RenderStatsRenderer {
    pub fn new(window_extent: WindowExtent) -> RenderStatsRenderer {
        let position = Vector2::new(8, window_extent.height - 24);

        RenderStatsRenderer { position, active: true }
    }

    pub fn handle_window_resize(&mut self, new_extent: WindowExtent) {
        self.position = Vector2::new(8, new_extent.height - 24);
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn draw(&mut self, context: &mut Context, text_sbo: BufferObjectHandle) -> u32 {
        let renderstats = stats::get();

        let mut instance_count = 0;

        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &*format!("FPS: {}", renderstats.get_fps()),
            self.position,
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &*format!("Frame time: {0:.3} ms", renderstats.get_frametime() * 1000f32),
            self.position - Vector2::new(0, 18),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &*format!("Draw count: {}", renderstats.get_render_stats().draw_command_count),
            self.position - Vector2::new(0, 18 * 3),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &*format!("Triangle count: {}", renderstats.get_render_stats().triangle_count),
            self.position - Vector2::new(0, 18 * 4),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &*format!(
                "TransferCmdBuf: {0:.3} ms",
                renderstats.get_render_stats().transfer_commands_bake_time.as_micros() as f32 / 1000f32
            ),
            self.position - Vector2::new(0, 18 * 6),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &*format!(
                "    DrawCmdBuf: {0:.3} ms",
                renderstats.get_render_stats().draw_commands_bake_time.as_micros() as f32 / 1000f32
            ),
            self.position - Vector2::new(0, 18 * 7),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );

        instance_count
    }
}

pub struct TopBar {
    extent: WindowExtent,
    active: bool,
}

impl TopBar {
    pub fn new(window_extent: WindowExtent) -> TopBar {
        TopBar {
            extent: window_extent,
            active: true,
        }
    }

    pub fn handle_window_resize(&mut self, new_extent: WindowExtent) {
        self.extent = new_extent;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn draw(&mut self, context: &mut Context, text_sbo: BufferObjectHandle) -> u32 {
        let mut instance_count = 0;

        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &*format!("VULKRAP {}.{}.{}", ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2),
            Vector2::new(self.extent.width - 218, self.extent.height - 24),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count
    }
}
