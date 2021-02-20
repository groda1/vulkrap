#[macro_use]
extern crate lazy_static;

use winit::event_loop::{EventLoop};

mod console;

use console::logger;

mod window;
mod vulkan;

const ENGINE_NAME: &'static str = "cvulkan";


const APPLICATION_VERSION: (u32, u32, u32) = (1, 0, 0);
const ENGINE_VERSION: (u32, u32, u32) = (1, 0, 0);


const WINDOW_TITLE: &'static str = "cvulkan test";
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

fn main() {
    logger::log_info("cvulkan init...");

    let event_loop = EventLoop::new();
    let _window = window::winit::init_window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, &event_loop);
    let mut _vulkan_context = vulkan::context::Context::new();

    window::winit::main_loop(event_loop, _window, _vulkan_context);

    logger::log_info("Exiting");
}
