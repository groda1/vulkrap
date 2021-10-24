use crate::engine::console::Console;
use crate::engine::datatypes::{TexturedColoredVertex2D, WindowExtent};
use crate::engine::stats;
use crate::engine::ui::colors::{
    COLOR_BLACK, COLOR_INPUT_TEXT, COLOR_TEXT, COLOR_TEXT_CVAR, COLOR_TEXT_DEBUG, COLOR_TEXT_ERROR, COLOR_TEXT_INFO,
    COLOR_WHITE,
};
use crate::engine::ui::draw::{draw_quad_ng, draw_text_ng, draw_text_shadowed_ng};
use crate::log::logger;
use crate::log::logger::MessageLevel;
use crate::renderer::buffer::DynamicBufferHandle;
use crate::renderer::context::{Context, DynamicBufferHandler, PipelineHandle, UniformHandle};
use crate::renderer::pipeline::PipelineDrawCommand;
use crate::renderer::rawarray::RawArray;
use crate::ENGINE_VERSION;
use ash::vk::Extent2D;
use cgmath::{Vector2, Vector4};
use std::ptr;
use winit::dpi::PhysicalSize;

// Console
const BORDER_OFFSET: u32 = 4;
const CONSOLE_HEIGHT_FACTOR: f32 = 0.75;
const TEXT_SIZE_PX: u32 = 16;
const LINE_SPACING: u32 = 2;
const INPUT_BOX_OFFSET: u32 = 2;

pub struct ConsoleRenderer {
    main_pipeline: PipelineHandle,
    text_pipeline: PipelineHandle,

    simple_dynamic_vertex_buffer: DynamicBufferHandle,
    text_dynamic_vertex_buffer: DynamicBufferHandle,

    extent: WindowExtent,
}

impl ConsoleRenderer {
    pub fn new(
        context: &mut Context,
        main_pipeline: PipelineHandle,
        text_pipeline: PipelineHandle,
        window_extent: WindowExtent,
    ) -> ConsoleRenderer {
        let text_dynamic_vertex_buffer = context.add_dynamic_vertex_buffer::<TexturedColoredVertex2D>(20000);
        let simple_dynamic_vertex_buffer = context.add_dynamic_vertex_buffer::<TexturedColoredVertex2D>(100);

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
        dynamic_buffer_handler: &mut dyn DynamicBufferHandler,
        draw_command_buffer: &mut Vec<PipelineDrawCommand>,
        console: &Console,
    ) {
        self._draw_console(dynamic_buffer_handler, console);

        let draw_command_console = PipelineDrawCommand::new_raw(
            dynamic_buffer_handler,
            self.main_pipeline,
            ptr::null(),
                self.simple_dynamic_vertex_buffer,
        );
        let draw_command_text = PipelineDrawCommand::new_raw(
            dynamic_buffer_handler,
            self.text_pipeline,
            ptr::null(),
            self.text_dynamic_vertex_buffer,
        );

        draw_command_buffer.push(draw_command_console);
        draw_command_buffer.push(draw_command_text);
    }

    fn _draw_console(&mut self, dynamic_buffer_handler: &mut dyn DynamicBufferHandler, console: &Console) {
        if !console.is_visible() {
            return;
        }

        let height = (self.extent.height as f32 * CONSOLE_HEIGHT_FACTOR) as u32;
        let offset = (console.get_current_y_offset() * height as f32) as u32;

        draw_quad_ng(
            dynamic_buffer_handler.borrow_mut_raw_array(self.simple_dynamic_vertex_buffer),
            Vector2::new(0, self.extent.height - height + offset),
            Vector2::new(self.extent.width, height),
            //Vector4::new(0.02, 0.02, 0.02, 0.95),
            Vector4::new(0.02, 0.02, 0.02, 0.95),
        );

        draw_text_ng(
            dynamic_buffer_handler.borrow_mut_raw_array(self.text_dynamic_vertex_buffer),
            &*format!("> {}", console.get_current_input()),
            Vector2::new(BORDER_OFFSET, self.extent.height - height + offset + BORDER_OFFSET),
            TEXT_SIZE_PX,
            COLOR_INPUT_TEXT,
        );

        if console.is_caret_visible() && console.is_active() {

            draw_quad_ng(
                dynamic_buffer_handler.borrow_mut_raw_array(self.simple_dynamic_vertex_buffer),
                Vector2::new(
                    BORDER_OFFSET + console.get_input_index() * TEXT_SIZE_PX + (2 * TEXT_SIZE_PX),
                    self.extent.height - height + offset + BORDER_OFFSET,
                ),
                Vector2::new(4, TEXT_SIZE_PX),
                COLOR_INPUT_TEXT,
            );

        }

        self._draw_console_history(
            dynamic_buffer_handler.borrow_mut_raw_array(self.text_dynamic_vertex_buffer),
            console,
            height,
            offset,
        );
    }

