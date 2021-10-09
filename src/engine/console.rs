use crate::engine::cvars::ConfigVariables;
use std::num::ParseFloatError;
use winit::event::{ElementState, VirtualKeyCode};

const TOGGLE_SPEED: f32 = 7.5;
const CARET_BLINK_SPEED: f32 = 1.5;

pub struct Console {
    history: Vec<HistoryLine>,
    input_history: Vec<String>,
    input_history_index : usize,

    active: bool,
    input_buffer: Vec<char>,
    input_index: u32,

    current_draw_offset: f32,

    shift_active: bool,

    caret_visible: bool,
    caret_delta: f32,
}

impl Console {
    pub const TOGGLE_BUTTON: VirtualKeyCode = VirtualKeyCode::F1;

    pub fn new() -> Console {
        Console {
            history: Vec::new(),
            input_history: Vec::new(),
            input_history_index: 0,

            active: false,
            input_buffer: Vec::new(),
            input_index: 0,
            current_draw_offset: 1.0,

            caret_visible: false,
            caret_delta: 0.0,
            shift_active: false,
        }
    }

    pub fn handle_keyboard_event(&mut self, cfg: &mut ConfigVariables, key: VirtualKeyCode, state: ElementState) {
        let char_inut = match (key, state, self.shift_active) {
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
        };

        if let Some(x) = char_inut {
            self.input_buffer.push(x);
            self.input_index += 1;
            self._reset_caret();
        }

        match (key, state) {
            (VirtualKeyCode::Back, ElementState::Pressed) => {
                if self.input_index > 0 {
                    self.input_buffer.pop();
                    self.input_index -= 1;
                    self._reset_caret();
                }
            }
            (VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter, ElementState::Pressed) => {
                self._handle_input(cfg);
            }
            (VirtualKeyCode::RShift | VirtualKeyCode::LShift, ElementState::Pressed) => {
                self.shift_active = true;
            }
            (VirtualKeyCode::RShift | VirtualKeyCode::LShift, ElementState::Released) => {
                self.shift_active = false;
            }
            (VirtualKeyCode::Up, ElementState::Pressed) => {
                self._handle_up();
            }
            (VirtualKeyCode::Down, ElementState::Pressed) => {
                self._handle_down();
            }
            (Console::TOGGLE_BUTTON, ElementState::Pressed) => {
                self.toggle();
            }

            _ => {
                println!("key {:?}", key)
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
        self.history.push(HistoryLine {
            line_type: LineType::Error,
            line: message,
        });
    }

    pub fn input(&mut self, message: String) {
        self.history.push(HistoryLine {
            line_type: LineType::Input,
            line: message,
        });
    }

    pub fn output(&mut self, message: String) {
        self.history.push(HistoryLine {
            line_type: LineType::Output,
            line: message,
        });
    }

    pub fn cvar(&mut self, message: String) {
        self.history.push(HistoryLine {
            line_type: LineType::Cvar,
            line: message,
        });
    }

    pub fn get_history(&self, line_count: usize) -> &[HistoryLine] {
        &self.history[self.history.len() - line_count.min(self.history.len())..]
    }

    fn _reset_caret(&mut self) {
        self.caret_visible = true;
        self.caret_delta = 0.0;
    }

    fn _handle_input(&mut self, cfg: &mut ConfigVariables) {
        if self.input_buffer.is_empty() {
            return;
        }
        self.input(self.get_current_input());
        let input = self.get_current_input();

        let split: Vec<&str> = input.split(" ").collect();
        let cvar_opt = cfg.get_cvar_id_from_str(split[0]);

        if let Some(cvar) = cvar_opt {
            let cvar_id = *cvar;
            if split.len() >= 2 {
                let parsed_arg = split[1].parse::<f32>();
                if let Ok(arg) = parsed_arg {
                    cfg.set(cvar_id, arg);
                } else {
                    self.error(format!("failed to parse cvar argument: {}", split[1]));
                }
            }
            self.cvar(cfg.get_desc(cvar_id));
        } else {
            self.error(format!("unknown command or cvar: {}", self.get_current_input()));
        }

        if self.input_history_index > 0 {
            self.input_history.pop();
            self.input_history_index = 0;
        }
        self.input_history.push(self.input_buffer.iter().collect());
        self.input_index = 0;
        self.input_buffer.clear();
        self._reset_caret();
    }

    fn _clear_input_buffer(&mut self) {
        self.input_buffer.clear();
        self.input_index = 0;
    }

    fn _handle_up(&mut self) {
        if self.input_history_index == 0 {
            self.input_history.push(self.input_buffer.iter().collect())
        }

        self.input_history_index += 1;

        if self.input_history.len() > self.input_history_index {
            self.input_buffer = self.input_history[self.input_history.len() - self.input_history_index - 1].chars().collect();
        } else {
            self.input_history_index = 0;
            self.input_buffer = self.input_history.pop().unwrap().chars().collect();
        }
        self.input_index = self.input_buffer.len() as u32;
    }

    fn _handle_down(&mut self) {
        if self.input_history_index == 0 {
            return;
        }
        self.input_history_index -= 1;

        if self.input_history_index == 0 {
            self.input_buffer = self.input_history.pop().unwrap().chars().collect()
        } else {
            self.input_buffer = self.input_history[self.input_history.len() - self.input_history_index- 1].chars().collect();
        }
        self.input_index = self.input_buffer.len() as u32;
    }
}

pub struct HistoryLine {
    pub line_type: LineType,
    pub line: String,
}

pub enum LineType {
    Input,
    Output,
    Info,
    Warning,
    Error,
    Cvar,
}
