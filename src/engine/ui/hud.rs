use std::path::Path;
use std::ptr;

use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3, Vector4};
use winit::dpi::PhysicalSize;

use crate::engine::console::Console;
use crate::engine::datatypes::{
    ModelColorPushConstant, SimpleVertex, TexturedColoredVertex2D, TexturedVertex, ViewProjectionUniform, WindowExtent,
};
use crate::engine::mesh::{Mesh, MeshManager, PredefinedMesh};
use crate::engine::ui::colors::{COLOR_BLACK, COLOR_TEXT_CVAR, COLOR_WHITE};
use crate::engine::ui::draw;
use crate::engine::ui::draw::{draw_quad_ng, draw_text_ng, draw_text_shadowed_ng};
use crate::engine::ui::widgets::{ConsoleRenderer, RenderStatsRenderer, TopBar};
use crate::engine::{image, stats};
use crate::log::logger;
use crate::log::logger::MessageLevel;
use crate::renderer::buffer::DynamicBufferHandle;
use crate::renderer::context::{Context, DynamicBufferHandler, PipelineHandle, UniformHandle};
use crate::renderer::pipeline::{PipelineConfiguration, PipelineDrawCommand};
use crate::renderer::rawarray::RawArray;
use crate::renderer::uniform::UniformStage;
use crate::util::file;
use crate::ENGINE_VERSION;

pub struct HUD {
    uniform: UniformHandle,

    main_pipeline: PipelineHandle,
    text_pipeline: PipelineHandle,

    render_stats_renderer: RenderStatsRenderer,
    console_renderer: ConsoleRenderer,
    top_bar_renderer: TopBar,

    window_extent: WindowExtent,
}

impl HUD {
    pub fn new(context: &mut Context, window_extent: WindowExtent) -> Self {
        let uniform = context.create_uniform::<ViewProjectionUniform>(UniformStage::Vertex);
        let data = _create_view_projection_uniform(window_extent);
        context.set_uniform_data(uniform, data);

        let text_dynamic_vertex_buffer = context.add_dynamic_vertex_buffer::<TexturedColoredVertex2D>(15000);

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
            .with_vertex_uniform(0, uniform)
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
            .with_vertex_uniform(0, uniform)
            .with_alpha_blending()
            .add_texture(1, font_texture, sampler)
            .build();
        let text_pipeline = context.add_pipeline::<TexturedColoredVertex2D>(text_pipeline_config);

        let render_stats_renderer = RenderStatsRenderer::new(context, main_pipeline, text_pipeline, window_extent);
        let console_renderer = ConsoleRenderer::new(context, main_pipeline, text_pipeline, window_extent);
        let top_bar_renderer = TopBar::new(context, text_pipeline, window_extent);

        HUD {
            uniform,
            main_pipeline,
            text_pipeline,

            render_stats_renderer,
            console_renderer,
            top_bar_renderer,

            window_extent,
        }
    }

    pub fn draw(
        &mut self,
        dynamic_buffer_handler: &mut dyn DynamicBufferHandler,
        draw_command_buffer: &mut Vec<PipelineDrawCommand>,
        console: &Console,
    ) {
        if self.top_bar_renderer.is_active() {
            self.top_bar_renderer.draw(dynamic_buffer_handler, draw_command_buffer);
        }

        if self.render_stats_renderer.is_active() {
            self.render_stats_renderer
                .draw(dynamic_buffer_handler, draw_command_buffer);
        }

        if console.is_visible() {
            self.console_renderer
                .draw(dynamic_buffer_handler, draw_command_buffer, console);
        }
    }

    pub fn handle_window_resize(&mut self, context: &mut Context, new_extent: WindowExtent) {
        self.window_extent = new_extent;
        let data = _create_view_projection_uniform(new_extent);
        context.set_uniform_data(self.uniform, data);

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
