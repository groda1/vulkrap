use crate::engine::cvars::{ConfigVariables, CvarType};
use winit::event::{ElementState, VirtualKeyCode};
use crate::log::logger;

const TOGGLE_SPEED: f32 = 7.5;
const CARET_BLINK_SPEED: f32 = 1.5;

const SCROLL_LINES :usize = 15;

pub struct Console {
    input_history: Vec<String>,
    input_history_index: usize,

    scroll : usize,

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
            input_history: Vec::new(),
            input_history_index: 0,

            scroll : 0,

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
        let char_inut = crate::window::winit::map_input_to_chr(key, state, self.shift_active);

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
            (VirtualKeyCode::PageUp, ElementState::Pressed) => {
                self._scroll_up();
            }
            (VirtualKeyCode::PageDown, ElementState::Pressed) => {
                self._scroll_down();
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
        self.scroll = 0;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_visible(&self) -> bool {
        self.current_draw_offset < 1.0
    }

    pub fn get_scroll(&self) -> usize {
        self.scroll
    }

    fn _reset_caret(&mut self) {
        self.caret_visible = true;
        self.caret_delta = 0.0;
    }

    fn _handle_input(&mut self, cfg: &mut ConfigVariables) {
        self.scroll = 0;

        if self.input_buffer.is_empty() {
            return;
        }

        logger::input(&*self.get_current_input());
        let input = self.get_current_input();

        let split: Vec<&str> = input.split(' ').collect();
        let cvar_opt = cfg.get_cvar_id_from_str(split[0]);

        if let Some(cvar) = cvar_opt {
            let cvar_id = *cvar;
            if split.len() >= 2 {
                let datatype = cfg.get(cvar_id).get_type();
                let mut parsed = false;

                match datatype {
                    CvarType::Float => {
                        let parsed_arg = split[1].parse::<f32>();
                        if let Ok(arg) = parsed_arg {
                            cfg.set(cvar_id, arg);
                            parsed = true;
                        }
                    }
                    CvarType::Integer => {
                        let parsed_arg = split[1].parse::<u32>();
                        if let Ok(arg) = parsed_arg {
                            cfg.set(cvar_id, arg);
                            parsed = true;
                        }
                    }
                    CvarType::String => {
                        let string_split: Vec<&str> = input.split('"').collect();
                        if string_split.len() > 2 {
                            cfg.set(cvar_id, String::from(string_split[1]));
                            parsed = true;
                        }
                    }
                }
                if !parsed {
                    log_error!("failed to parse cvar argument: {}", split[1]);
                }
            }
            logger::cvar(&*cfg.get_desc(cvar_id));
        } else {
            log_error!("unknown command or cvar: {}", self.get_current_input());
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
            self.input_buffer = self.input_history[self.input_history.len() - self.input_history_index - 1]
                .chars()
                .collect();
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
            self.input_buffer = self.input_history[self.input_history.len() - self.input_history_index - 1]
                .chars()
                .collect();
        }
        self.input_index = self.input_buffer.len() as u32;
    }

    fn _scroll_up(&mut self) {
        let history_length = logger::len();
        self.scroll += SCROLL_LINES;

        if self.scroll > history_length {
            self.scroll = history_length;
        }
    }

    fn _scroll_down(&mut self) {
        if self.scroll >= SCROLL_LINES {
            self.scroll -= SCROLL_LINES;
        } else {
            self.scroll = 0;
        }
    }
}

