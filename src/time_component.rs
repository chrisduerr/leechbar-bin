use image_cache::ImageCache;
use std::time::Duration;
use leechbar::*;
use std::thread;
use time;
use chan;

pub struct Time {
    bar: Bar,
    image_cache: ImageCache,
    last_content: String,
    last_text: Option<Text>,
}

impl Time {
    pub fn new(bar: Bar, image_cache: ImageCache) -> Self {
        Self {
            bar,
            image_cache,
            last_text: None,
            last_content: String::new(),
        }
    }
}

impl Component for Time {
    fn update(&mut self) -> bool {
        let time = time::now();
        let content = format!("{:02}:{:02}", time.tm_hour, time.tm_min);

        if content != self.last_content {
            self.last_text = if !content.is_empty() {
                self.last_content = content;
                Some(Text::new(&self.bar, &self.last_content, None, None).unwrap())
            } else {
                None
            };

            true
        } else {
            false
        }
    }

    fn background(&self) -> Background {
        Background::new().image(self.image_cache.get("./images/bg_sec.png").unwrap())
    }

    fn foreground(&self) -> Foreground {
        if let Some(ref last_text) = self.last_text {
            last_text.clone().into()
        } else {
            Foreground::new()
        }
    }

    fn redraw_timer(&mut self) -> chan::Receiver<()> {
        let (tx, rx) = chan::sync(0);

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(15));
            let _ = tx.send(());
        });

        rx
    }

    fn width(&self) -> Width {
        Width::new().fixed(100)
    }
}
