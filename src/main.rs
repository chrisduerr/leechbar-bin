extern crate chan;
extern crate env_logger;
extern crate i3ipc;
extern crate image;
#[macro_use]
extern crate lazy_static;
extern crate leechbar;
extern crate libc;
extern crate libpulse_sys;
#[macro_use]
extern crate log;
extern crate time;


mod workspace_component;
mod volume_component;
mod time_component;
mod image_cache;
mod i3;

use workspace_component::Workspace;
use volume_component::Volume;
use image_cache::ImageCache;
use time_component::Time;
use leechbar::*;
use std::env;
use i3::I3;

const BG: &str = "./images/bg.png";
const FONT: &str = "Fira Sans 12";
const NAME: &str = "LeechBar";

fn main() {
    env_logger::init().unwrap();

    // Get output and ws offset
    let output = env::args().nth(1).expect("Please specify an output");
    let ws_offset = env::args().nth(2).expect("Please spcify ws offset");
    let ws_offset = i32::from_str_radix(&ws_offset, 10).unwrap();

    let bg_img = image::open(BG).unwrap();
    let mut bar = BarBuilder::new()
        .foreground_color(Color::new(158, 158, 158, 255))
        .background_image(bg_img.clone())
        .text_yoffset(-1)
        .output(output)
        .height(32)
        .font(FONT)
        .name(NAME)
        .spawn()
        .unwrap();

    let image_cache = ImageCache::new(bar.clone());

    // Workspaces
    let mut eye_three = I3::new();
    for i in 0..5 {
        let id = (i * 3 + ws_offset).to_string();

        let (tx, rx) = ::std::sync::mpsc::channel();
        eye_three.add(id.clone(), tx);

        let ws = Workspace::new(i, image_cache.clone(), rx);
        bar.add(ws);
    }

    // Time
    let time = Time::new(bar.clone(), image_cache.clone());
    bar.add(time);

    // Volume
    let vol = Volume::new(bar.clone(), image_cache.clone());
    bar.add(vol);

    bar.start_event_loop();
}
