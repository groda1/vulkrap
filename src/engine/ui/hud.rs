use std::path::Path;
use std::ptr;

use cgmath::{Matrix4, SquareMatrix};

use crate::engine::console::Console;
use crate::engine::datatypes::{
    InstancedCharacter, InstancedQuad, TexturedVertex, ViewProjectionUniform, WindowExtent,
};

use crate::engine::image;
use crate::engine::mesh::PredefinedMesh::TexturedQuad;
use crate::engine::mesh::{Mesh, MeshManager};
use crate::engine::ui::widgets::{ConsoleRenderer, RenderStatsRenderer, TopBar, WipRenderer};
use crate::renderer::types::BufferObjectHandle;

use crate::renderer::context::Context;
use crate::renderer::types::SWAPCHAIN_PASS;
use crate::renderer::types::{DrawCommand, PipelineConfiguration, PipelineHandle, UniformHandle, UniformStage};

use crate::util::file;

pub struct Hud {
    uniform: UniformHandle,

    text_sbo: BufferObjectHandle,
    quad_sbo: BufferObjectHandle,
    text_pipeline: PipelineHandle,
    quad_pipeline: PipelineHandle,
    mesh: Mesh,

    render_stats_renderer: RenderStatsRenderer,
    console_renderer: ConsoleRenderer,
    top_bar_renderer: TopBar,
    wip_renderer: WipRenderer,

    window_extent: WindowExtent,
}

impl Hud {
    pub fn new(context: &mut Context, window_extent: WindowExtent, mesh_manager: &MeshManager) -> Self {
        let vp_uniform = context.create_uniform_buffer::<ViewProjectionUniform>(UniformStage::Vertex);
        let text_sbo = context.create_storage_buffer::<InstancedCharacter>(500);
        let quad_sbo = context.create_storage_buffer::<InstancedQuad>(10);
        let data = _create_view_projection_uniform(window_extent);
        context.set_buffer_object(vp_uniform, data);

        let font_image = image::load_image(Path::new("./resources/textures/font.png"));
        let font_texture = context.add_texture(font_image.width, font_image.height, &font_image.data);
        let sampler = context.add_sampler();

        let text_pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/2d_text_ssbo_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/2d_text_ssbo_frag.spv")))
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

        let mesh = *mesh_manager.get_predefined_mesh(TexturedQuad);
        let wip_renderer = WipRenderer::new();
        let render_stats_renderer = RenderStatsRenderer::new(window_extent);
        let console_renderer = ConsoleRenderer::new(window_extent);
        let top_bar_renderer = TopBar::new(window_extent);

        Hud {
            uniform: vp_uniform,
            text_sbo,
            quad_sbo,
            text_pipeline,
            quad_pipeline,
            mesh,
            render_stats_renderer,
            console_renderer,
            top_bar_renderer,
            wip_renderer,

            window_extent,
        }
    }

    pub fn draw(&mut self, context: &mut Context, console: &Console) {
        context.reset_buffer_object(self.text_sbo);
        context.reset_buffer_object(self.quad_sbo);

        let mut foreground_instance_count = 0;
        foreground_instance_count += self.wip_renderer.draw(context, self.text_sbo);
        if self.render_stats_renderer.is_active() {
            foreground_instance_count += self.render_stats_renderer.draw(context, self.text_sbo);
        }
        if self.top_bar_renderer.is_active() {
            foreground_instance_count += self.top_bar_renderer.draw(context, self.text_sbo);
        }

        context.add_draw_command(DrawCommand::new_buffered(
            self.text_pipeline,
            ptr::null(),
            self.mesh.vertex_buffer,
            self.mesh.index_buffer,
            self.mesh.index_count,
            foreground_instance_count,
            0,
        ));

        if console.is_visible() {
            let (console_instance_count_text, instance_count_quad) =
                self.console_renderer
                    .draw(context, self.text_sbo, self.quad_sbo, console);

            context.add_draw_command(DrawCommand::new_buffered(
                self.quad_pipeline,
                ptr::null(),
                self.mesh.vertex_buffer,
                self.mesh.index_buffer,
                self.mesh.index_count,
                instance_count_quad,
                0,
            ));

            context.add_draw_command(DrawCommand::new_buffered(
                self.text_pipeline,
                ptr::null(),
                self.mesh.vertex_buffer,
                self.mesh.index_buffer,
                self.mesh.index_count,
                console_instance_count_text,
                foreground_instance_count,
            ));
        }
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
