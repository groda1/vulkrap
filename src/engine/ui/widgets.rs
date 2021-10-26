use crate::engine::console::Console;
use crate::engine::datatypes::{TexturedColoredVertex2D, WindowExtent};
use crate::engine::stats;
use crate::engine::ui::colors::{
    COLOR_BLACK, COLOR_INPUT_TEXT, COLOR_TEXT, COLOR_TEXT_CVAR, COLOR_TEXT_DEBUG, COLOR_TEXT_ERROR, COLOR_TEXT_INFO,
    COLOR_WHITE,
};
use crate::engine::ui::draw::{draw_quad, draw_text, draw_text_shadowed};
use crate::log::logger;
use crate::log::logger::{LogMessage, MessageLevel};
use crate::renderer::buffer::BufferObjectHandle;
use crate::renderer::context::{Context, PipelineHandle};
use crate::renderer::pipeline::PipelineDrawCommand;
use crate::ENGINE_VERSION;

use cgmath::{Vector2, Vector4};
use std::ptr;

// Console
const BORDER_OFFSET: u32 = 4;
const CONSOLE_HEIGHT_FACTOR: f32 = 0.75;
const TEXT_SIZE_PX: u32 = 16;
const LINE_SPACING: u32 = 2;
const INPUT_BOX_OFFSET: u32 = 2;

pub struct ConsoleRenderer {
    main_pipeline: PipelineHandle,
    text_pipeline: PipelineHandle,

    simple_dynamic_vertex_buffer: BufferObjectHandle,
    text_dynamic_vertex_buffer: BufferObjectHandle,

    extent: WindowExtent,
}

impl ConsoleRenderer {
    pub fn new(
        context: &mut Context,
        main_pipeline: PipelineHandle,
        text_pipeline: PipelineHandle,
        window_extent: WindowExtent,
    ) -> ConsoleRenderer {
        let text_dynamic_vertex_buffer = context.create_vertex_buffer::<TexturedColoredVertex2D>();
        let simple_dynamic_vertex_buffer = context.create_vertex_buffer::<TexturedColoredVertex2D>();

        ConsoleRenderer {
            main_pipeline,
            text_pipeline,
            simple_dynamic_vertex_buffer,
            text_dynamic_vertex_buffer,
            extent: window_extent,
        }
    }

    pub fn handle_window_resize(&mut self, new_extent: WindowExtent) {
        self.extent = new_extent;
    }

    pub fn draw(
        &mut self,
        context: &mut Context,
        draw_command_buffer: &mut Vec<PipelineDrawCommand>,
        console: &Console,
    ) {
        context.reset_buffer_object(self.simple_dynamic_vertex_buffer);
        context.reset_buffer_object(self.text_dynamic_vertex_buffer);
        self._draw_console(context, console);

        let draw_command_console = PipelineDrawCommand::new_immediate(
            context,
            self.main_pipeline,
            ptr::null(),
            self.simple_dynamic_vertex_buffer,
        );
        let draw_command_text = PipelineDrawCommand::new_immediate(
            context,
            self.text_pipeline,
            ptr::null(),
            self.text_dynamic_vertex_buffer,
        );

        draw_command_buffer.push(draw_command_console);
        draw_command_buffer.push(draw_command_text);
    }

    fn _draw_console(&mut self, context: &mut Context, console: &Console) {
        if !console.is_visible() {
            return;
        }

        let height = (self.extent.height as f32 * CONSOLE_HEIGHT_FACTOR) as u32;
        let offset = (console.get_current_y_offset() * height as f32) as u32;

        draw_quad(
            context,
            self.simple_dynamic_vertex_buffer,
            Vector2::new(0, self.extent.height - height + offset),
            Vector2::new(self.extent.width, height),
            //Vector4::new(0.02, 0.02, 0.02, 0.95),
            Vector4::new(0.02, 0.02, 0.02, 0.95),
        );

        draw_text(
            context,
            self.text_dynamic_vertex_buffer,
            &*format!("> {}", console.get_current_input()),
            Vector2::new(BORDER_OFFSET, self.extent.height - height + offset + BORDER_OFFSET),
            TEXT_SIZE_PX,
            COLOR_INPUT_TEXT,
        );

        if console.is_caret_visible() && console.is_active() {
            draw_quad(
                context,
                self.simple_dynamic_vertex_buffer,
                Vector2::new(
                    BORDER_OFFSET + console.get_input_index() * TEXT_SIZE_PX + (2 * TEXT_SIZE_PX),
                    self.extent.height - height + offset + BORDER_OFFSET,
                ),
                Vector2::new(4, TEXT_SIZE_PX),
                COLOR_INPUT_TEXT,
            );
        }

        self._draw_console_history(context, console, height, offset);
    }

