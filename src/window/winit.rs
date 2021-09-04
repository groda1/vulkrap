use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::engine::game::VulkrapApplication;
use crate::util::frametimer::FrameTimer;

pub fn init_window(title: &'static str, width: u32, height: u32, event_loop: &EventLoop<()>) -> Window {
    winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(event_loop)
        .expect("Failed to create window.")
}

pub fn main_loop(event_loop: EventLoop<()>, window: Window, mut vulkrap_app: VulkrapApplication) {
    let mut frame_timer = FrameTimer::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    virtual_keycode, state, ..
                } => match (virtual_keycode, state) {
                    (Some(VirtualKeyCode::Escape), ElementState::Pressed) => *control_flow = ControlFlow::Exit,
                    (Some(key), state) => vulkrap_app.handle_keyboard_event(key, state),
                    _ => {}
                },
            },
            WindowEvent::Resized(new_size) => {
                vulkrap_app.handle_window_resize(new_size.width, new_size.height);
            }
            _ => {}
        },
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => {
                vulkrap_app.handle_mouse_input(delta.0, delta.1);
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::RedrawRequested(_window_id) => {
            vulkrap_app.update(frame_timer.delta_time_sec());
            frame_timer.tick_frame();
        }
        Event::LoopDestroyed => {
            vulkrap_app.exit();
        }
        _ => (),
    })
}
