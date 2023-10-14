use std::path::Path;
use cgmath::{Matrix4, SquareMatrix, Vector2};
use winit::event::{ElementState, VirtualKeyCode};
use vulkrap::engine::cvars::ConfigVariables;
use vulkrap::engine::datatypes::{ViewProjectionUniform, WindowExtent};
use vulkrap::engine::image::load_image;
use vulkrap::engine::mesh::{MeshHandle, PredefinedMesh};
use vulkrap::engine::runtime::{ControlSignal, EngineParameters, VulkrapApplication};
use vulkrap::engine::ui::widgets::TextRenderer;
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::{SWAPCHAIN_PASS, UniformHandle, UniformStage};
use vulkrap::vulkrap_start;

const WINDOW_TITLE: &str = "text sbo example";
const WINDOW_WIDTH: u32 = 1500;
const WINDOW_HEIGHT: u32 = 850;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PushConstantType {
    transform: Matrix4<f32>,
    wobble: f32,
}

impl PushConstantType {
    pub fn new(transform: Matrix4<f32>, wobble: f32) -> Self {
        PushConstantType { transform, wobble }
    }
}

struct TextSBO {
    vp_uniform: UniformHandle,
    text_renderer: TextRenderer,

    text_position: Vector2<u32>,
    text_size: u32,
    text: String,
}

impl VulkrapApplication for TextSBO {
    fn update(&mut self, _context: &mut Context, _delta_time_s: f32) {
        self.text_renderer.set_size(self.text_size);
        self.text_renderer.set_position(self.text_position);
    }

    fn draw(&mut self, context: &mut Context) {
        self.text_renderer.draw(context, self.text.as_str());
    }

    fn reconfigure(&mut self, _config: &ConfigVariables) {}

    fn handle_mouse_input(&mut self, _x_delta: f64, _y_delta: f64) {}

    fn handle_window_resize(&mut self, context: &mut Context, new_size: WindowExtent) {
        let vp = create_view_projection_uniform(new_size);
        context.set_buffer_object(self.vp_uniform, vp);
    }

    fn handle_keyboard_event(&mut self, _context: &mut Context, key: VirtualKeyCode, state: ElementState) -> ControlSignal {
        match (key, state) {
            (VirtualKeyCode::Space, ElementState::Pressed) => self.text += "!",
            _ => {}
        }

        ControlSignal::None
    }
}

impl TextSBO {
    pub fn new(context: &mut Context, engine_params: EngineParameters) -> TextSBO {
        let mesh = *engine_params.mesh_manager.get_mesh(PredefinedMesh::TexturedQuad as MeshHandle);

        let font_image = load_image(Path::new("./resources/textures/font.png"));
        let font_texture = context.add_texture(font_image.width, font_image.height, &font_image.data);
        let sampler = context.add_sampler();

        let vp = create_view_projection_uniform(engine_params.window_extent);
        let vp_uniform = context.create_uniform_buffer::<ViewProjectionUniform>(UniformStage::Vertex);
        context.set_buffer_object(vp_uniform, vp);

        let text_renderer = TextRenderer::new(context, SWAPCHAIN_PASS, vp_uniform, mesh, font_texture, sampler);

        TextSBO {
            vp_uniform,
            text_renderer,
            text_position: Vector2::new(300, 300),
            text_size: 64,
            text: String::from("hello")
        }
    }
}

fn create_view_projection_uniform(window_extent: WindowExtent) -> ViewProjectionUniform {
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

fn main() {
    vulkrap_start(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, TextSBO::new);
}
