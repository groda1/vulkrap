use std::path::Path;
use cgmath::{Deg, Matrix4, Quaternion, Rotation, Rotation3, SquareMatrix, Vector3, Vector4};
use winit::event::{ElementState, VirtualKeyCode};
use vulkrap::engine::camera::Camera;
use vulkrap::engine::cvars::ConfigVariables;
use vulkrap::engine::datatypes::{Mesh, NormalVertex, TransformColorPushConstant, WindowExtent};
use vulkrap::engine::runtime::{ControlSignal, EngineParameters, VulkrapApplication};
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::{DrawCommand, PipelineConfiguration, PipelineHandle, SWAPCHAIN_PASS};
use vulkrap::util::file;
use vulkrap::vulkrap_start;

const WINDOW_TITLE: &str = "model example";
const WINDOW_WIDTH: u32 = 1500;
const WINDOW_HEIGHT: u32 = 850;

const ROT_SPEED_SUZANNE: f32 = 20.0;
const ROT_SPEED_SPHERE: f32 = 5.0;

struct ModelExample {
    pipeline: PipelineHandle,
    camera: Camera,

    suzanne: Entity,
    sphere: Entity,

}

struct Entity {
    mesh: Mesh,
    push_constant: TransformColorPushConstant,
    orientation: Quaternion<f32>
}


impl VulkrapApplication for ModelExample {
    fn update(&mut self, _context: &mut Context, delta_time_s: f32) {
        self.suzanne.orientation = self.suzanne.orientation * Quaternion::from_angle_y(Deg(delta_time_s * ROT_SPEED_SUZANNE));
        self.suzanne.push_constant.transform = Matrix4::from_translation(Vector3::new(0.0, 0.0, -3.0))
            * Matrix4::from(self.suzanne.orientation);

        self.sphere.orientation = self.sphere.orientation
            * Quaternion::from_angle_z(Deg(delta_time_s * ROT_SPEED_SPHERE));
        self.sphere.push_constant.transform = Matrix4::from_translation(Vector3::new(0.0, 0.0, -3.0))
            * Matrix4::from(self.sphere.orientation) * Matrix4::from(self.suzanne.orientation.invert())
            * Matrix4::from_translation(Vector3::new(2.0, 0.0, 0.0))
            * Matrix4::from_scale(0.5);
    }

    fn draw(&mut self, context: &mut Context) {
       context.add_draw_command(DrawCommand::new_buffered(
            self.pipeline,
            &self.suzanne.push_constant,
            self.suzanne.mesh,
        ));
        context.add_draw_command(DrawCommand::new_buffered(
            self.pipeline,
            &self.sphere.push_constant,
            self.sphere.mesh,
        ));
    }

    fn reconfigure(&mut self, _config: &ConfigVariables) {}

    fn handle_mouse_input(&mut self, _x_delta: f64, _y_delta: f64) {}

    fn handle_window_resize(&mut self, context: &mut Context, _new_size: WindowExtent) {
        self.camera.update_uniform(context);
    }

    fn handle_keyboard_event(&mut self, _context: &mut Context, _key: VirtualKeyCode, _state: ElementState) -> ControlSignal {
        ControlSignal::None
    }
}

impl ModelExample {
    pub fn new(context: &mut Context, engine_params: EngineParameters) -> ModelExample {
        let (_, mesh) = engine_params.mesh_manager.load_new_mesh(context, Path::new("./resources/models/suzanne.obj")).unwrap();

        let suzanne = Entity {
            mesh: *mesh,
            push_constant: TransformColorPushConstant::new(
                Matrix4::identity(),
                Vector4::from((0.25, 0.25, 0.12, 1.0))),
            orientation: Quaternion::from_angle_y(Deg(0.0)),
        };

        let (_, mesh) = engine_params.mesh_manager.load_new_mesh(context, Path::new("./resources/models/sphere.obj")).unwrap();
        let sphere = Entity {
            mesh: *mesh,
            push_constant: TransformColorPushConstant::new(
                Matrix4::identity(),
                Vector4::from((0.0, 0.75, 0.15, 1.0))),
            orientation: Quaternion::from_angle_y(Deg(0.0)),
        };

        let camera = Camera::new(context, engine_params.config);
        let vp_uniform = camera.get_uniform();

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new(
                "./resources/shaders/default_ppl_vert.spv",
            )))
            .with_fragment_shader(file::read_file(Path::new(
                "./resources/shaders/default_ppl_frag.spv",
            )))
            .with_vertex_uniform(0, vp_uniform)
            .with_push_constant::<TransformColorPushConstant>()
            .build();
        let pipeline = context.add_pipeline::<NormalVertex>(SWAPCHAIN_PASS, pipeline_config);

        ModelExample {
            pipeline,
            camera,
            suzanne,
            sphere,
        }
    }
}

fn main() {
    vulkrap_start(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, ModelExample::new);
}
