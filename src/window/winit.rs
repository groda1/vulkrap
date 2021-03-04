use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::renderer::context::Context;

pub fn init_window(
    title: &'static str,
    width: u32,
    height: u32,
    event_loop: &EventLoop<()>,
) -> Window {
    winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(event_loop)
        .expect("Failed to create window.")
}

pub fn main_loop(event_loop: EventLoop<()>, window: Window, mut rendering_context: Context) {
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                            *control_flow = ControlFlow::Exit
                        }
                        _ => {}
                    },
                },
                _ => {}
            },
            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::MouseMotion { delta: _ } => {
                        //println!("Mouse move {} {}", delta.0, delta.1)
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                rendering_context.draw_frame();
                //thread::sleep(time::Duration::from_millis(10));
            }
            Event::LoopDestroyed => unsafe {
                rendering_context.wait_idle();
            },
            _ => (),
        }
    })
}
