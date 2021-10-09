use std::path::Path;

use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use winit::event::{ElementState, VirtualKeyCode};
use winit::window::Window;

use crate::engine::camera::Camera;
use crate::engine::datatypes::{ColoredVertex, ModelWoblyPushConstant, VertexNormal};
use crate::engine::entity::WobblyEntity;

use crate::engine::console::Console;
use crate::engine::cvars::{ConfigVariables};
use crate::engine::mesh::{MeshManager, PredefinedMesh};
use crate::engine::renderstats;
use crate::engine::scene::Scene;
use crate::renderer::context::{Context, UniformHandle};
use crate::renderer::pipeline::{PipelineConfiguration, VertexTopology};
use crate::renderer::uniform::UniformStage;
use crate::util::file;

pub struct VulkrapApplication {
    context: Context,
    config: ConfigVariables,
    mesh_manager: MeshManager,
    scene: Scene,

    camera: Camera,
    flags_uniform: UniformHandle,
    movement: MovementFlags,

    console: Console,

    draw_wireframe: bool,
}

impl VulkrapApplication {
    pub fn new(window: &Window) -> VulkrapApplication {
        let mut context = Context::new(window);
        let config = ConfigVariables::new();

        let mesh_manager = MeshManager::new(&mut context);
        let camera = Camera::new(&mut context, &config);

        let flags_uniform = context.create_uniform::<u32>(UniformStage::Fragment);
        context.set_uniform_data(flags_uniform, 0_u32);

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
        let wobbly_pipeline = context.add_pipeline::<ColoredVertex>(pipeline_config);

        let pipeline_config = PipelineConfiguration::builder()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/terrain_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/terrain_frag.spv")))
            .with_vertex_topology(VertexTopology::TriangeStrip)
            .with_vertex_uniform(0, camera.get_uniform())
            .with_fragment_uniform(1, flags_uniform)
            .build();
        let terrain_pipeline = context.add_pipeline::<VertexNormal>(pipeline_config);

        let scene = Scene::new(&mut context, &mesh_manager, wobbly_pipeline, terrain_pipeline);

        let mut app = VulkrapApplication {
            context,
            config,
            mesh_manager,
            scene,
            camera,
            flags_uniform,
            console: Console::new(),
            movement: MovementFlags::ZERO,
            draw_wireframe: false,
        };
        app.create_entities();

        app
    }

    pub fn update(&mut self, delta_time_s: f32) {
        renderstats::update_delta_time(delta_time_s);

        self.camera.update(&mut self.context, self.movement, delta_time_s);
        self.scene.update(delta_time_s);
        self.console.update(delta_time_s);

        let render_job = self.scene.fetch_render_job(&self.console);
        self.context.draw_frame(render_job);
    }

    pub fn handle_mouse_input(&mut self, x_delta: f64, y_delta: f64) {
        self.camera.update_yaw_pitch(x_delta as f32, y_delta as f32);
    }

    pub fn handle_window_resize(&mut self, width: u32, height: u32) {
        self.scene.handle_window_resize(&mut self.context, width, height);
        self.context.handle_window_resize();
    }

    pub fn exit(&self) {
        unsafe {
            self.context.wait_idle();
        }
    }

    fn toggle_wireframe(&mut self) {
        self.draw_wireframe = !self.draw_wireframe;
        self.context
            .set_uniform_data(self.flags_uniform, self.draw_wireframe as u32);
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

        self.scene.add_wobbly_entity(quad1);
        self.scene.add_wobbly_entity(quad2);
    }

    pub fn handle_keyboard_event(&mut self, key: VirtualKeyCode, state: ElementState) {
        if self.console.is_active() {
            self.console.handle_keyboard_event(&mut self.config, key, state);

            if self.config.is_dirty() {
                self.reconfigure();
                self.config.clear_dirty();
            }
            return;
        }

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
            (VirtualKeyCode::F2, ElementState::Pressed) => self.toggle_wireframe(),

            (Console::TOGGLE_BUTTON, ElementState::Pressed) => self.console.toggle(),

            _ => {}
        }
    }

    fn reconfigure(&mut self) {
        self.camera.reconfigure(&self.config);
    }
}

bitflags! {
    pub struct MovementFlags: u8 {
        const ZERO = 0;
        const FORWARD = 1 << 0;
        const BACKWARD = 1 << 1;
        const LEFT = 1 << 2;
        const RIGHT = 1 << 3;
        const UP = 1 << 4;
        const DOWN = 1 << 5;
    }
}
