use std::process::Command as Cmd;
use std::time::Duration;
use leechbar::*;

const COMMAND: &str = "pactl list sinks | grep '^[[:space:]]Volume:' | \
                       head -n 1 | tail -n 1 | sed -e 's,.* \\([0-9][0-9]*\\)%.*,\\1,'";

pub struct Volume {
    bar: Bar,
    redraw: bool,
    bg_img: Image,
    last_content: String,
    last_text: Option<Text>,
}

impl Volume {
    pub fn new(bar: Bar, bg_img: Image) -> Self {
        Self {
            bar,
            bg_img,
            redraw: true,
            last_text: None,
            last_content: String::new(),
        }
    }
}

impl Component for Volume {
    fn background(&mut self) -> Background {
        Background::new().image(&self.bg_img)
    }

    fn foreground(&mut self) -> Option<Foreground> {
        let output = Cmd::new("sh").args(&["-c", COMMAND]).output().unwrap();
        let content = String::from_utf8_lossy(&output.stdout).trim().to_owned();

        if content.is_empty() {
            return None;
        }

        if content != self.last_content {
            self.last_content = content;
            self.last_text = Some(Text::new(&self.bar, &self.last_content, None, None).unwrap());
            Some(Foreground::new(self.last_text.as_ref().unwrap()))
        } else {
            self.redraw = false;
            None
        }
    }

    fn timeout(&mut self) -> Option<Duration> {
        Some(Duration::from_millis(250))
    }

    fn alignment(&mut self) -> Alignment {
        Alignment::RIGHT
    }

    fn width(&mut self) -> Width {
        Width::new().fixed(75)
    }

    fn redraw(&mut self) -> bool {
        if !self.redraw {
            self.redraw = true;
            false
        } else {
            true
        }
    }
}
