#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate memoffset;

#[macro_export]
macro_rules! log_info {
    () => (crate::console::logger::info(""));
    ($($arg:tt)*) => ({
        crate::console::logger::info(format!($($arg)*).as_str());
    })
}
#[macro_export]
macro_rules! log_warning {
    () => (crate::console::logger::warning(""));
    ($($arg:tt)*) => ({
        crate::console::logger::warning(format!($($arg)*).as_str());
    })
}
#[macro_export]
macro_rules! log_debug {
    () => (crate::console::logger::debug(""));
    ($($arg:tt)*) => ({
        crate::console::logger::debug(format!($($arg)*).as_str());
    })
}

use crate::game::game::VulkrapApplication;
use winit::event_loop::EventLoop;

mod console;
mod game;
mod renderer;
mod util;
mod window;

const ENGINE_NAME: &'static str = "vulkrap";
const APPLICATION_VERSION: (u32, u32, u32) = (1, 0, 0);
const ENGINE_VERSION: (u32, u32, u32) = (1, 0, 0);

const WINDOW_TITLE: &'static str = "vulkrap";
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

fn main() {
    log_info!("vulkrap init...");

    let event_loop = EventLoop::new();
    let window = window::winit::init_window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, &event_loop);

    let vulkan_app = VulkrapApplication::new(&window);
    window::winit::main_loop(event_loop, window, vulkan_app);

    log_info!("Exiting");
}
