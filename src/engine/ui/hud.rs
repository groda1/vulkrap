use std::path::Path;

use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3, Vector4};

use crate::engine::datatypes::{
    ModelColorPushConstant, SimpleVertex, TextPushConstant, TexturedVertex, ViewProjectionUniform,
};
use crate::engine::mesh::{Mesh, MeshManager, PredefinedMesh};
use crate::engine::ui::draw;
use crate::engine::{image, renderstats};
use crate::renderer::context::{Context, PipelineHandle, UniformHandle};
use crate::renderer::pipeline::{PipelineConfiguration, PipelineDrawCommand};
use crate::renderer::uniform::UniformStage;
use crate::util::file;
use crate::ENGINE_VERSION;
use crate::engine::console::Console;
use crate::log::logger;
use crate::log::logger::MessageLevel;

const COLOR_WHITE: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const COLOR_BLACK: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

const COLOR_INPUT_TEXT: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const COLOR_INPUT_TEXT_A: Vector4<f32> = Vector4::new(1.0, 1.0, 1.0, 1.0);
const COLOR_TEXT: Vector3<f32> = Vector3::new(0.7, 0.7, 0.8);
const COLOR_TEXT_ERROR: Vector3<f32> = Vector3::new(0.9, 0.3, 0.3);
const COLOR_TEXT_CVAR: Vector3<f32> = Vector3::new(0.3, 0.3, 0.9);
const COLOR_TEXT_INFO: Vector3<f32> = Vector3::new(0.3, 0.9, 0.3);
const COLOR_TEXT_DEBUG: Vector3<f32> = Vector3::new(0.3, 0.9, 0.9);

// Console
const BORDER_OFFSET: u32 = 4;
const CONSOLE_HEIGHT_FACTOR: f32 = 0.75;
const TEXT_SIZE_PX: u32 = 16;
const LINE_SPACING: u32 = 2;
const INPUT_BOX_OFFSET: u32 = 2;

pub struct HUD {
    uniform: UniformHandle,
    main_pipeline: PipelineHandle,
    text_pipeline: PipelineHandle,
    quad_textured_mesh: Mesh,
    quad_simple_mesh: Mesh,

    // TODO: this is so bad. Figure out a way to store this dynamically in a PipelineDrawCommand
    text_push_constant_buffer: Vec<TextPushConstant>,
    flat_push_constant_buffer: Vec<ModelColorPushConstant>,

    window_width: u32,
    window_height: u32,
}

impl HUD {
    pub fn new(context: &mut Context, mesh_manager: &MeshManager, window_width: u32, window_height: u32) -> Self {
        let uniform = context.create_uniform::<ViewProjectionUniform>(UniformStage::Vertex);
        let data = _create_view_projection_uniform(window_width, window_height);
        context.set_uniform_data(uniform, data);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/flat_color_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/flat_color_frag.spv")))
            .with_push_constant::<ModelColorPushConstant>()
            .with_vertex_uniform(0, uniform)
            .with_alpha_blending()
            .build();
        let main_pipeline = context.add_pipeline::<SimpleVertex>(pipeline_config);

        let font_image = image::load_image(Path::new("./resources/textures/font.png"));
        let font_texture = context.add_texture(font_image.width, font_image.height, &font_image.data);
        let sampler = context.add_sampler();

        let text_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/text_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/text_frag.spv")))
            .with_push_constant::<TextPushConstant>()
            .with_vertex_uniform(0, uniform)
            .with_alpha_blending()
            .add_texture(1, font_texture, sampler)
            .build();
        let text_pipeline = context.add_pipeline::<TexturedVertex>(text_pipeline_config);

        HUD {
            uniform,
            main_pipeline,
            text_pipeline,
            quad_textured_mesh: *mesh_manager.get_predefined_mesh(PredefinedMesh::TexturedQuad),
            quad_simple_mesh: *mesh_manager.get_predefined_mesh(PredefinedMesh::SimpleQuad),
            text_push_constant_buffer: Vec::new(),
            flat_push_constant_buffer: Vec::new(),
            window_width,
            window_height,
        }
    }

