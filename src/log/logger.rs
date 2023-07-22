use std::sync::{Mutex, MutexGuard};

const CAPACITY: usize = 10000;

lazy_static! {
    static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

pub struct Logger {
    history: Vec<LogMessage>,
}

#[allow(dead_code)]
impl Logger {
    pub fn new() -> Logger {
        Logger {
            history: Vec::with_capacity(CAPACITY),
        }
    }

    fn add_once(&mut self, message: LogMessage) {
        if let Some(last_message) = self.history.last() {
            if last_message.message.eq(&message.message) {
                return;
            }
        }
        self.history.push(message);
    }

    pub fn get_history(&self, line_count: usize, scroll: usize) -> &[LogMessage] {
        let end = if self.history.len() >= scroll {
            self.history.len() - scroll
        } else {
            self.history.len()
        };

        let start = self.history.len() - line_count.min(self.history.len());
        let start_offset = if start >= scroll { start - scroll } else { 0 };

        &self.history[start_offset..end]
    }
}

pub struct LogMessage {
    pub level: MessageLevel,
    pub message: String,
}

impl LogMessage {
    pub fn new(level: MessageLevel, message: String) -> Self {
        LogMessage {
            level,
            message
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MessageLevel {
    Input,
    Output,
    Cvar,
    Error,
    Warning,
    Info,
    Debug,
    Khronos,
}

fn fmt_line(line: &str, level: MessageLevel) -> impl Iterator<Item=LogMessage> + '_ {
    line.split('\n')
        // TODO how do we wrap long lines?

        .map(move |str| LogMessage::new(level, String::from(str)))
}

fn add_line(line: &str, level: MessageLevel) {
    fmt_line(line, level)
        .for_each(|log_message| LOGGER.lock().unwrap().history.push(log_message));
}

fn add_line_once(line: &str, level: MessageLevel) {
    fmt_line(line, level)
        .for_each(|log_message| LOGGER.lock().unwrap().add_once(log_message));
}

pub fn input(line: &str) {
    add_line(line, MessageLevel::Input);
}

pub fn output(line: &str) {
    add_line(line, MessageLevel::Output);
}

pub fn cvar(line: &str) {
    add_line(line, MessageLevel::Cvar);
}

pub fn error(line: &str) {
    add_line(line, MessageLevel::Error);
}

pub fn warning(line: &str) {
    add_line(line, MessageLevel::Warning);
}

pub fn info(line: &str) {
    add_line(line, MessageLevel::Info);
}

pub fn debug(line: &str) {
    add_line(line, MessageLevel::Debug);
}

pub fn khronos(line: &str) {
    add_line(line, MessageLevel::Khronos);
}

pub fn debug_once(line: &str) {
    add_line_once(line, MessageLevel::Debug);
}

pub fn get() -> MutexGuard<'static, Logger> {
    LOGGER.lock().unwrap()
}

pub fn len() -> usize {
    LOGGER.lock().unwrap().history.len()
}
