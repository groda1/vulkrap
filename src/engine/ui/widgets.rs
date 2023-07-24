use std::path::Path;
use crate::engine::console::Console;
use crate::engine::datatypes::{InstancedCharacter, InstancedQuad, Mesh, ModelWoblyPushConstant, PosSizeColor2dPushConstant, TexturedVertex, WindowExtent};
use crate::engine::stats;
use crate::engine::ui::colors::{COLOR_BLACK, COLOR_INPUT_TEXT, COLOR_TEXT, COLOR_TEXT_CVAR, COLOR_TEXT_DEBUG, COLOR_TEXT_ERROR, COLOR_TEXT_INFO, COLOR_TEXT_KHRONOS, COLOR_WHITE};
use crate::engine::ui::draw::{draw_quad, draw_text, draw_text_shadowed};
use crate::log::logger;
use crate::log::logger::{MessageLevel};
use crate::renderer::context::Context;
use crate::renderer::types::{BufferObjectHandle, DrawCommand, PipelineConfiguration, PipelineHandle, SamplerHandle, SWAPCHAIN_PASS, TextureHandle, UniformHandle};
use crate::ENGINE_VERSION;

use cgmath::{Vector2, Vector4};
use std::ptr;
use crate::renderer::rawarray::RawArrayPtr;
use crate::util::file;

// Console
const BORDER_OFFSET: u32 = 4;
const CONSOLE_HEIGHT_FACTOR: f32 = 0.75;
const TEXT_SIZE_PX: u32 = 16;
const LINE_SPACING: u32 = 2;
const INPUT_BOX_OFFSET: u32 = 2;

pub struct TexturedQuadRenderer {
    pipeline: PipelineHandle,
    push_constant_buf: PosSizeColor2dPushConstant,
    mesh: Mesh,
}

impl TexturedQuadRenderer {
    pub fn new(context: &mut Context, vp_uniform: UniformHandle, mesh: Mesh, texture: TextureHandle, sampler: SamplerHandle) -> Self {
        let textured_quad_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/2d_texture_push_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/2d_texture_ssbo_frag.spv")))
            .with_vertex_uniform(0, vp_uniform)
            .with_push_constant::<PosSizeColor2dPushConstant>()
            .add_texture(1, texture, sampler)
            .build();

        let pipeline = context.add_pipeline::<TexturedVertex>(SWAPCHAIN_PASS, textured_quad_pipeline_config);

        TexturedQuadRenderer {
            pipeline,
            push_constant_buf: PosSizeColor2dPushConstant::default(),
            mesh,
        }
    }

    pub fn set(&mut self, position: Vector2<f32>, size: Vector2<f32>, color: Vector4<f32>) {
        self.push_constant_buf = PosSizeColor2dPushConstant::new(position, size, color);
    }

    pub fn draw(&mut self, context: &mut Context) {
        context.add_draw_command(DrawCommand::new_buffered(
            self.pipeline,
            &self.push_constant_buf as *const PosSizeColor2dPushConstant as RawArrayPtr,
            self.mesh.vertex_data,
            1,
            0,
        ));
    }
}


pub struct TextRenderer {
    text: String,
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
    text_sbo: BufferObjectHandle,
    quad_sbo: BufferObjectHandle,
    text_pipeline: PipelineHandle,
    quad_pipeline: PipelineHandle,
    mesh: Mesh,
}

