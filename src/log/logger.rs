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

    pub fn input(&mut self, message: String) {
        self.history.push(LogMessage::new(MessageLevel::Input, message));
    }

    pub fn output(&mut self, message: String) {
        self.history.push(LogMessage::new(MessageLevel::Output, message));
    }

    pub fn cvar(&mut self, message: String) {
        self.history.push(LogMessage::new(MessageLevel::Cvar, message));
    }

    pub fn error(&mut self, message: String) {
        self.history.push(LogMessage::new(MessageLevel::Error, message));
    }

    pub fn warning(&mut self, message: String) {
        self.history.push(LogMessage::new(MessageLevel::Warning, message));
    }

    pub fn info(&mut self, message: String) {
        println!("{}",message);
        self.history.push(LogMessage::new(MessageLevel::Info, message));
    }

    pub fn debug(&mut self, message: String) {
        println!("{}",message);
        self.history.push(LogMessage::new(MessageLevel::Debug, message));
    }

    pub fn debug_once(&mut self, message: String) {
        if let Some(last_message) = self.history.last() {
            if last_message.message.eq(&message) {
                return;
            }
        }
        self.history.push(LogMessage::new(MessageLevel::Debug, message));
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
        LogMessage { level, message }
    }
}

pub enum MessageLevel {
    Input,
    Output,
    Cvar,
    Error,
    Warning,
    Info,
    Debug,
}

pub fn input(line: &str) {
    LOGGER.lock().unwrap().input(String::from(line));
}

pub fn _output(line: &str) {
    LOGGER.lock().unwrap().output(String::from(line));
}

pub fn cvar(line: &str) {
    LOGGER.lock().unwrap().cvar(String::from(line));
}

pub fn error(line: &str) {
    line.split('\n')
        .for_each(|s| LOGGER.lock().unwrap().error(String::from(s)));
}

pub fn warning(line: &str) {
    line.split('\n')
        .for_each(|s| LOGGER.lock().unwrap().warning(String::from(s)));
}

pub fn info(line: &str) {
    line.split('\n')
        .for_each(|s| LOGGER.lock().unwrap().info(String::from(s)));
}

pub fn debug(line: &str) {
    line.split('\n')
        .for_each(|s| LOGGER.lock().unwrap().debug(String::from(s)));
}

pub fn debug_once(line: &str) {
    line.split('\n')
        .for_each(|s| LOGGER.lock().unwrap().debug_once(String::from(s)));
}

pub fn get() -> MutexGuard<'static, Logger> {
    LOGGER.lock().unwrap()
}

pub fn len() -> usize {
    LOGGER.lock().unwrap().history.len()
}
