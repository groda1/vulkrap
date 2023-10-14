use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

pub fn read_file(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|_| panic!("Unable to open file: {:?}", path))
}

pub fn read_lines(path: &Path) -> Result<io::Lines<io::BufReader<File>>, &'static str> {
    let file = File::open(path);
    if file.is_err() {
        return Err("Failed to open file");
    }

    Ok(io::BufReader::new(file.unwrap()).lines())
}