impl ConsoleRenderer {
    pub fn new(context: &mut Context,
               vp_uniform: BufferObjectHandle,
               mesh: Mesh,
               extent: WindowExtent,
               font_texture: TextureHandle,
               sampler: SamplerHandle) -> ConsoleRenderer {
        let text_sbo = context.create_storage_buffer::<InstancedCharacter>(500);
        let quad_sbo = context.create_storage_buffer::<InstancedQuad>(10);

        let text_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/2d_text_ssbo_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/2d_texture_ssbo_frag.spv")))
            .with_vertex_uniform(0, vp_uniform)
            .with_storage_buffer_object(2, text_sbo)
            .with_alpha_blending()
            .add_texture(1, font_texture, sampler)
            .build();
        let text_pipeline = context.add_pipeline::<TexturedVertex>(SWAPCHAIN_PASS, text_pipeline_config);
        let quad_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/2d_flat_ssbo_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/2d_flat_ssbo_frag.spv")))
            .with_vertex_uniform(0, vp_uniform)
            .with_storage_buffer_object(2, quad_sbo)
            .with_alpha_blending()
            .build();
        let quad_pipeline = context.add_pipeline::<TexturedVertex>(SWAPCHAIN_PASS, quad_pipeline_config);

        ConsoleRenderer { extent, text_sbo, quad_sbo, text_pipeline, quad_pipeline, mesh }
    }

    pub fn handle_window_resize(&mut self, new_extent: WindowExtent) {
        self.extent = new_extent;
    }

