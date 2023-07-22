#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate memoffset;

#[macro_export]
macro_rules! log_debug {
    () => ($crate::log::logger::debug(""));
    ($($arg:tt)*) => ({
        $crate::log::logger::debug(format!($($arg)*).as_str());
    })
}

#[macro_export]
macro_rules! log_khronos {
    () => ($crate::log::logger::khronos(""));
    ($($arg:tt)*) => ({
        $crate::log::logger::khronos(format!($($arg)*).as_str());
    })
}

#[macro_export]
macro_rules! log_debug_once {
    () => ($crate::log::logger::debug_once(""));
    ($($arg:tt)*) => ({
        $crate::log::logger::debug_once(format!($($arg)*).as_str());
    })
}
#[macro_export]
macro_rules! log_info {
    () => ($crate::log::logger::info(""));
    ($($arg:tt)*) => ({
        $crate::log::logger::info(format!($($arg)*).as_str());
    })
}
#[macro_export]
macro_rules! log_warning {
    () => ($crate::log::logger::warning(""));
    ($($arg:tt)*) => ({
        $crate::log::logger::warning(format!($($arg)*).as_str());
    })
}
#[macro_export]
macro_rules! log_error {
    () => ($crate::log::logger::error(""));
    ($($arg:tt)*) => ({
        $crate::log::logger::error(format!($($arg)*).as_str());
    })
}

use winit::event_loop::EventLoop;
use crate::engine::cvars::{ConfigVariables};
use crate::engine::runtime::{Runtime, VulkrapApplication, VulkrapApplicationFactory};

pub mod renderer;
pub mod engine;
pub mod util;
pub mod log;

mod window;

const ENGINE_NAME: &str = "vulkrap";
const APPLICATION_VERSION: (u32, u32, u32) = (1, 0, 0);
const ENGINE_VERSION: (u32, u32, u32) = (0, 0, 1);


pub fn vulkrap_start<T: VulkrapApplication + 'static>(window_title: &'static str,
                                                      window_width: u32,
                                                      window_height: u32,
                                                      app_factory: VulkrapApplicationFactory<T>) {

    log_info!("vulkrap init...");

    let event_loop = EventLoop::new();
    let window = window::winit::init_window(window_title, window_width, window_height, &event_loop);
    window.set_cursor_visible(false);

    let config = ConfigVariables::new();

    let vulkrap_runtime = Runtime::new(&window, config, app_factory);

    window::winit::main_loop(event_loop, window, vulkrap_runtime);

    log_info!("Exiting");
}
