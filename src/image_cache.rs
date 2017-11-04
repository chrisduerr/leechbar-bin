use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use leechbar::{Bar, Image};
use std::path::PathBuf;
use image;

#[derive(Clone)]
pub struct ImageCache {
    bar: Bar,
    cache: Arc<Mutex<HashMap<PathBuf, Image>>>,
}

impl ImageCache {
    pub fn new(bar: Bar) -> Self {
        Self {
            bar,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get<T: Into<PathBuf>>(&self, path: T) -> Option<Image> {
        let path = path.into();
        let mut lock = self.cache.lock().unwrap();

        if let Some(image) = lock.get(&path) {
            return Some(image.clone());
        }

        if let Ok(img) = image::open(&path) {
            let image = Image::new(&self.bar, &img).unwrap();
            lock.insert(path.clone(), image.clone());
            Some(image)
        } else {
            None
        }
    }
}
