use winit::event::{ElementState, VirtualKeyCode};
use crate::engine::cvars::ConfigVariables;

const TOGGLE_SPEED: f32 = 7.5;
const CARET_BLINK_SPEED: f32 = 1.5;

pub struct Console {
    history: Vec<HistoryLine>,

    active: bool,
    input_buffer: Vec<char>,
    input_index: u32,

    current_draw_offset: f32,

    caret_visible: bool,
    caret_delta: f32,
}

impl Console {
    pub const TOGGLE_BUTTON: VirtualKeyCode = VirtualKeyCode::F8;

    pub fn new() -> Console {
        Console {
            history: Vec::new(),
            active: false,
            input_buffer: Vec::new(),
            input_index: 0,
            current_draw_offset: 1.0,

            caret_visible: false,
            caret_delta: 0.0,
        }
    }

    pub fn handle_keyboard_event(&mut self, cfg : &ConfigVariables, key: VirtualKeyCode, state: ElementState) {
        let char_inut = match (key, state) {
            (VirtualKeyCode::Key1, ElementState::Pressed) => Some('1'),
            (VirtualKeyCode::Key2, ElementState::Pressed) => Some('2'),
            (VirtualKeyCode::Key3, ElementState::Pressed) => Some('3'),
            (VirtualKeyCode::Key4, ElementState::Pressed) => Some('4'),
            (VirtualKeyCode::Key5, ElementState::Pressed) => Some('5'),
            (VirtualKeyCode::Key6, ElementState::Pressed) => Some('6'),
            (VirtualKeyCode::Key7, ElementState::Pressed) => Some('7'),
            (VirtualKeyCode::Key8, ElementState::Pressed) => Some('8'),
            (VirtualKeyCode::Key9, ElementState::Pressed) => Some('9'),
            (VirtualKeyCode::Key0, ElementState::Pressed) => Some('0'),
            (VirtualKeyCode::A, ElementState::Pressed) => Some('a'),
            (VirtualKeyCode::B, ElementState::Pressed) => Some('b'),
            (VirtualKeyCode::C, ElementState::Pressed) => Some('c'),
            (VirtualKeyCode::D, ElementState::Pressed) => Some('d'),
            (VirtualKeyCode::E, ElementState::Pressed) => Some('e'),
            (VirtualKeyCode::F, ElementState::Pressed) => Some('f'),
            (VirtualKeyCode::G, ElementState::Pressed) => Some('g'),
            (VirtualKeyCode::H, ElementState::Pressed) => Some('h'),
            (VirtualKeyCode::I, ElementState::Pressed) => Some('i'),
            (VirtualKeyCode::J, ElementState::Pressed) => Some('j'),
            (VirtualKeyCode::K, ElementState::Pressed) => Some('k'),
            (VirtualKeyCode::L, ElementState::Pressed) => Some('l'),
            (VirtualKeyCode::M, ElementState::Pressed) => Some('m'),
            (VirtualKeyCode::N, ElementState::Pressed) => Some('n'),
            (VirtualKeyCode::O, ElementState::Pressed) => Some('o'),
            (VirtualKeyCode::P, ElementState::Pressed) => Some('p'),
            (VirtualKeyCode::Q, ElementState::Pressed) => Some('q'),
            (VirtualKeyCode::R, ElementState::Pressed) => Some('r'),
            (VirtualKeyCode::S, ElementState::Pressed) => Some('s'),
            (VirtualKeyCode::T, ElementState::Pressed) => Some('t'),
            (VirtualKeyCode::U, ElementState::Pressed) => Some('u'),
            (VirtualKeyCode::V, ElementState::Pressed) => Some('v'),
            (VirtualKeyCode::W, ElementState::Pressed) => Some('w'),
            (VirtualKeyCode::X, ElementState::Pressed) => Some('x'),
            (VirtualKeyCode::Y, ElementState::Pressed) => Some('y'),
            (VirtualKeyCode::Z, ElementState::Pressed) => Some('z'),
            _ => None,
        };

        if let Some(x) = char_inut {
            self.input_buffer.push(x);
            self.input_index += 1;
            self._reset_caret();
        }

        match (key, state) {
            (VirtualKeyCode::Back, ElementState::Pressed) => {
                self.input_buffer.pop();
                self.input_index -= 1;
                self._reset_caret();
            }
            (VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter, ElementState::Pressed) => {
                self._handle_input(cfg);
            }
            (Console::TOGGLE_BUTTON, ElementState::Pressed) => {
                self.toggle();
            }

            _ => {
                //println!("key {:?}", key)
            }
        }
    }

    pub fn update(&mut self, delta_time_s: f32) {
        if self.active && self.current_draw_offset > 0.0 {
            self.current_draw_offset -= TOGGLE_SPEED * delta_time_s;

            if self.current_draw_offset < 0.0 {
                self.current_draw_offset = 0.0;
            }
        } else if !self.active && self.current_draw_offset <= 1.0 {
            self.current_draw_offset += TOGGLE_SPEED * delta_time_s;

            if self.current_draw_offset >= 1.0 {
                self._clear_input_buffer();
            }
        }

        if self.active {
            self.caret_delta += CARET_BLINK_SPEED * delta_time_s;
            if self.caret_delta >= 1.0 {
                self.caret_visible = !self.caret_visible;
                self.caret_delta -= 1.0;
            }
        }
    }

    pub fn get_current_input(&self) -> String {
        self.input_buffer.iter().collect()
    }

    pub fn get_current_y_offset(&self) -> f32 {
        self.current_draw_offset
    }

    pub fn is_caret_visible(&self) -> bool {
        self.caret_visible
    }

    pub fn get_input_index(&self) -> u32 {
        self.input_index
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_visible(&self) -> bool {
        self.current_draw_offset < 1.0
    }

    pub fn error(&mut self, message: String) {
        self.history.push(HistoryLine { line_type: LineType::Error, line: message });
    }

    pub fn input(&mut self, message: String) {
        self.history.push(HistoryLine { line_type: LineType::Input, line: message });
    }

    pub fn output(&mut self, message: String) {
        self.history.push(HistoryLine { line_type: LineType::Output, line: message });
    }

    pub fn cvar(&mut self, message: String) {
        self.history.push(HistoryLine { line_type: LineType::Cvar, line: message });
    }


    pub fn get_history(&self, line_count : usize) -> &[HistoryLine] {
        &self.history[self.history.len() - line_count.min(self.history.len())..]
    }

    fn _reset_caret(&mut self) {
        self.caret_visible = true;
        self.caret_delta = 0.0;
    }

    fn _handle_input(&mut self, cfg : &ConfigVariables) {
        if self.input_buffer.is_empty() {
            return;
        }
        self.input(self.get_current_input());

        let input = self.get_current_input();

        let split: Vec<&str> = input.split(" ").collect();
        let cvar_opt =  cfg.get_cvar_id_from_str(split[0]);

        if let Some(cvar) = cvar_opt {
            self.cvar(cfg.get_desc(*cvar));
        } else {
            self.error(format!("unknown command or cvar: {}", self.get_current_input()));
        }

        self.input_index = 0;
        self.input_buffer.clear();
        self._reset_caret();
    }

    fn _clear_input_buffer(&mut self) {
        self.input_buffer.clear();
        self.input_index = 0;
    }
}

pub struct HistoryLine {
    pub line_type : LineType,
    pub line: String
}

pub enum LineType {
    Input,
    Output,
    Info,
    Warning,
    Error,
    Cvar,
}
