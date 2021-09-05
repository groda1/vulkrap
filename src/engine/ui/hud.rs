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

const COLOR_WHITE: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const COLOR_BLACK: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
const COLOR_RED: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);

pub struct HUD {
    uniform: UniformHandle,
    main_pipeline: PipelineHandle,
    text_pipeline: PipelineHandle,
    quad_textured_mesh: Mesh,
    quad_simple_mesh: Mesh,

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

    pub fn draw(&mut self, draw_command_buffer: &mut Vec<PipelineDrawCommand>) {
        self.text_push_constant_buffer.clear();

        draw::draw_quad(
            draw_command_buffer,
            &mut self.flat_push_constant_buffer,
            self.main_pipeline,
            &self.quad_simple_mesh,
            Vector2::new(500, 500),
            Vector2::new(400, 200),
            Vector4::new(0.02, 0.02, 0.02, 0.95),
        );

        self._draw_render_status(draw_command_buffer);

        draw::draw_text_shadowed(
            draw_command_buffer,
            &mut self.text_push_constant_buffer,
            self.text_pipeline,
            &self.quad_textured_mesh,
            &*format!("VULKRAP {}.{}.{}", ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2),
            Vector2::new(self.window_width - 210, self.window_height - 16),
            16,
            COLOR_WHITE,
            COLOR_BLACK,
        );
    }

    pub fn _draw_render_status(&mut self, draw_command_buffer: &mut Vec<PipelineDrawCommand>) {
        draw::draw_text_shadowed(
            draw_command_buffer,
            &mut self.text_push_constant_buffer,
            self.text_pipeline,
            &self.quad_textured_mesh,
            &*format!("FPS: {}", renderstats::get_fps()),
            Vector2::new(20, self.window_height - 16),
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
            Vector2::new(20, self.window_height - 34),
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
