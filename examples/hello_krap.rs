use std::path::Path;
use cgmath::{Deg, Matrix4, Quaternion, Rotation3, SquareMatrix, Vector3};
use winit::event::{ElementState, VirtualKeyCode};
use vulkrap::engine::cvars::ConfigVariables;
use vulkrap::engine::datatypes::{ColoredVertex, Mesh, ViewProjectionUniform, WindowExtent};
use vulkrap::engine::mesh::{MeshHandle, PredefinedMesh};
use vulkrap::engine::runtime::{ControlSignal, EngineParameters, VulkrapApplication};
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::{DrawCommand, PipelineConfiguration, PipelineHandle, SWAPCHAIN_PASS, UniformHandle, UniformStage};
use vulkrap::util::file;
use vulkrap::vulkrap_start;

const WINDOW_TITLE: &str = "hello vulkrap";
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

struct HelloKrap {
    mesh: Mesh,
    position: Vector3<f32>,
    orientation: Quaternion<f32>,
    push_constant_buf: PushConstantType,
    pipeline: PipelineHandle,
    vp_uniform: UniformHandle,
}

impl VulkrapApplication for HelloKrap {
    fn update(&mut self, _context: &mut Context, delta_time_s: f32) {
        const ROT_SPEED: f32 = 25.0;

        self.orientation = self.orientation * Quaternion::from_angle_z(Deg(-delta_time_s * ROT_SPEED));

        self.push_constant_buf.transform = Matrix4::from_translation(self.position) * Matrix4::from(self.orientation) * Matrix4::from_scale(512.0);
        self.push_constant_buf.wobble += delta_time_s * 5.0;
    }

    fn draw(&mut self, context: &mut Context) {
        context.add_draw_command(DrawCommand::new_buffered(
            self.pipeline,
            &self.push_constant_buf,
            self.mesh,
        ));
    }

    fn reconfigure(&mut self, _config: &ConfigVariables) {}

    fn handle_mouse_input(&mut self, _x_delta: f64, _y_delta: f64) {}

    fn handle_window_resize(&mut self, context: &mut Context, new_size: WindowExtent) {
        let vp = create_view_projection_uniform(new_size);
        context.set_buffer_object(self.vp_uniform, vp);
        self.position = Vector3::new(new_size.width as f32 / 2.0, new_size.height as f32 / 2.0, 0.0)
    }

    fn handle_keyboard_event(&mut self, _context: &mut Context, _key: VirtualKeyCode, _state: ElementState) -> ControlSignal {
        ControlSignal::None
    }
}

impl HelloKrap {
    pub fn new(context: &mut Context, engine_params: EngineParameters) -> HelloKrap {
        let mesh = *engine_params.mesh_manager.get_mesh(PredefinedMesh::ColoredQuad as MeshHandle);
        let push_constant_buf = PushConstantType::new(Matrix4::identity(), 0.0);

        let vp = create_view_projection_uniform(engine_params.window_extent);
        let vp_uniform = context.create_uniform_buffer::<ViewProjectionUniform>(UniformStage::Vertex);
        context.set_buffer_object(vp_uniform, vp);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new(
                "./resources/shaders/example_hello_krap_vert.spv",
            )))
            .with_fragment_shader(file::read_file(Path::new(
                "./resources/shaders/example_hello_krap_frag.spv",
            )))
            .with_push_constant::<PushConstantType>()
            .with_vertex_uniform(0, vp_uniform)
            .build();
        let pipeline = context.add_pipeline::<ColoredVertex>(SWAPCHAIN_PASS, pipeline_config);

         HelloKrap {
            mesh,
            position: Vector3::new(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0, 0.0),
            orientation: Quaternion::from_angle_z(Deg(0.0)),
            push_constant_buf,
            pipeline,
            vp_uniform,
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
    vulkrap_start(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, HelloKrap::new);
}
