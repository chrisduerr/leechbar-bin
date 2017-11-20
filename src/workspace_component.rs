use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use image_cache::ImageCache;
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::path::Path;
use i3::I3Change;
use leechbar::*;
use std::thread;
use chan;

pub struct Workspace {
    id: i32,
    image_cache: ImageCache,
    visible: Arc<AtomicBool>,
    title: Arc<Mutex<String>>,
    receiver: Option<Receiver<I3Change>>,
}

impl Workspace {
    pub fn new(id: i32, image_cache: ImageCache, rc: Receiver<I3Change>) -> Self {
        Self {
            id: id + 1,
            image_cache,
            receiver: Some(rc),
            title: Arc::new(Mutex::new("empty".into())),
            visible: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl Component for Workspace {
    fn redraw_timer(&mut self) -> chan::Receiver<()> {
        let (tx, rx) = chan::sync(0);

        let receiver = self.receiver.take().unwrap();
        let visible = Arc::clone(&self.visible);
        let title = Arc::clone(&self.title);
        thread::spawn(move || {
            loop {
                if let Ok(change) = receiver.recv() {
                    if let Some(new_visible) = change.state {
                        if visible.load(Ordering::Relaxed) != new_visible {
                            visible.store(new_visible, Ordering::Relaxed);
                            tx.send(());
                        }
                    }

                    let mut title_lock = title.lock().unwrap();
                    if let Some(new_title) = change.title {
                        if *title_lock != new_title {
                            *title_lock = new_title;
                            tx.send(());
                        }
                    }
                }
            }
        });

        rx
    }

    fn event(&mut self, event: Event) -> bool {
        if let Event::ClickEvent(e) = event {
            if let MouseButton::Left = e.button {
                // Change workspace and swallow stdout
                let command = format!("$HOME/scripts/switch_focused_workspace {}", self.id);
                let _ = Command::new("sh").args(&["-c", &command]).output();
            }
        }

        false
    }

    fn background(&self) -> Background {
        // Lock title to this thread
        let title_lock = self.title.lock().unwrap();

        let path_str;
        let path = if !self.visible.load(Ordering::Relaxed) {
            path_str = format!("./images/ws/{}.png", title_lock);
            let path = Path::new(&path_str);
            if path.exists() {
                path
            } else {
                Path::new("./images/ws/mixed.png")
            }
        } else {
            path_str = format!("./images/ws/{}_sec.png", title_lock);
            let path = Path::new(&path_str);
            if path.exists() {
                path
            } else {
                Path::new("./images/ws/mixed_sec.png")
            }
        };

        self.image_cache.get(path).unwrap().into()
    }

    fn foreground(&self) -> Foreground {
        Foreground::new()
    }

    fn alignment(&self) -> Alignment {
        Alignment::LEFT
    }

    fn width(&self) -> Width {
        Width::new().fixed(60)
    }
}