    pub fn draw(&mut self, draw_command_buffer: &mut Vec<PipelineDrawCommand>, console: &Console) {
        self.text_push_constant_buffer.clear();
        self.flat_push_constant_buffer.clear();

        self._draw_render_status(draw_command_buffer);
        draw::draw_text_shadowed(
            draw_command_buffer,
            &mut self.text_push_constant_buffer,
            self.text_pipeline,
            &self.quad_textured_mesh,
            &*format!("VULKRAP {}.{}.{}", ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2),
            Vector2::new(self.window_width - 218, self.window_height - 24),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );

        self._draw_console(draw_command_buffer, console)
    }

    fn _draw_console(&mut self, draw_command_buffer: &mut Vec<PipelineDrawCommand>, console: &Console) {
        if !console.is_visible() {
            return;
        }

        let height = (self.window_height as f32 * CONSOLE_HEIGHT_FACTOR) as u32;
        let offset = (console.get_current_y_offset() * height as f32) as u32;

        draw::draw_quad(
            draw_command_buffer,
            &mut self.flat_push_constant_buffer,
            self.main_pipeline,
            &self.quad_simple_mesh,
            Vector2::new(0, self.window_height - height + offset),
            Vector2::new(self.window_width, height),
            Vector4::new(0.02, 0.02, 0.02, 0.95),
        );

        draw::draw_text(
            draw_command_buffer,
            &mut self.text_push_constant_buffer,
            self.text_pipeline,
            &self.quad_textured_mesh,
            &*format!("> {}", console.get_current_input()),
            Vector2::new(BORDER_OFFSET, self.window_height - height + offset + BORDER_OFFSET),
            TEXT_SIZE_PX,
            COLOR_INPUT_TEXT,
        );

        if console.is_caret_visible() && console.is_active() {
            draw::draw_quad(
                draw_command_buffer,
                &mut self.flat_push_constant_buffer,
                self.main_pipeline,
                &self.quad_simple_mesh,
                Vector2::new(
                    BORDER_OFFSET + console.get_input_index() * TEXT_SIZE_PX + (2 * TEXT_SIZE_PX),
                    self.window_height - height + offset + BORDER_OFFSET,
                ),
                Vector2::new(4, TEXT_SIZE_PX),
                COLOR_INPUT_TEXT_A,
            );
        }

        self._draw_console_history(console, draw_command_buffer, height, offset)
    }

