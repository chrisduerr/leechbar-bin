use std::process::Command as Cmd;
use image_cache::ImageCache;
use std::time::Duration;
use std::thread;
use leechbar::*;
use chan;

const COMMAND: &str = "pactl list sinks | grep '^[[:space:]]Volume:' | \
                       head -n 1 | tail -n 1 | sed -e 's,.* \\([0-9][0-9]*\\)%.*,\\1,'";

pub struct Volume {
    bar: Bar,
    cache: ImageCache,
    last_content: String,
    last_text: Option<Text>,
}

impl Volume {
    pub fn new(bar: Bar, cache: ImageCache) -> Self {
        Self {
            bar,
            cache,
            last_text: None,
            last_content: String::new(),
        }
    }
}

impl Component for Volume {
    fn update(&mut self) -> bool {
        let output = Cmd::new("sh").args(&["-c", COMMAND]).output().unwrap();
        let content = String::from_utf8_lossy(&output.stdout).trim().to_owned();

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
        Background::new().image(self.cache.get("./images/bg_sec.png").unwrap())
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
            thread::sleep(Duration::from_secs(1));
            let _ = tx.send(());
        });

        rx
    }

    fn alignment(&self) -> Alignment {
        Alignment::RIGHT
    }

    fn width(&self) -> Width {
        Width::new().fixed(75)
    }
}
