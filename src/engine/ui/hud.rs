use std::path::Path;
use cgmath::{Matrix4, SquareMatrix};

use crate::engine::console::Console;
use crate::engine::datatypes::{ViewProjectionUniform, WindowExtent};

use crate::engine::image;
use crate::engine::mesh::PredefinedMesh::TexturedQuad;
use crate::engine::mesh::{MeshHandle, MeshManager};
use crate::engine::ui::widgets::{ConsoleRenderer, TextOverlayRenderer};

use crate::renderer::context::Context;
use crate::renderer::types::{UniformHandle, UniformStage};

pub struct Hud {
    uniform: UniformHandle,

    text_overlay_renderer: TextOverlayRenderer,
    console_renderer: ConsoleRenderer,

    window_extent: WindowExtent,
}

impl Hud {
    pub fn new(context: &mut Context, window_extent: WindowExtent, mesh_manager: &MeshManager) -> Self {
        let vp_uniform = context.create_uniform_buffer::<ViewProjectionUniform>(UniformStage::Vertex);

        let data = _create_view_projection_uniform(window_extent);
        context.set_buffer_object(vp_uniform, data);

        let font_image = image::load_image(Path::new("./resources/textures/font.png"));
        let font_texture = context.add_texture(font_image.width, font_image.height, &font_image.data);
        let sampler = context.add_sampler();

        let mesh = *mesh_manager.get_mesh(TexturedQuad as MeshHandle);

        let text_overlay_renderer = TextOverlayRenderer::new(context, vp_uniform, mesh, window_extent, font_texture, sampler);
        let console_renderer = ConsoleRenderer::new(context, vp_uniform, mesh, window_extent, font_texture, sampler);

        Hud {
            uniform: vp_uniform,
            text_overlay_renderer,
            console_renderer,

            window_extent,
        }
    }

    pub fn draw(&mut self, context: &mut Context, console: &Console) {
        self.text_overlay_renderer.draw(context);

        if console.is_visible() {
            self.console_renderer.draw(context, console);
        }
    }

    pub fn handle_window_resize(&mut self, context: &mut Context, new_extent: WindowExtent) {
        self.window_extent = new_extent;
        let data = _create_view_projection_uniform(new_extent);
        context.set_buffer_object(self.uniform, data);

        self.text_overlay_renderer.handle_window_resize(new_extent);
        self.console_renderer.handle_window_resize(new_extent);
    }

    pub fn get_vp_uniform(&self) -> UniformHandle {
        self.uniform
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
