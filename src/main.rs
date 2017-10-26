extern crate chan;
extern crate env_logger;
extern crate i3ipc;
extern crate image;
extern crate leechbar;
extern crate time;

// mod workspace_component;
// mod volume_component;
mod time_component;
mod i3;

// use workspace_component::Workspace;
// use volume_component::Volume;
use std::collections::HashMap;
use time_component::Time;
use leechbar::*;

const OUTPUT: &str = "DisplayPort-0";

const BG: &str = "./images/bg.png";
const BG_SEC: &str = "./images/bg_sec.png";

fn main() {
    env_logger::init().unwrap();

    let bg_img = image::open(BG).unwrap();
    let mut bar = BarBuilder::new()
        .foreground_color(Color::new(158, 158, 158, 255))
        .background_image(bg_img.clone())
        .font("Fira Sans 12")
        .name("LeechBar")
        .text_yoffset(-1)
        .output(OUTPUT)
        .height(32)
        .spawn()
        .unwrap();

    // Create cache for globally used images
    let mut cache = HashMap::new();
    cache.insert("bg", Image::new(&bar, &bg_img).unwrap());
    cache.insert(
        "bg_sec",
        Image::new(&bar, &image::open(BG_SEC).unwrap()).unwrap(),
    );

    // Workspaces
    // for i in 0..5 {
    //     let id = (i * 3 + 1).to_string();
    //     let ws = Workspace::new(bar.clone(), bg_img.clone(), bg_sec.clone(), id);
    //     bar.add(ws);
    // }

    // Time
    let time = Time::new(bar.clone(), cache.clone());
    bar.add(time);

    // Volume
    // let vol = Volume::new(bar.clone(), bg_sec.clone());
    // bar.add(vol);

    bar.start_event_loop();
}
