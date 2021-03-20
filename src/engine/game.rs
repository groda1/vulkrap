use std::path::Path;

use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use winit::event::{ElementState, VirtualKeyCode};
use winit::window::Window;

use crate::engine::camera::Camera;
use crate::engine::datatypes::{
    ColoredVertex, SimpleVertex, VertexNormal, ViewProjectionUniform, MODEL_COLOR_PUSH_CONSTANT_SIZE,
    MODEL_WOBLY_PUSH_CONSTANT_SIZE,
};
use crate::engine::entity::{FlatColorEntity, WobblyEntity};
use crate::engine::mesh::{MeshManager, PredefinedMesh};
use crate::engine::scene::Scene;
use crate::renderer::context::Context;
use crate::renderer::pipeline::{PipelineConfiguration, PipelineHandle, VertexTopology};
use crate::util::file;

pub struct VulkrapApplication {
    context: Context,
    mesh_manager: MeshManager,
    scene: Scene,

    camera: Camera,

    main_pipeline: PipelineHandle,
    flat_color_pipeline: PipelineHandle,
    terrain_pipeline: PipelineHandle,

    movement: MovementFlags,

    elapsed_time_s: f32,
}

impl VulkrapApplication {
    pub fn new(window: &Window) -> VulkrapApplication {
        let mut context = Context::new(window);
        let mesh_manager = MeshManager::new(&mut context);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new(
                "./resources/shaders/crazy_triangle_vert.spv",
            )))
            .with_fragment_shader(file::read_file(Path::new(
                "./resources/shaders/crazy_triangle_frag.spv",
            )))
            .with_push_constant(MODEL_WOBLY_PUSH_CONSTANT_SIZE)
            .build();
        let main_pipeline = context.add_pipeline::<ColoredVertex>(pipeline_config);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/flat_color_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/flat_color_frag.spv")))
            .with_push_constant(MODEL_COLOR_PUSH_CONSTANT_SIZE)
            .build();
        let flat_color_pipeline = context.add_pipeline::<SimpleVertex>(pipeline_config);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/terrain_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/terrain_frag.spv")))
            .with_vertex_topology(VertexTopology::TriangeStrip)
            .build();
        let terrain_pipeline = context.add_pipeline::<VertexNormal>(pipeline_config);

        let scene = Scene::new(&mut context, main_pipeline, flat_color_pipeline, terrain_pipeline);

        let mut app = VulkrapApplication {
            context,
            mesh_manager,
            scene,
            camera: Camera::new(),
            main_pipeline,
            flat_color_pipeline,
            terrain_pipeline,
            elapsed_time_s: 0.0,
            movement: MovementFlags::ZERO,
        };
        app.create_entities();

        app
    }

    pub fn update(&mut self, delta_time_s: f32) {
        self.elapsed_time_s += delta_time_s;

        self.update_camera(delta_time_s);

        self.scene.update(delta_time_s);
        // TODO MOVE TO scene.update
        self.update_uniform_data(delta_time_s);

        let render_job = self.scene.get_render_job();
        self.context.draw_frame(render_job);
    }

    pub fn handle_mouse_input(&mut self, x_delta: f64, y_delta: f64) {
        self.camera.update_yaw_pitch(x_delta as f32, y_delta as f32);
    }

    pub fn exit(&self) {
        unsafe {
            self.context.wait_idle();
        }
    }

    fn update_uniform_data(&mut self, _delta_time_s: f32) {
        let data = ViewProjectionUniform {
            view: self.camera.get_view_matrix(),
            proj: cgmath::perspective(Deg(60.0), self.context.get_aspect_ratio(), 0.1, 100.0),
        };

        // TODO Should I use a global uniform for VP?
        self.context.update_pipeline_uniform_data(self.main_pipeline, data);
        self.context
            .update_pipeline_uniform_data(self.flat_color_pipeline, data);
        self.context.update_pipeline_uniform_data(self.terrain_pipeline, data);
    }

    fn update_camera(&mut self, delta_time_s: f32) {
        if self.movement.contains(MovementFlags::FORWARD) {
            self.camera.move_forward(delta_time_s);
        }
        if self.movement.contains(MovementFlags::BACKWARD) {
            self.camera.move_backward(delta_time_s);
        }
        if self.movement.contains(MovementFlags::LEFT) {
            self.camera.move_left(delta_time_s);
        }
        if self.movement.contains(MovementFlags::RIGHT) {
            self.camera.move_right(delta_time_s);
        }
        if self.movement.contains(MovementFlags::UP) {
            self.camera.move_up(delta_time_s);
        }
        if self.movement.contains(MovementFlags::DOWN) {
            self.camera.move_down(delta_time_s);
        }
        if !self.movement.is_empty() {
            //self.camera._debug_position();
        }
    }

    fn create_entities(&mut self) {
        let quad1 = WobblyEntity::new(
            Vector3::new(0.0, 0.0, -1.0),
            Quaternion::from_angle_z(Deg(0.0)),
            *self.mesh_manager.get_predefined_mesh(PredefinedMesh::ColoredQuad),
            0.0,
        );

        let quad2 = WobblyEntity::new(
            Vector3::new(0.5, 1.0, -2.0),
            Quaternion::from_angle_z(Deg(0.0)),
            *self.mesh_manager.get_predefined_mesh(PredefinedMesh::ColoredTriangle),
            0.0,
        );

        let triangle = FlatColorEntity::new(
            Vector3::new(-0.5, 1.0, -4.0),
            Quaternion::from_angle_z(Deg(0.0)),
            *self.mesh_manager.get_predefined_mesh(PredefinedMesh::SimpleTriangle),
            Vector3::new(1.0, 0.0, 0.0),
        );

        self.scene.add_wobbly_entity(quad1);
        self.scene.add_wobbly_entity(quad2);
        self.scene.add_flat_color_entity(triangle);
    }

    pub fn handle_keyboard_event(&mut self, key: VirtualKeyCode, state: ElementState) {
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
            _ => {}
        }
    }
}

bitflags! {
    struct MovementFlags: u8 {
        const ZERO = 0;
        const FORWARD = 1 << 0;
        const BACKWARD = 1 << 1;
        const LEFT = 1 << 2;
        const RIGHT = 1 << 3;
        const UP = 1 << 4;
        const DOWN = 1 << 5;
    }
}
