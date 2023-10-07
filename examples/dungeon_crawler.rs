mod dungeon_crawler_example;

use vulkrap::vulkrap_start;

use crate::dungeon_crawler_example::app::DungeonCrawler;

const WINDOW_TITLE: &str = "dungeon crawler";
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;


pub fn main() {
    vulkrap_start(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, DungeonCrawler::new);
}
