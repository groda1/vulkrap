use std::path::Path;

pub fn read_file(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|_| panic!("Unable to open file: {:?}", path))
}
