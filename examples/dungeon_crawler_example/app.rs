use cgmath::{Vector2, Vector4};
use winit::event::{ElementState, VirtualKeyCode};
use vulkrap::engine::camera::Camera;
use vulkrap::engine::cvars::{ConfigVariables, FOV};
use vulkrap::engine::datatypes::WindowExtent;
use vulkrap::engine::runtime::{ControlSignal, EngineParameters, VulkrapApplication};
use vulkrap::engine::ui::widgets::TexturedQuadRenderer;
use vulkrap::renderer::context::Context;
use crate::dungeon_crawler_example::movement::{Movement, MovementInput, Orientation};
use crate::dungeon_crawler_example::scene::Scene;


pub struct DungeonCrawler {
    scene: Scene,
    camera: Camera,
    movement: Movement,
    main: TexturedQuadRenderer,
}

impl VulkrapApplication for DungeonCrawler {
    fn update(&mut self, context: &mut Context, delta_time_s: f32) {
        self.movement.update(delta_time_s);
        self.movement.update_camera(context, &mut self.camera);

        self.scene.update(context, delta_time_s);
    }

    fn draw(&mut self, context: &mut Context) {
        self.scene.draw(context, &self.movement);
        self.main.draw(context);
    }

    fn reconfigure(&mut self, config: &ConfigVariables) {
        self.camera.reconfigure(config);
        self.scene.reconfigure(config);
    }

    fn handle_mouse_input(&mut self, _x_delta: f64, _y_delta: f64) {
    }

    fn handle_window_resize(&mut self, _context: &mut Context, new_size: WindowExtent) {
        self.main.set(
            Vector2::new((new_size.width / 2) as f32, (new_size.height / 2) as f32),
            Vector2::new(new_size.width as f32, new_size.height as f32),
            Vector4::new(1.0, 1.0, 1.0, 1.0));
    }

    fn handle_keyboard_event(&mut self, _context: &mut Context, key: VirtualKeyCode, state: ElementState) -> ControlSignal {

        match (key, state) {
            (VirtualKeyCode::W, ElementState::Pressed) => self.movement.add_input(MovementInput::Forward),
            (VirtualKeyCode::S, ElementState::Pressed) => self.movement.add_input(MovementInput::Backward),
            (VirtualKeyCode::A, ElementState::Pressed) => self.movement.add_input(MovementInput::Left),
            (VirtualKeyCode::D, ElementState::Pressed) => self.movement.add_input(MovementInput::Right),
            (VirtualKeyCode::Q, ElementState::Pressed) => self.movement.add_input(MovementInput::RotateLeft),
            (VirtualKeyCode::E, ElementState::Pressed) => self.movement.add_input(MovementInput::RotateRight),

            _ => {}
        }


        ControlSignal::None
    }
}

impl DungeonCrawler {
    pub fn new(context: &mut Context, engine_params: EngineParameters) -> DungeonCrawler {
        let start_position = Vector2::new(2, 6);

        engine_params.config.set(FOV, 75);
        let mut camera = Camera::new(context, engine_params.config);
        camera.set_pitch(-0.1); /* Slight pitch downward */

        let scene = Scene::new(context, engine_params.mesh_manager, &camera);
        let movement = Movement::new(start_position, Orientation::North);
        movement.update_camera(context, &mut camera);

        let sampler = context.add_sampler();
        let mut texture_quad_renderer = TexturedQuadRenderer::new(
            context,
            engine_params.hud_vp_uniform,
            engine_params.mesh_manager,
            scene.get_target_texture(),
            sampler);

        texture_quad_renderer.set(
            Vector2::new((engine_params.window_extent.width / 2) as f32, (engine_params.window_extent.height / 2) as f32),
            Vector2::new(engine_params.window_extent.width as f32, engine_params.window_extent.height as f32),
            Vector4::new(1.0, 1.0, 1.0, 1.0));

        DungeonCrawler {
            scene,
            camera,
            movement,
            main: texture_quad_renderer,
        }
    }
}