    fn _draw_console_history(
        &mut self,
        console : &Console,
        draw_command_buffer: &mut Vec<PipelineDrawCommand>,
        height: u32,
        offset: u32,
    ) {
        let history_count_visible = height / (TEXT_SIZE_PX + LINE_SPACING) - 1;

        let logger_mutex = logger::get();

        let history = logger_mutex.get_history(history_count_visible as usize, console.get_scroll());

        for (i, line) in history.iter().rev().enumerate() {
            let mut x_offset: u32 = 0;

            match &line.level {
                MessageLevel::Input => {
                    x_offset = 1;
                    draw::draw_text(
                        draw_command_buffer,
                        &mut self.text_push_constant_buffer,
                        self.text_pipeline,
                        &self.quad_textured_mesh,
                        ">",
                        Vector2::new(
                            BORDER_OFFSET,
                            self.window_height - height
                                + offset
                                + BORDER_OFFSET
                                + INPUT_BOX_OFFSET
                                + ((i + 1) as u32 * (TEXT_SIZE_PX + LINE_SPACING)),
                        ),
                        TEXT_SIZE_PX,
                        COLOR_TEXT,
                    );
                }
                MessageLevel::Error => {
                    let error_message = "[error] ";
                    x_offset += error_message.len() as u32;
                    draw::draw_text(
                        draw_command_buffer,
                        &mut self.text_push_constant_buffer,
                        self.text_pipeline,
                        &self.quad_textured_mesh,
                        error_message,
                        Vector2::new(
                            BORDER_OFFSET,
                            self.window_height - height
                                + offset
                                + BORDER_OFFSET
                                + INPUT_BOX_OFFSET
                                + ((i + 1) as u32 * (TEXT_SIZE_PX + LINE_SPACING)),
                        ),
                        TEXT_SIZE_PX,
                        COLOR_TEXT_ERROR,
                    );
                }
                MessageLevel::Info => {
                    let error_message = "[info] ";
                    x_offset += error_message.len() as u32;
                    draw::draw_text(
                        draw_command_buffer,
                        &mut self.text_push_constant_buffer,
                        self.text_pipeline,
                        &self.quad_textured_mesh,
                        error_message,
                        Vector2::new(
                            BORDER_OFFSET,
                            self.window_height - height
                                + offset
                                + BORDER_OFFSET
                                + INPUT_BOX_OFFSET
                                + ((i + 1) as u32 * (TEXT_SIZE_PX + LINE_SPACING)),
                        ),
                        TEXT_SIZE_PX,
                        COLOR_TEXT_INFO,
                    );
                }
                MessageLevel::Debug => {
                    let error_message = "[dbg] ";
                    x_offset += error_message.len() as u32;
                    draw::draw_text(
                        draw_command_buffer,
                        &mut self.text_push_constant_buffer,
                        self.text_pipeline,
                        &self.quad_textured_mesh,
                        error_message,
                        Vector2::new(
                            BORDER_OFFSET,
                            self.window_height - height
                                + offset
                                + BORDER_OFFSET
                                + INPUT_BOX_OFFSET
                                + ((i + 1) as u32 * (TEXT_SIZE_PX + LINE_SPACING)),
                        ),
                        TEXT_SIZE_PX,
                        COLOR_TEXT_DEBUG,
                    );
                }
                MessageLevel::Cvar => {
                    let error_message = "[cvar] ";
                    x_offset += error_message.len() as u32;
                    draw::draw_text(
                        draw_command_buffer,
                        &mut self.text_push_constant_buffer,
                        self.text_pipeline,
                        &self.quad_textured_mesh,
                        error_message,
                        Vector2::new(
                            BORDER_OFFSET,
                            self.window_height - height
                                + offset
                                + BORDER_OFFSET
                                + INPUT_BOX_OFFSET
                                + ((i + 1) as u32 * (TEXT_SIZE_PX + LINE_SPACING)),
                        ),
                        TEXT_SIZE_PX,
                        COLOR_TEXT_CVAR,
                    );
                }

                _ => {}
            }

            draw::draw_text(
                draw_command_buffer,
                &mut self.text_push_constant_buffer,
                self.text_pipeline,
                &self.quad_textured_mesh,
                &line.message,
                Vector2::new(
                    BORDER_OFFSET + (x_offset * TEXT_SIZE_PX),
                    self.window_height - height
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

    fn _draw_render_status(&mut self, draw_command_buffer: &mut Vec<PipelineDrawCommand>) {
        draw::draw_text_shadowed(
            draw_command_buffer,
            &mut self.text_push_constant_buffer,
            self.text_pipeline,
            &self.quad_textured_mesh,
            &*format!("FPS: {}", renderstats::get_fps()),
            Vector2::new(8, self.window_height - 24),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );

        draw::draw_text_shadowed(
            draw_command_buffer,
            &mut self.text_push_constant_buffer,
            self.text_pipeline,
            &self.quad_textured_mesh,
            &*format!("Frame time: {0:.3} ms", renderstats::get_frametime() * 1000f32),
            Vector2::new(8, self.window_height - 42),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
    }

    pub fn handle_window_resize(&mut self, context: &mut Context, width: u32, height: u32) {
        self.window_width = width;
        self.window_height = height;
        let data = _create_view_projection_uniform(width, height);
        context.set_uniform_data(self.uniform, data);
    }
}

fn _create_view_projection_uniform(window_width: u32, window_height: u32) -> ViewProjectionUniform {
    ViewProjectionUniform {
        view: Matrix4::identity(),
        proj: cgmath::ortho(0.0, window_width as f32, 0.0, window_height as f32, -1.0, 1.0),
    }
}
