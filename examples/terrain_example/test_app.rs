use std::path::Path;

use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use winit::event::{ElementState, VirtualKeyCode};

use vulkrap::engine::camera::Camera;
use vulkrap::engine::cvars::ConfigVariables;
use vulkrap::engine::datatypes::{ColoredVertex, ModelWoblyPushConstant, MovementFlags, VertexNormal, WindowExtent};
use vulkrap::engine::entity::WobblyEntity;
use vulkrap::engine::mesh::{MeshManager, PredefinedMesh};
use vulkrap::engine::runtime::{ControlSignal, VulkrapApplication};
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::{PipelineConfiguration, SWAPCHAIN_PASS, UniformHandle, UniformStage, VertexTopology};
use vulkrap::util::file;

use crate::terrain_example::scene::Scene;

pub struct TestApp {
    scene: Scene,

    camera: Camera,
    flags_uniform: UniformHandle,
    movement: MovementFlags,

    draw_wireframe: bool,
}


impl VulkrapApplication for TestApp {

    fn update(&mut self, context: &mut Context, delta_time_s: f32) {
        self.camera.update(context, self.movement, delta_time_s);
        self.scene.update(delta_time_s);
    }

    fn draw(&mut self, context: &mut Context) {
        self.scene.draw(context);
    }

    fn reconfigure(&mut self, config: &ConfigVariables) {
        self.camera.reconfigure(config);
    }

    fn handle_mouse_input(&mut self, x_delta: f64, y_delta: f64) {
        self.camera.update_yaw_pitch(x_delta as f32, y_delta as f32);
    }

    fn handle_window_resize(&mut self, _context: &mut Context, _new_size: WindowExtent) {

    }

    fn handle_keyboard_event(&mut self, context: &mut Context, key: VirtualKeyCode, state: ElementState) -> ControlSignal {

        match (key, state) {
            (VirtualKeyCode::W, ElementState::Pressed) => self.movement.insert(MovementFlags::FORWARD),
            (VirtualKeyCode::W, ElementState::Released) => self.movement.remove(MovementFlags::FORWARD),
            (VirtualKeyCode::S, ElementState::Pressed) => self.movement.insert(MovementFlags::BACKWARD),
            (VirtualKeyCode::S, ElementState::Released) => self.movement.remove(MovementFlags::BACKWARD),
            (VirtualKeyCode::A, ElementState::Pressed) => self.movement.insert(MovementFlags::LEFT),
            (VirtualKeyCode::A, ElementState::Released) => self.movement.remove(MovementFlags::LEFT),
            (VirtualKeyCode::D, ElementState::Pressed) => self.movement.insert(MovementFlags::RIGHT),
            (VirtualKeyCode::D, ElementState::Released) => self.movement.remove(MovementFlags::RIGHT),
            (VirtualKeyCode::Space, ElementState::Pressed) => self.movement.insert(MovementFlags::UP),
            (VirtualKeyCode::Space, ElementState::Released) => self.movement.remove(MovementFlags::UP),
            (VirtualKeyCode::C, ElementState::Pressed) => self.movement.insert(MovementFlags::DOWN),
            (VirtualKeyCode::C, ElementState::Released) => self.movement.remove(MovementFlags::DOWN),
            (VirtualKeyCode::F2, ElementState::Pressed) => self.toggle_wireframe(context),

            _ => {}
        }

        ControlSignal::None
    }
}


impl TestApp {
    pub fn new(context: &mut Context, mesh_manager: &mut MeshManager, config: &ConfigVariables) -> TestApp {

        let camera = Camera::new(context, config);
        let flags_uniform = context.create_uniform_buffer::<u32>(UniformStage::Fragment);

        context.set_buffer_object(flags_uniform, 0_u32);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new(
                "./resources/shaders/crazy_triangle_vert.spv",
            )))
            .with_fragment_shader(file::read_file(Path::new(
                "./resources/shaders/crazy_triangle_frag.spv",
            )))
            .with_push_constant::<ModelWoblyPushConstant>()
            .with_vertex_uniform(0, camera.get_uniform())
            .build();
        let wobbly_pipeline = context.add_pipeline::<ColoredVertex>(SWAPCHAIN_PASS, pipeline_config);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/terrain_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/terrain_frag.spv")))
            .with_vertex_topology(VertexTopology::TriangeStrip)
            .with_vertex_uniform(0, camera.get_uniform())
            .with_fragment_uniform(1, flags_uniform)
            .build();
        let terrain_pipeline = context.add_pipeline::<VertexNormal>(SWAPCHAIN_PASS, pipeline_config);

        let scene = Scene::new(context, &mesh_manager, wobbly_pipeline, terrain_pipeline);

        let mut app = TestApp {
            scene,
            camera,
            flags_uniform,
            movement: MovementFlags::ZERO,
            draw_wireframe: false,
        };

        app.create_entities(mesh_manager);

        app
    }

    fn toggle_wireframe(&mut self, context: &mut Context) {
        self.draw_wireframe = !self.draw_wireframe;

        context.set_buffer_object(self.flags_uniform, self.draw_wireframe as u32);
    }

    fn create_entities(&mut self, mesh_manager: &MeshManager) {
        let quad1 = WobblyEntity::new(
            Vector3::new(0.0, 0.0, -1.0),
            Quaternion::from_angle_z(Deg(0.0)),
            *mesh_manager.get_predefined_mesh(PredefinedMesh::ColoredQuad),
            0.0,
        );

        let quad2 = WobblyEntity::new(
            Vector3::new(0.5, 1.0, -2.0),
            Quaternion::from_angle_z(Deg(0.0)),
            *mesh_manager.get_predefined_mesh(PredefinedMesh::ColoredTriangle),
            0.0,
        );

        self.scene.add_wobbly_entity(quad1);
        self.scene.add_wobbly_entity(quad2);
    }

}


