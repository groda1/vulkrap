use crate::engine::datatypes::{ViewProjectionUniform, MODEL_COLOR_PUSH_CONSTANT_SIZE, TexturedVertex, ModelColorPushConstant};
use crate::renderer::uniform::UniformStage;
use crate::renderer::context::{UniformHandle, Context, PipelineHandle};
use cgmath::{Matrix4, SquareMatrix, Vector3};
use std::path::Path;
use crate::renderer::pipeline::{PipelineConfiguration, PipelineDrawCommand};
use crate::util::file;
use crate::engine::mesh::Mesh;
use super::image;

pub struct UI {
    uniform : UniformHandle,
    font_pipeline : PipelineHandle,
    mesh: Mesh,
    push_constant_buffer : Vec<ModelColorPushConstant>,
}

impl UI {
    pub fn new(context : &mut Context, mesh : Mesh, window_width : u32, window_height: u32) -> Self {
        let uniform = context.create_uniform::<ViewProjectionUniform>(UniformStage::Vertex);
        let data = _create_view_projection_uniform(window_width, window_height);
        context.set_uniform_data(uniform, data);

        let font_image = image::load_image(Path::new("./resources/textures/font.png"));
        let font_texture = context.add_texture(font_image.width, font_image.height, &font_image.data);
        let sampler = context.add_sampler();

        let font_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/flat_textured_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/flat_textured_frag.spv")))
            .with_push_constant(MODEL_COLOR_PUSH_CONSTANT_SIZE)
            .with_vertex_uniform(0, uniform)
            .add_texture(1, font_texture, sampler)
            .build();
        let font_pipeline = context.add_pipeline::<TexturedVertex>(font_pipeline_config);

        UI {
            uniform,
            font_pipeline,
            mesh,
            push_constant_buffer: Vec::new()
        }
    }

    pub fn draw(&mut self, draw_command_buffer: &mut Vec<PipelineDrawCommand>) {
        self.push_constant_buffer.clear();

        self.push_constant_buffer.push(ModelColorPushConstant::new(
            Matrix4::from_translation(Vector3::new(500.0, 500.0, 0.0)) *Matrix4::from_scale(512.0) ,
            Vector3::new(0.0, 1.0, 1.0)));

        draw_command_buffer.push(
            PipelineDrawCommand::new(
                self.font_pipeline,
                self.mesh.vertex_buffer,
                self.mesh.index_buffer,
                self.mesh.index_count,
                &self.push_constant_buffer[0] as *const ModelColorPushConstant as *const u8,
            ));
    }

    pub fn handle_window_resize(&mut self, context: &mut Context, width: u32, height: u32) {
        let data = _create_view_projection_uniform(width, height);
        context.set_uniform_data(self.uniform, data);
    }
}

fn _create_view_projection_uniform(window_width: u32, window_height : u32) -> ViewProjectionUniform {
    ViewProjectionUniform {
        view: Matrix4::identity(),
        proj: cgmath::ortho(0.0, window_width as f32, 0.0, window_height as f32, -1.0, 1.0),
    }
}
