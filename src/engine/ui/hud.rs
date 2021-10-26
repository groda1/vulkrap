use std::path::Path;
use std::ptr;

use cgmath::{Matrix4, SquareMatrix};

use crate::engine::console::Console;
use crate::engine::datatypes::{InstancedCharacter, TexturedColoredVertex2D, TexturedVertex, ViewProjectionUniform, WindowExtent};

use crate::engine::image;
use crate::engine::mesh::{Mesh, MeshManager};
use crate::engine::mesh::PredefinedMesh::TexturedQuad;
use crate::engine::ui::widgets::{ConsoleRenderer, RenderStatsRenderer, TopBar, WipRenderer};
use crate::renderer::buffer::BufferObjectHandle;

use crate::renderer::context::{Context, PipelineHandle, UniformHandle};
use crate::renderer::pipeline::{PipelineConfiguration, PipelineDrawCommand, UniformStage};

use crate::util::file;

pub struct HUD {
    uniform: UniformHandle,

    text_sbo: BufferObjectHandle,
    text_pipeline : PipelineHandle,
    mesh: Mesh,

    render_stats_renderer: RenderStatsRenderer,
    console_renderer: ConsoleRenderer,
    top_bar_renderer: TopBar,
    wip_renderer: WipRenderer,

    window_extent: WindowExtent,
}

impl HUD {
    pub fn new(context: &mut Context, window_extent: WindowExtent, mesh_manager: &MeshManager) -> Self {
        let vp_uniform = context.create_uniform_buffer::<ViewProjectionUniform>(UniformStage::Vertex);
        let text_sbo = context.create_storage_buffer::<InstancedCharacter>();
        let data = _create_view_projection_uniform(window_extent);
        context.set_buffer_object(vp_uniform, data);

        let font_image = image::load_image(Path::new("./resources/textures/font.png"));
        let font_texture = context.add_texture(font_image.width, font_image.height, &font_image.data);
        let sampler = context.add_sampler();

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new(
                "./resources/shaders/2d_flat_coloredtexturevertex_vert.spv",
            )))
            .with_fragment_shader(file::read_file(Path::new(
                "./resources/shaders/2d_flat_coloredtexturevertex_frag.spv",
            )))
            .with_vertex_uniform(0, vp_uniform)
            .with_alpha_blending()
            .build();
        let main_pipeline = context.add_pipeline::<TexturedColoredVertex2D>(pipeline_config);

        let text_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new(
                "./resources/shaders/2d_text_coloredtexturevertex_vert.spv",
            )))
            .with_fragment_shader(file::read_file(Path::new(
                "./resources/shaders/2d_text_coloredtexturevertex_frag.spv",
            )))
            .with_vertex_uniform(0, vp_uniform)
            .with_alpha_blending()
            .add_texture(1, font_texture, sampler)
            .build();
        let text_pipeline = context.add_pipeline::<TexturedColoredVertex2D>(text_pipeline_config);

        let text_ng_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new(
                "./resources/shaders/2d_text_ssbo_vert.spv",
            )))
            .with_fragment_shader(file::read_file(Path::new(
                "./resources/shaders/2d_text_ssbo_frag.spv",
            )))
            .with_vertex_uniform(0, vp_uniform)
            .with_storage_buffer_object(2, text_sbo)
            .with_alpha_blending()
            .add_texture(1, font_texture, sampler)
            .build();
        let text_ng_pipeline = context.add_pipeline::<TexturedVertex>(text_ng_pipeline_config);


        let mesh = *mesh_manager.get_predefined_mesh(TexturedQuad);
        let wip_renderer = WipRenderer::new();
        let render_stats_renderer = RenderStatsRenderer::new(window_extent);
        let console_renderer = ConsoleRenderer::new(context, main_pipeline, text_pipeline, window_extent);
        let top_bar_renderer = TopBar::new(context, text_pipeline, window_extent);

        HUD {
            uniform: vp_uniform,
            text_sbo,
            text_pipeline: text_ng_pipeline,
            mesh,
            render_stats_renderer,
            console_renderer,
            top_bar_renderer,
            wip_renderer,

            window_extent,
        }
    }

    pub fn draw(
        &mut self,
        context: &mut Context,
        draw_command_buffer: &mut Vec<PipelineDrawCommand>,
        console: &Console,
    ) {

        context.reset_buffer_object(self.text_sbo);

        let mut instance_count = 0;

        instance_count += self.wip_renderer.draw(context, self.text_sbo);

        if self.render_stats_renderer.is_active() {
            instance_count += self.render_stats_renderer.draw(context, self.text_sbo);
        }

        let draw_command_text = PipelineDrawCommand::new_buffered(
            self.text_pipeline,
            ptr::null(),
            self.mesh.vertex_buffer,
            self.mesh.index_buffer,
            self.mesh.index_count,
            instance_count,
        );

        draw_command_buffer.push(draw_command_text);
        /*
        if self.top_bar_renderer.is_active() {
            self.top_bar_renderer.draw(context, draw_command_buffer);
        }


        if console.is_visible() {
            self.console_renderer.draw(context, draw_command_buffer, console);
        }*/
    }

    pub fn handle_window_resize(&mut self, context: &mut Context, new_extent: WindowExtent) {
        self.window_extent = new_extent;
        let data = _create_view_projection_uniform(new_extent);
        context.set_buffer_object(self.uniform, data);

        self.top_bar_renderer.handle_window_resize(new_extent);
        self.render_stats_renderer.handle_window_resize(new_extent);
        self.console_renderer.handle_window_resize(new_extent);
    }
}

fn _create_view_projection_uniform(window_extent: WindowExtent) -> ViewProjectionUniform {
    ViewProjectionUniform {
        view: Matrix4::identity(),
        proj: cgmath::ortho(
            0.0,
            window_extent.width as f32,
            0.0,
            window_extent.height as f32,
            -1.0,
            1.0,
        ),
    }
}
