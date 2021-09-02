use image::{GenericImageView};
use std::path::Path;

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Image {
    fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Image { width, height, data }
    }
}

pub fn load_image(image_path: &Path) -> Image {
    let mut image_object = image::open(image_path).unwrap();
    let width = image_object.width();
    let height = image_object.height();

    let data = image_object.to_rgba8().into_raw();

    Image::new(width, height, data)
}
