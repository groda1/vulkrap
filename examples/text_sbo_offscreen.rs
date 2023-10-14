use std::path::Path;
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector4};
use winit::event::{ElementState, VirtualKeyCode};
use vulkrap::engine::cvars::ConfigVariables;
use vulkrap::engine::datatypes::{ViewProjectionUniform, WindowExtent};
use vulkrap::engine::image::load_image;
use vulkrap::engine::mesh::{MeshHandle, PredefinedMesh};
use vulkrap::engine::runtime::{ControlSignal, EngineParameters, VulkrapApplication};
use vulkrap::engine::ui::widgets::{TextRenderer, TexturedQuadRenderer};
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::UniformStage;
use vulkrap::vulkrap_start;

const WINDOW_TITLE: &str = "text sbo example offscreen";
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
    texture_quad_renderer: TexturedQuadRenderer,
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
        self.texture_quad_renderer.draw(context);
    }

    fn reconfigure(&mut self, _config: &ConfigVariables) {}

    fn handle_mouse_input(&mut self, _x_delta: f64, _y_delta: f64) {}

    fn handle_window_resize(&mut self, _context: &mut Context, _new_size: WindowExtent) {

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

        let render_texture = context.add_render_texture(384, 216);
        let sampler = context.add_sampler();
        let pass = context.create_render_pass(render_texture, 1000).unwrap();

        let mesh = *engine_params.mesh_manager.get_mesh(PredefinedMesh::TexturedQuad as MeshHandle);

        let font_image = load_image(Path::new("./resources/textures/font.png"));
        let font_texture = context.add_texture(font_image.width, font_image.height, &font_image.data);
        let vp = create_view_projection_uniform(WindowExtent::new(384, 216));
        let vp_uniform = context.create_uniform_buffer::<ViewProjectionUniform>(UniformStage::Vertex);
        context.set_buffer_object(vp_uniform, vp);

        let text_renderer = TextRenderer::new(context, pass, vp_uniform, mesh, font_texture, sampler);

        let mut texture_quad_renderer = TexturedQuadRenderer::new(context, engine_params.hud_vp_uniform, mesh, render_texture, sampler);
        texture_quad_renderer.set(
            Vector2::new((engine_params.window_extent.width / 2) as f32, (engine_params.window_extent.height / 2) as f32),
            Vector2::new(engine_params.window_extent.width as f32, engine_params.window_extent.height as f32),
            Vector4::new(1.0, 1.0, 1.0, 1.0));


        TextSBO {
            texture_quad_renderer,
            text_renderer,
            text_position: Vector2::new(150, 100),
            text_size: 8,
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
