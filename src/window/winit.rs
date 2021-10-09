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

pub fn map_input_to_chr(key: VirtualKeyCode, state: ElementState, shift_active : bool) -> Option<char> {
    match (key, state, shift_active) {
        (VirtualKeyCode::Key1, ElementState::Pressed, false) => Some('1'),
        (VirtualKeyCode::Key2, ElementState::Pressed, false) => Some('2'),
        (VirtualKeyCode::Key3, ElementState::Pressed, false) => Some('3'),
        (VirtualKeyCode::Key4, ElementState::Pressed, false) => Some('4'),
        (VirtualKeyCode::Key5, ElementState::Pressed, false) => Some('5'),
        (VirtualKeyCode::Key6, ElementState::Pressed, false) => Some('6'),
        (VirtualKeyCode::Key7, ElementState::Pressed, false) => Some('7'),
        (VirtualKeyCode::Key8, ElementState::Pressed, false) => Some('8'),
        (VirtualKeyCode::Key9, ElementState::Pressed, false) => Some('9'),
        (VirtualKeyCode::Key0, ElementState::Pressed, false) => Some('0'),
        (VirtualKeyCode::Key1, ElementState::Pressed, true) => Some('!'),
        (VirtualKeyCode::Key2, ElementState::Pressed, true) => Some('"'),
        (VirtualKeyCode::Key3, ElementState::Pressed, true) => Some('#'),
        (VirtualKeyCode::Key4, ElementState::Pressed, true) => Some('¤'),
        (VirtualKeyCode::Key5, ElementState::Pressed, true) => Some('%'),
        (VirtualKeyCode::Key6, ElementState::Pressed, true) => Some('&'),
        (VirtualKeyCode::Key7, ElementState::Pressed, true) => Some('/'),
        (VirtualKeyCode::Key8, ElementState::Pressed, true) => Some('('),
        (VirtualKeyCode::Key9, ElementState::Pressed, true) => Some(')'),
        (VirtualKeyCode::Key0, ElementState::Pressed, true) => Some('='),
        (VirtualKeyCode::A, ElementState::Pressed, false) => Some('a'),
        (VirtualKeyCode::B, ElementState::Pressed, false) => Some('b'),
        (VirtualKeyCode::C, ElementState::Pressed, false) => Some('c'),
        (VirtualKeyCode::D, ElementState::Pressed, false) => Some('d'),
        (VirtualKeyCode::E, ElementState::Pressed, false) => Some('e'),
        (VirtualKeyCode::F, ElementState::Pressed, false) => Some('f'),
        (VirtualKeyCode::G, ElementState::Pressed, false) => Some('g'),
        (VirtualKeyCode::H, ElementState::Pressed, false) => Some('h'),
        (VirtualKeyCode::I, ElementState::Pressed, false) => Some('i'),
        (VirtualKeyCode::J, ElementState::Pressed, false) => Some('j'),
        (VirtualKeyCode::K, ElementState::Pressed, false) => Some('k'),
        (VirtualKeyCode::L, ElementState::Pressed, false) => Some('l'),
        (VirtualKeyCode::M, ElementState::Pressed, false) => Some('m'),
        (VirtualKeyCode::N, ElementState::Pressed, false) => Some('n'),
        (VirtualKeyCode::O, ElementState::Pressed, false) => Some('o'),
        (VirtualKeyCode::P, ElementState::Pressed, false) => Some('p'),
        (VirtualKeyCode::Q, ElementState::Pressed, false) => Some('q'),
        (VirtualKeyCode::R, ElementState::Pressed, false) => Some('r'),
        (VirtualKeyCode::S, ElementState::Pressed, false) => Some('s'),
        (VirtualKeyCode::T, ElementState::Pressed, false) => Some('t'),
        (VirtualKeyCode::U, ElementState::Pressed, false) => Some('u'),
        (VirtualKeyCode::V, ElementState::Pressed, false) => Some('v'),
        (VirtualKeyCode::W, ElementState::Pressed, false) => Some('w'),
        (VirtualKeyCode::X, ElementState::Pressed, false) => Some('x'),
        (VirtualKeyCode::Y, ElementState::Pressed, false) => Some('y'),
        (VirtualKeyCode::Z, ElementState::Pressed, false) => Some('z'),
        (VirtualKeyCode::A, ElementState::Pressed, true) => Some('A'),
        (VirtualKeyCode::B, ElementState::Pressed, true) => Some('B'),
        (VirtualKeyCode::C, ElementState::Pressed, true) => Some('C'),
        (VirtualKeyCode::D, ElementState::Pressed, true) => Some('D'),
        (VirtualKeyCode::E, ElementState::Pressed, true) => Some('E'),
        (VirtualKeyCode::F, ElementState::Pressed, true) => Some('F'),
        (VirtualKeyCode::G, ElementState::Pressed, true) => Some('G'),
        (VirtualKeyCode::H, ElementState::Pressed, true) => Some('H'),
        (VirtualKeyCode::I, ElementState::Pressed, true) => Some('I'),
        (VirtualKeyCode::J, ElementState::Pressed, true) => Some('J'),
        (VirtualKeyCode::K, ElementState::Pressed, true) => Some('K'),
        (VirtualKeyCode::L, ElementState::Pressed, true) => Some('L'),
        (VirtualKeyCode::M, ElementState::Pressed, true) => Some('M'),
        (VirtualKeyCode::N, ElementState::Pressed, true) => Some('N'),
        (VirtualKeyCode::O, ElementState::Pressed, true) => Some('O'),
        (VirtualKeyCode::P, ElementState::Pressed, true) => Some('P'),
        (VirtualKeyCode::Q, ElementState::Pressed, true) => Some('Q'),
        (VirtualKeyCode::R, ElementState::Pressed, true) => Some('R'),
        (VirtualKeyCode::S, ElementState::Pressed, true) => Some('S'),
        (VirtualKeyCode::T, ElementState::Pressed, true) => Some('T'),
        (VirtualKeyCode::U, ElementState::Pressed, true) => Some('U'),
        (VirtualKeyCode::V, ElementState::Pressed, true) => Some('V'),
        (VirtualKeyCode::W, ElementState::Pressed, true) => Some('W'),
        (VirtualKeyCode::X, ElementState::Pressed, true) => Some('X'),
        (VirtualKeyCode::Y, ElementState::Pressed, true) => Some('Y'),
        (VirtualKeyCode::Z, ElementState::Pressed, true) => Some('Z'),

        (VirtualKeyCode::Minus, ElementState::Pressed, false) => Some('-'),
        (VirtualKeyCode::Minus, ElementState::Pressed, true) => Some('_'),

        (VirtualKeyCode::Space, ElementState::Pressed, false) => Some(' '),

        _ => None,
    }
}
