#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate memoffset;

#[macro_export]
macro_rules! log_debug {
    () => (crate::log::logger::debug(""));
    ($($arg:tt)*) => ({
        crate::log::logger::debug(format!($($arg)*).as_str());
    })
}

#[macro_export]
macro_rules! log_debug_once {
    () => (crate::log::logger::debug(""));
    ($($arg:tt)*) => ({
        crate::log::logger::debug_once(format!($($arg)*).as_str());
    })
}
#[macro_export]
macro_rules! log_info {
    () => (crate::log::logger::info(""));
    ($($arg:tt)*) => ({
        crate::log::logger::info(format!($($arg)*).as_str());
    })
}
#[macro_export]
macro_rules! log_warning {
    () => (crate::log::logger::warning(""));
    ($($arg:tt)*) => ({
        crate::log::logger::warning(format!($($arg)*).as_str());
    })
}
#[macro_export]
macro_rules! log_error {
    () => (crate::log::logger::error(""));
    ($($arg:tt)*) => ({
        crate::log::logger::error(format!($($arg)*).as_str());
    })
}

use crate::engine::game::VulkrapApplication;
use winit::event_loop::EventLoop;

mod engine;
mod log;
mod renderer;
mod util;
mod window;

const ENGINE_NAME: &str = "vulkrap";
const APPLICATION_VERSION: (u32, u32, u32) = (1, 0, 0);
const ENGINE_VERSION: (u32, u32, u32) = (0, 0, 1);

const WINDOW_TITLE: &str = "vulkrap";
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

fn main() {
    log_info!("vulkrap init...");

    let event_loop = EventLoop::new();
    let window = window::winit::init_window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, &event_loop);
    window.set_cursor_visible(false);

    let vulkan_app = VulkrapApplication::new(&window);
    window::winit::main_loop(event_loop, window, vulkan_app);

    log_info!("Exiting");
}
