use std::collections::HashMap;
use std::time::Duration;
use leechbar::*;
use time;

pub struct Time<'a> {
    bar: Bar,
    last_content: String,
    last_text: Option<Text>,
    cache: HashMap<&'a str, Image>,
}

impl<'a> Time<'a> {
    pub fn new(bar: Bar, cache: HashMap<&'a str, Image>) -> Self {
        Self {
            bar,
            cache,
            last_text: None,
            last_content: String::new(),
        }
    }
}

impl<'a> Component for Time<'a> {
    fn background(&self) -> Background {
        Background::new().image(self.cache.get("bg_sec").unwrap().clone())
    }

    fn foreground(&self) -> Foreground {
        if let Some(ref last_text) = self.last_text {
            last_text.clone().into()
        } else {
            Foreground::new()
        }
    }

    fn timeout(&self) -> Option<Timeout> {
        Some(Timeout::new_duration(Duration::from_secs(15)))
    }

    fn width(&self) -> Width {
        Width::new().fixed(100)
    }

    fn update(&mut self) -> bool {
        let time = time::now();
        let content = format!("{:02}:{:02}", time.tm_hour, time.tm_min);

        self.last_text = if content != self.last_content && !content.is_empty() {
            self.last_content = content;
            Some(Text::new(&self.bar, &self.last_content, None, None).unwrap())
        } else {
            None
        };

        true
    }
}