    pub fn draw(
        &mut self,
        context: &mut Context,
        console: &Console) {
        context.reset_buffer_object(self.text_sbo);
        context.reset_buffer_object(self.quad_sbo);

        let height = (self.extent.height as f32 * CONSOLE_HEIGHT_FACTOR) as u32;
        let offset = (console.get_current_y_offset() * height as f32) as u32;

        let mut quad_instance_count = 0;
        let mut text_instance_count = 0;

        // Draw console bg
        quad_instance_count += draw_quad(
            context,
            self.quad_sbo,
            Vector2::new(0, self.extent.height - height + offset),
            Vector2::new(self.extent.width, height),
            //Vector4::new(0.02, 0.02, 0.02, 0.95),
            Vector4::new(0.02, 0.02, 0.02, 0.95),
        );

        // Draw prompt
        text_instance_count += draw_text(
            context,
            self.text_sbo,
            &format!("> {}", console.get_current_input()),
            Vector2::new(BORDER_OFFSET, self.extent.height - height + offset + BORDER_OFFSET),
            TEXT_SIZE_PX,
            COLOR_INPUT_TEXT,
        );

        // Draw caret
        if console.is_caret_visible() && console.is_active() {
            quad_instance_count += draw_quad(
                context,
                self.quad_sbo,
                Vector2::new(
                    BORDER_OFFSET + console.get_input_index() * TEXT_SIZE_PX + (2 * TEXT_SIZE_PX),
                    self.extent.height - height + offset + BORDER_OFFSET,
                ),
                Vector2::new(4, TEXT_SIZE_PX),
                COLOR_INPUT_TEXT,
            );
        }

        // Draw history
        text_instance_count += self._draw_console_history(context, self.text_sbo, console, height, offset);

        context.add_draw_command(DrawCommand::new_buffered(
            self.quad_pipeline,
            ptr::null(),
            self.mesh.vertex_data,
            quad_instance_count,
            0,
        ));

        context.add_draw_command(DrawCommand::new_buffered(
            self.text_pipeline,
            ptr::null(),
            self.mesh.vertex_data,
            text_instance_count,
            0,
        ));
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
        let history = unsafe { std::slice::from_raw_parts(__history_ptr, __history_len) };

        for (i, line) in history.iter().rev().enumerate() {
            let (prefix_text, prefix_color) = match &line.level {
                MessageLevel::Input => (">", COLOR_TEXT),
                MessageLevel::Error => ("[error]", COLOR_TEXT_ERROR),
                MessageLevel::Info => ("[info] ", COLOR_TEXT_INFO),
                MessageLevel::Debug => ("[debug]", COLOR_TEXT_DEBUG),
                MessageLevel::Cvar => ("[cvar] ", COLOR_TEXT_CVAR),
                MessageLevel::Khronos => ("[khr]  ", COLOR_TEXT_KHRONOS),
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

pub struct TextOverlayRenderer {
    extent: WindowExtent,
    text_sbo: BufferObjectHandle,
    quad_sbo: BufferObjectHandle,
    text_pipeline: PipelineHandle,
    quad_pipeline: PipelineHandle,
    mesh: Mesh,

    renderstats_active: bool,
    version_active: bool,
}

impl TextOverlayRenderer {
    pub fn new(context: &mut Context,
               vp_uniform: BufferObjectHandle,
               mesh: Mesh,
               extent: WindowExtent,
               font_texture: TextureHandle,
               sampler: SamplerHandle) -> Self {
        let text_sbo = context.create_storage_buffer::<InstancedCharacter>(500);
        let quad_sbo = context.create_storage_buffer::<InstancedQuad>(10);

        let text_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/2d_text_ssbo_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/2d_texture_ssbo_frag.spv")))
            .with_vertex_uniform(0, vp_uniform)
            .with_storage_buffer_object(2, text_sbo)
            .with_alpha_blending()
            .add_texture(1, font_texture, sampler)
            .build();
        let text_pipeline = context.add_pipeline::<TexturedVertex>(SWAPCHAIN_PASS, text_pipeline_config);
        let quad_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/2d_flat_ssbo_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/2d_flat_ssbo_frag.spv")))
            .with_vertex_uniform(0, vp_uniform)
            .with_storage_buffer_object(2, quad_sbo)
            .with_alpha_blending()
            .build();
        let quad_pipeline = context.add_pipeline::<TexturedVertex>(SWAPCHAIN_PASS, quad_pipeline_config);

        TextOverlayRenderer { extent, text_sbo, quad_sbo, text_pipeline, quad_pipeline, mesh, renderstats_active: true, version_active: true }
    }

    pub fn handle_window_resize(&mut self, new_extent: WindowExtent) {
        self.extent = new_extent;
    }

    pub fn draw(&mut self, context: &mut Context) {
        context.reset_buffer_object(self.text_sbo);
        context.reset_buffer_object(self.quad_sbo);

        let mut foreground_instance_count = 0;
        if self.renderstats_active {
            foreground_instance_count += self.draw_renderstats(context, self.text_sbo);
        }
        if self.version_active {
            foreground_instance_count += self.draw_engine_info(context, self.text_sbo);
        }
        context.add_draw_command(DrawCommand::new_buffered(
            self.text_pipeline,
            ptr::null(),
            self.mesh.vertex_data,
            foreground_instance_count,
            0,
        ));
    }

    fn draw_engine_info(&mut self, context: &mut Context, text_sbo: BufferObjectHandle) -> u32 {
        let mut instance_count = 0;

        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &format!("VULKRAP {}.{}.{}", ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2),
            Vector2::new(self.extent.width.wrapping_sub(218), self.extent.height.wrapping_sub(24)),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count
    }

    fn draw_renderstats(&mut self, context: &mut Context, text_sbo: BufferObjectHandle) -> u32 {
        let position = Vector2::new(8, self.extent.height - 24);
        let renderstats = stats::get();

        let mut instance_count = 0;

        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &format!("FPS: {}", renderstats.get_fps()),
            position,
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &format!("Frame time: {0:.3} ms", renderstats.get_frametime() * 1000f32),
            position - Vector2::new(0, 18),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &format!("Draw count: {}", renderstats.get_render_stats().draw_command_count),
            position - Vector2::new(0, 18 * 3),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &format!("Triangle count: {}", renderstats.get_render_stats().triangle_count),
            position - Vector2::new(0, 18 * 4),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &format!(
                "TransferCmdBuf: {0:.3} ms",
                renderstats.get_render_stats().transfer_commands_bake_time.as_micros() as f32 / 1000f32
            ),
            position - Vector2::new(0, 18 * 6),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
        instance_count += draw_text_shadowed(
            context,
            text_sbo,
            &format!(
                "    DrawCmdBuf: {0:.3} ms",
                renderstats.get_render_stats().draw_commands_bake_time.as_micros() as f32 / 1000f32
            ),
            position - Vector2::new(0, 18 * 7),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );

        instance_count
    }
}
