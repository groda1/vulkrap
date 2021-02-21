use std::sync::Mutex;

lazy_static! {
    static ref BUFFER: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn info(line: &str) {
    BUFFER.lock().unwrap().push(line.to_string());
    println!("[INFO] {}", line);
}

pub fn debug(line: &str) {
    BUFFER.lock().unwrap().push(line.to_string());
    println!("[DEBUG] {}", line);
}

pub fn warning(line: &str) {
    BUFFER.lock().unwrap().push(line.to_string());
    println!("[WARNING] {}", line);
}

pub fn print_last_10() {
    let buffer = BUFFER.lock().unwrap();

    for i in 0..buffer.len() {
        println!("log: {}", buffer[i]);
    }
}
