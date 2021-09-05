use std::path::Path;

use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3};

use crate::engine::datatypes::{TextPushConstant, TexturedVertex, ViewProjectionUniform};
use crate::engine::image;
use crate::engine::mesh::Mesh;
use crate::engine::ui::text;
use crate::renderer::context::{Context, PipelineHandle, UniformHandle};
use crate::renderer::pipeline::{PipelineConfiguration, PipelineDrawCommand};
use crate::renderer::uniform::UniformStage;
use crate::util::file;
use crate::ENGINE_VERSION;


const COLOR_WHITE: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const COLOR_BLACK: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
const COLOR_RED: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);

pub struct HUD {
    uniform: UniformHandle,
    text_pipeline: PipelineHandle,
    quad_mesh: Mesh,
    push_constant_buffer: Vec<TextPushConstant>,

    window_width: u32,
    window_height: u32,


}

impl HUD {
    pub fn new(context: &mut Context, mesh: Mesh, window_width: u32, window_height: u32) -> Self {
        let uniform = context.create_uniform::<ViewProjectionUniform>(UniformStage::Vertex);
        let data = _create_view_projection_uniform(window_width, window_height);
        context.set_uniform_data(uniform, data);

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
            text_pipeline,
            quad_mesh: mesh,
            push_constant_buffer: Vec::new(),
            window_width,
            window_height
        }
    }

    pub fn draw(&mut self, draw_command_buffer: &mut Vec<PipelineDrawCommand>) {
        self.push_constant_buffer.clear();

        text::draw_text(
            draw_command_buffer,
            &mut self.push_constant_buffer,
            self.text_pipeline,
            &self.quad_mesh,
            "bajskorv",
            Vector2::new(20, 20),
            16,
            COLOR_RED
        );

        text::draw_text_shadowed(
            draw_command_buffer,
            &mut self.push_constant_buffer,
            self.text_pipeline,
            &self.quad_mesh,
            &*format!("VULKRAP {}.{}.{}", ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2),
            Vector2::new(self.window_width - 210, self.window_height - 16),
            16,
            COLOR_WHITE,
            COLOR_BLACK
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