    fn _draw_console_history(
        &mut self,
        dynamic_vertex_buf: &mut RawArray,
        console: &Console,
        height: u32,
        offset: u32,
    ) {
        let history_count_visible = height / (TEXT_SIZE_PX + LINE_SPACING) - 1;
        let logger_mutex = logger::get();

        let history = logger_mutex.get_history(history_count_visible as usize, console.get_scroll());

        for (i, line) in history.iter().rev().enumerate() {
            let (prefix_text, prefix_color) = match &line.level {
                MessageLevel::Input => (">", COLOR_TEXT),
                MessageLevel::Error => ("[error]", COLOR_TEXT_ERROR),
                MessageLevel::Info => ("[info]", COLOR_TEXT_INFO),
                MessageLevel::Debug => ("[dbg]", COLOR_TEXT_DEBUG),
                MessageLevel::Cvar => ("[cvar]", COLOR_TEXT_CVAR),
                _ => ("---", COLOR_TEXT),
            };

            draw_text_ng(
                dynamic_vertex_buf,
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

            draw_text_ng(
                dynamic_vertex_buf,
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
    main_pipeline: PipelineHandle,
    text_pipeline: PipelineHandle,

    simple_dynamic_vertex_buffer: DynamicBufferHandle,
    text_dynamic_vertex_buffer: DynamicBufferHandle,

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
        let text_dynamic_vertex_buffer = context.add_dynamic_vertex_buffer::<TexturedColoredVertex2D>(20000);
        let simple_dynamic_vertex_buffer = context.add_dynamic_vertex_buffer::<TexturedColoredVertex2D>(100);

        let position = Vector2::new(8, window_extent.height - 24);

        RenderStatsRenderer {
            main_pipeline,
            text_pipeline,
            simple_dynamic_vertex_buffer,
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

    pub fn draw(
        &mut self,
        dynamic_buffer_handler: &mut dyn DynamicBufferHandler,
        draw_command_buffer: &mut Vec<PipelineDrawCommand>,
    ) {
        self._draw_render_stats(dynamic_buffer_handler.borrow_mut_raw_array(self.text_dynamic_vertex_buffer));

        let draw_command_text = PipelineDrawCommand::new_raw(
            dynamic_buffer_handler,
            self.text_pipeline,
            ptr::null(),
            self.text_dynamic_vertex_buffer,
        );


        draw_command_buffer.push(draw_command_text);
    }

    fn _draw_render_stats(&mut self, dynamic_vertex_buf: &mut RawArray) {
        let renderstats = stats::get();

        draw_text_shadowed_ng(
            dynamic_vertex_buf,
            &*format!("FPS: {}", renderstats.get_fps()),
            self.position,
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed_ng(
            dynamic_vertex_buf,
            &*format!("Frame time: {0:.3} ms", renderstats.get_frametime() * 1000f32),
            self.position - Vector2::new(0, 18 * 1),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed_ng(
            dynamic_vertex_buf,
            &*format!("Draw count: {}", renderstats.get_render_stats().draw_command_count),
            self.position - Vector2::new(0, 18 * 3),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed_ng(
            dynamic_vertex_buf,
            &*format!("Triangle count: {}", renderstats.get_render_stats().triangle_count),
            self.position - Vector2::new(0, 18 * 4),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed_ng(
            dynamic_vertex_buf,
            &*format!(
                "TransferCmdBuf: {0:.3} ms",
                renderstats.get_render_stats().transfer_commands_bake_time.as_micros() as f32 / 1000f32
            ),
            self.position - Vector2::new(0, 18 * 6),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        draw_text_shadowed_ng(
            dynamic_vertex_buf,
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
    text_dynamic_vertex_buffer: DynamicBufferHandle,
    extent: WindowExtent,
    active: bool,
}

impl TopBar {
    pub fn new(
        context: &mut Context,
        text_pipeline: PipelineHandle,
        window_extent: WindowExtent,
    ) -> TopBar {
        let text_dynamic_vertex_buffer = context.add_dynamic_vertex_buffer::<TexturedColoredVertex2D>(200);

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

    pub fn draw(
        &mut self,
        dynamic_buffer_handler: &mut dyn DynamicBufferHandler,
        draw_command_buffer: &mut Vec<PipelineDrawCommand>,
    ) {
        draw_text_shadowed_ng(
            dynamic_buffer_handler.borrow_mut_raw_array(self.text_dynamic_vertex_buffer),
            &*format!("VULKRAP {}.{}.{}", ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2),
            Vector2::new(self.extent.width - 218, self.extent.height - 24),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );

        draw_command_buffer.push(PipelineDrawCommand::new_raw(
            dynamic_buffer_handler,
            self.text_pipeline,
            ptr::null(),
            self.text_dynamic_vertex_buffer,
        ));
    }
}
