
use vulkrap::vulkrap_start;
use crate::terrain_example::test_app::TerrainApp;


const WINDOW_TITLE: &str = "terrain test";
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

mod terrain_example;

fn main() {
    vulkrap_start(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, TerrainApp::new);
}
