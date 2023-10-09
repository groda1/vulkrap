use cgmath::Vector2;
use winit::event::{ElementState, VirtualKeyCode};
use vulkrap::engine::camera::Camera;
use vulkrap::engine::cvars::ConfigVariables;
use vulkrap::engine::datatypes::WindowExtent;
use vulkrap::engine::runtime::{ControlSignal, EngineParameters, VulkrapApplication};
use vulkrap::renderer::context::Context;
use crate::dungeon_crawler_example::movement::{Movement, MovementInput, Orientation};
use crate::dungeon_crawler_example::scene::Scene;



pub struct DungeonCrawler {
    scene: Scene,
    camera: Camera,

    movement: Movement,

}

impl VulkrapApplication for DungeonCrawler {
    fn update(&mut self, context: &mut Context, delta_time_s: f32) {


        self.movement.update(delta_time_s);
        self.movement.update_camera(context, &mut self.camera);

        self.scene.update(context, delta_time_s);
    }

    fn draw(&mut self, context: &mut Context) {
        self.scene.draw(context);
    }

    fn reconfigure(&mut self, config: &ConfigVariables) {
        self.camera.reconfigure(config);
        self.scene.reconfigure(config);
    }

    fn handle_mouse_input(&mut self, _x_delta: f64, _y_delta: f64) {
        //self.camera.update_yaw_pitch(x_delta as f32, y_delta as f32);
    }

    fn handle_window_resize(&mut self, _context: &mut Context, _new_size: WindowExtent) {}

    fn handle_keyboard_event(&mut self, _context: &mut Context, key: VirtualKeyCode, state: ElementState) -> ControlSignal {

        match (key, state) {
            (VirtualKeyCode::W, ElementState::Pressed) => self.movement.add_input(MovementInput::Forward),
            (VirtualKeyCode::S, ElementState::Pressed) => self.movement.add_input(MovementInput::Backward),
            (VirtualKeyCode::A, ElementState::Pressed) => self.movement.add_input(MovementInput::Left),
            (VirtualKeyCode::D, ElementState::Pressed) => self.movement.add_input(MovementInput::Right),
            (VirtualKeyCode::Q, ElementState::Pressed) => self.movement.add_input(MovementInput::RotateLeft),
            (VirtualKeyCode::E, ElementState::Pressed) => self.movement.add_input(MovementInput::RotateRight),
            //(VirtualKeyCode::Space, ElementState::Pressed) => self.movement.insert(MovementFlags::UP),
            //(VirtualKeyCode::C, ElementState::Pressed) => self.movement.insert(MovementFlags::DOWN),

            _ => {}
        }


        ControlSignal::None
    }
}

impl DungeonCrawler {
    pub fn new(context: &mut Context, engine_params: EngineParameters) -> DungeonCrawler {
        let start_position = Vector2::new(2, 6);
        let mut camera = Camera::new(context, engine_params.config);

        let scene = Scene::new(context, engine_params.mesh_manager, &camera);

        let movement = Movement::new(start_position, Orientation::North);
        movement.update_camera(context, &mut camera);

        DungeonCrawler {
            scene,
            camera,
            movement,
        }
    }
}

