use winit::event::{ElementState, VirtualKeyCode};
use winit::window::Window;

use crate::engine::datatypes::{WindowExtent};

use crate::engine::console::Console;
use crate::engine::cvars::{ConfigVariables, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::engine::mesh::{MeshManager};
use crate::engine::stats;
use crate::engine::ui::hud::Hud;
use crate::renderer::context::Context;
use crate::renderer::types::UniformHandle;

pub trait VulkrapApplication {

    fn update(&mut self, context: &mut Context, delta_time_s: f32);
    fn draw(&mut self, context: &mut Context);

    fn reconfigure(&mut self, config: &ConfigVariables);
    fn handle_mouse_input(&mut self, x_delta: f64, y_delta: f64);
    fn handle_window_resize(&mut self, context: &mut Context, new_extent: WindowExtent);
    fn handle_keyboard_event(&mut self, context: &mut Context, key: VirtualKeyCode, state: ElementState) -> ControlSignal;
}

pub struct EngineParameters<'a> {
    pub mesh_manager: &'a mut MeshManager,
    pub config: &'a mut ConfigVariables,
    pub window_extent: WindowExtent,

    pub hud_vp_uniform: UniformHandle,
}

pub type VulkrapApplicationFactory<T> = fn(context: &mut Context, engine_parameters : EngineParameters) -> T;

pub struct Runtime<T: VulkrapApplication> {
    context: Context,
    config: ConfigVariables,
    console: Console,
    hud: Hud,
    app: T
}

impl<T: VulkrapApplication> Runtime<T> {
    pub fn new(window: &Window, mut config: ConfigVariables, app_factory: VulkrapApplicationFactory<T>) -> Runtime<T> {
        let mut context = Context::new(window);
        let mut mesh_manager = MeshManager::new(&mut context);

        let (window_width, window_height) = context.get_framebuffer_extent();
        let window_extent = WindowExtent::new(window_width, window_height);

        let hud = Hud::new(&mut context, window_extent, &mesh_manager);

        let engine_params = EngineParameters {
            mesh_manager: &mut mesh_manager,
            config: &mut config,
            window_extent,
            hud_vp_uniform: hud.get_vp_uniform()
        };

        let app = app_factory(&mut context, engine_params);

        Runtime {
            context,
            config,
            console: Console::new(),
            hud,
            app,
        }
    }

    pub fn update(&mut self, delta_time_s: f32) {
        self.console.update(delta_time_s);

        self.app.update(&mut self.context, delta_time_s);

        self.context.begin_frame();

        self.app.draw(&mut self.context);
        self.hud.draw(&mut self.context, &self.console);

        let render_stats = self.context.end_frame();
        {
            let mut engine_stats = stats::get();
            engine_stats.update_delta_time(delta_time_s);
            engine_stats.set_render_stats(render_stats);
        }
    }

    pub fn handle_mouse_input(&mut self, x_delta: f64, y_delta: f64) {
        self.app.handle_mouse_input(x_delta, y_delta);
    }

    pub fn handle_window_resize(&mut self, new_extent: WindowExtent) {
        self.config.set(WINDOW_WIDTH, new_extent.width);
        self.config.set(WINDOW_HEIGHT, new_extent.height);

        self.hud.handle_window_resize(&mut self.context, new_extent);
        self.app.handle_window_resize(&mut self.context, new_extent);

        self.context.handle_window_resize();
    }

    pub fn get_configured_extent(&self) -> WindowExtent {
        WindowExtent::new(self.config.get(WINDOW_WIDTH).as_int(), self.config.get(WINDOW_HEIGHT).as_int())
    }

    pub fn exit(&self) {
        unsafe {
            self.context.wait_idle();
        }
    }

    pub fn handle_keyboard_event(&mut self, key: VirtualKeyCode, state: ElementState) -> ControlSignal {
        if self.console.is_active() {
            let control = self.console.handle_keyboard_event(&mut self.config, key, state);

            if self.config.is_dirty() {
                self.reconfigure();
            }
            return control;
        }

        match (key, state) {
            (Console::TOGGLE_BUTTON, ElementState::Pressed) => self.console.toggle(),
            _ => {}
        }

        

        self.app.handle_keyboard_event(&mut self.context, key, state)
    }

    fn reconfigure(&mut self) {
        // TODO: should add some method to config that returns the dirty cvar ids so we dont have to reconfigure everything every time.
        self.app.reconfigure(&self.config);

        self.config.clear_dirty();
    }

}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ControlSignal {
    None,
    Quit,
    ResizeWindow,
}