    fn _draw_console_history(&mut self, context: &mut Context, console: &Console, height: u32, offset: u32) {
        let history_count_visible = height / (TEXT_SIZE_PX + LINE_SPACING) - 1;

        let mut __history_len = 0;
        let mut __history_ptr = ptr::null();

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

            draw_text(
                context,
                self.text_dynamic_vertex_buffer,
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

            draw_text(
                context,
                self.text_dynamic_vertex_buffer,
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
    }
}

pub struct RenderStatsRenderer {
    _main_pipeline: PipelineHandle,
    text_pipeline: PipelineHandle,

    _simple_dynamic_vertex_buffer: BufferObjectHandle,
    text_dynamic_vertex_buffer: BufferObjectHandle,

    position: Vector2<u32>,

    active: bool,
}

impl RenderStatsRenderer {
    pub fn new(
        context: &mut Context,
        main_pipeline: PipelineHandle,
        text_pipeline: PipelineHandle,
        window_extent: WindowExtent,
    ) -> RenderStatsRenderer {
        let text_dynamic_vertex_buffer = context.create_vertex_buffer::<TexturedColoredVertex2D>();
        let simple_dynamic_vertex_buffer = context.create_vertex_buffer::<TexturedColoredVertex2D>();

        let position = Vector2::new(8, window_extent.height - 24);

        RenderStatsRenderer {
            _main_pipeline: main_pipeline,
            text_pipeline,
            _simple_dynamic_vertex_buffer: simple_dynamic_vertex_buffer,
            text_dynamic_vertex_buffer,
            position,
            active: true,
        }
    }

    pub fn handle_window_resize(&mut self, new_extent: WindowExtent) {
        self.position = Vector2::new(8, new_extent.height - 24);
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn draw(&mut self, context: &mut Context, draw_command_buffer: &mut Vec<PipelineDrawCommand>) {
        context.reset_buffer_object(self._simple_dynamic_vertex_buffer);
        context.reset_buffer_object(self.text_dynamic_vertex_buffer);

        self._draw_render_stats(context);

        let draw_command_text = PipelineDrawCommand::new_immediate(
            context,
            self.text_pipeline,
            ptr::null(),
            self.text_dynamic_vertex_buffer,
        );

        draw_command_buffer.push(draw_command_text);
    }

    fn _draw_render_stats(&mut self, context: &mut Context) {
        let renderstats = stats::get();

        draw_text_shadowed(
            context,
            self.text_dynamic_vertex_buffer,
            &*format!("FPS: {}", renderstats.get_fps()),
            self.position,
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed(
            context,
            self.text_dynamic_vertex_buffer,
            &*format!("Frame time: {0:.3} ms", renderstats.get_frametime() * 1000f32),
            self.position - Vector2::new(0, 18 * 1),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed(
            context,
            self.text_dynamic_vertex_buffer,
            &*format!("Draw count: {}", renderstats.get_render_stats().draw_command_count),
            self.position - Vector2::new(0, 18 * 3),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed(
            context,
            self.text_dynamic_vertex_buffer,
            &*format!("Triangle count: {}", renderstats.get_render_stats().triangle_count),
            self.position - Vector2::new(0, 18 * 4),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed(
            context,
            self.text_dynamic_vertex_buffer,
            &*format!(
                "TransferCmdBuf: {0:.3} ms",
                renderstats.get_render_stats().transfer_commands_bake_time.as_micros() as f32 / 1000f32
            ),
            self.position - Vector2::new(0, 18 * 6),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed(
            context,
            self.text_dynamic_vertex_buffer,
            &*format!(
                "    DrawCmdBuf: {0:.3} ms",
                renderstats.get_render_stats().draw_commands_bake_time.as_micros() as f32 / 1000f32
            ),
            self.position - Vector2::new(0, 18 * 7),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
    }
}

pub struct TopBar {
    text_pipeline: PipelineHandle,
    text_dynamic_vertex_buffer: BufferObjectHandle,
    extent: WindowExtent,
    active: bool,
}

impl TopBar {
    pub fn new(context: &mut Context, text_pipeline: PipelineHandle, window_extent: WindowExtent) -> TopBar {
        let text_dynamic_vertex_buffer = context.create_vertex_buffer::<TexturedColoredVertex2D>();

        TopBar {
            text_pipeline,
            text_dynamic_vertex_buffer,
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

    pub fn draw(&mut self, context: &mut Context, draw_command_buffer: &mut Vec<PipelineDrawCommand>) {
        context.reset_buffer_object(self.text_dynamic_vertex_buffer);
        draw_text_shadowed(
            context,
            self.text_dynamic_vertex_buffer,
            &*format!("VULKRAP {}.{}.{}", ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2),
            Vector2::new(self.extent.width - 218, self.extent.height - 24),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );

        draw_command_buffer.push(PipelineDrawCommand::new_immediate(
            context,
            self.text_pipeline,
            ptr::null(),
            self.text_dynamic_vertex_buffer,
        ));
    }
}
