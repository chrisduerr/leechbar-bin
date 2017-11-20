use leechbar::{Alignment, Background, Bar, Component, Event, Foreground, Image, MouseButton, Text,
               Width};
use image::{DynamicImage, GenericImage, Rgba};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::AtomicUsize;
use image_cache::ImageCache;
use std::process::Command;
use libpulse_sys::*;
use std::sync::Arc;
use std::ptr;
use std::cmp;
use libc;
use chan;

// Set the 100% volume
const MAX_VOL: f64 = 65536.;

// The color of the filled slider
const SLIDER_COLOR: [u8; 4] = [117, 42, 42, 255];
// Color of the empty part of the slider
const TROUGH_COLOR: [u8; 4] = [27, 27, 27, 255];

// Create globals because the pulse event queue has no access to any struct
lazy_static! {
    // This is the current volume
    static ref VOLUME: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    // This channel is used for prompting the bar to redraw
    static ref CHANNEL: (chan::Sender<()>, chan::Receiver<()>) = chan::sync(0);
}

pub struct VolumeSlider {
    x: i16,
    y: i16,
    width: i16,
    height: i16,
    bar: Bar,
    holding: bool,
    slider_mode: bool,
    image_cache: ImageCache,
}

// Create the volume component
impl VolumeSlider {
    pub fn new(bar: Bar, image_cache: ImageCache, x: i16, y: i16, width: i16, height: i16) -> Self {
        // Start pulse listening
        unsafe { start_volume_listener() };

        Self {
            x,
            y,
            width,
            height,
            bar,
            image_cache,
            holding: false,
            slider_mode: false,
        }
    }
}

impl Component for VolumeSlider {
    fn event(&mut self, event: Event) -> bool {
        // Scroll change vol in all modes
        if let Event::ClickEvent(ref e) = event {
            if e.button == MouseButton::WheelUp {
                let vol = VOLUME.load(Relaxed);
                if vol < 100 {
                    VOLUME.store(vol + 1, Relaxed);
                    change_pulse_vol(vol + 1);
                    return true;
                }
            } else if e.button == MouseButton::WheelDown {
                let vol = VOLUME.load(Relaxed);
                if vol > 0 {
                    VOLUME.store(vol - 1, Relaxed);
                    change_pulse_vol(vol - 1);
                    return true;
                }
            }
        }

        // Change to slider mode when clicked
        if !self.slider_mode {
            if let Event::ClickEvent(ref e) = event {
                if e.button == MouseButton::Left {
                    self.slider_mode = true;
                    return true;
                }
            }
            return false;
        }

        // Update slider when already in slider mode
        match event {
            Event::ClickEvent(ref e) => if e.button == MouseButton::Left {
                // Stop dragging when button released
                if e.released {
                    self.holding = false;
                    return false;
                }

                // Set holding if clicked within bounds
                let pos = e.position;
                if pos.x >= self.x && pos.y < self.x + self.width && pos.y >= self.y
                    && pos.y < self.y + self.height
                {
                    self.holding = true;

                    // Update pointer position when within bounds
                    update_volume(pos.x, self.x, self.width);
                    return true;
                }
            } else if e.button == MouseButton::Right {
                // Leave slider mode when RMB has been pressed on the component
                self.slider_mode = false;
                return true;
            },
            Event::MotionEvent(ref e) => if self.holding {
                let pos = e.position;
                if pos.x == 0 || pos.y == 0 || pos.x == 2 * self.x + self.width
                    || pos.y == 2 * self.y + self.height
                {
                    // Remove holding when mouse leaves component
                    self.holding = false;
                    return false;
                } else {
                    // Update pointer position when within bounds
                    update_volume(pos.x, self.x, self.width);
                    return true;
                }
            },
        }

        false
    }

    // Redraw when global channel receives message
    fn redraw_timer(&mut self) -> chan::Receiver<()> {
        CHANNEL.1.clone()
    }

    // Display text only in non-slider mode
    fn foreground(&self) -> Foreground {
        if self.slider_mode {
            Foreground::new()
        } else {
            Text::new(&self.bar, &VOLUME.load(Relaxed).to_string(), None, None)
                .unwrap()
                .into()
        }
    }

    // Display slider in slider mode, otherwise just normal background
    fn background(&self) -> Background {
        let mut background =
            Background::new().image(self.image_cache.get("./images/bg_sec.png").unwrap());

        // Add slider to the background
        if self.slider_mode {
            let mut img = DynamicImage::new_rgba8(self.width as u32, (self.y + self.height) as u32);
            let max_x = (self.width * VOLUME.load(Relaxed) as i16) / 100;
            for x in 0..self.width {
                for y in self.y..self.height + self.y {
                    let rgba = if x < max_x {
                        Rgba { data: SLIDER_COLOR }
                    } else {
                        Rgba { data: TROUGH_COLOR }
                    };
                    img.put_pixel(x as u32, y as u32, rgba);
                }
            }
            let ximg = Image::new(&self.bar, &img).unwrap();
            background = background.image(ximg);
        }

        background
    }

    fn width(&self) -> Width {
        if self.slider_mode {
            Width::new().fixed((2 * self.x + self.width) as u16)
        } else {
            Width::new().fixed(75)
        }
    }

    fn alignment(&self) -> Alignment {
        Alignment::RIGHT
    }
}

fn update_volume(x: i16, x_offset: i16, width: i16) {
    // Get new volume percentage
    let relative_x = cmp::max(cmp::min(x - x_offset, width), 0);
    let percentage = (f64::from(relative_x) / f64::from(width) * 100f64) as usize;
    VOLUME.store(percentage, Relaxed);

    change_pulse_vol(percentage);
}

fn change_pulse_vol(vol: usize) {
    let vol = cmp::min(vol, 100);
    let command = format!("pactl set-sink-volume 0 {}%", vol);
    let _ = Command::new("sh").args(&["-c", &command]).output();
}

// Start the pulseaudio listener
unsafe fn start_volume_listener() {
    // Start the async main loop
    let pa_mainloop = pa_threaded_mainloop_new();
    pa_threaded_mainloop_start(pa_mainloop);

    // Create a pulseaudio context
    let pa_mainloop_api = pa_threaded_mainloop_get_api(pa_mainloop);
    let pa_context = pa_context_new(pa_mainloop_api, ptr::null());

    // Register the callback for successful context connection
    pa_context_set_state_callback(pa_context, Some(pa_context_callback), ptr::null_mut());
    pa_context_connect(pa_context, ptr::null(), PA_CONTEXT_NOFLAGS, ptr::null());
}

// Callback when pulseaudio context connected
unsafe extern "C" fn pa_context_callback(pa_context: *mut pa_context, _: *mut libc::c_void) {
    // Check the context state
    match pa_context_get_state(pa_context) {
        // Ignore these states
        PA_CONTEXT_CONNECTING | PA_CONTEXT_AUTHORIZING | PA_CONTEXT_SETTING_NAME => (),
        // If the state is ready, we can subscribe to pulse events
        PA_CONTEXT_READY => {
            // Setup the callback for the subscriptyon
            pa_context_set_subscribe_callback(
                pa_context,
                Some(pa_subscription_callback),
                ptr::null_mut(),
            );

            // Subscribe to all sink events
            let pa_operation =
                pa_context_subscribe(pa_context, PA_SUBSCRIPTION_MASK_SINK, None, ptr::null_mut());
            pa_operation_unref(pa_operation);
        }
        _ => {
            // Abort if connection to pulse was not possible
            let error = pa_strerror(pa_context_errno(pa_context));
            pa_context_unref(pa_context);
            panic!("Pulse connection failure: {:?}", error);
        }
    };
}

// Sink event callback
unsafe extern "C" fn pa_subscription_callback(
    pa_context: *mut pa_context,
    _: Enum_pa_subscription_event_type,
    _: u32,
    _: *mut libc::c_void,
) {
    // Get the sink info
    let pa_operation =
        pa_context_get_sink_info_list(pa_context, Some(pa_sink_callback), ptr::null_mut());
    pa_operation_unref(pa_operation);
}

// Get the volume percentage from a sink
unsafe extern "C" fn pa_sink_callback(
    _: *mut Struct_pa_context,
    pa_sink_info: *const Struct_pa_sink_info,
    _: i32,
    _: *mut libc::c_void,
) {
    if !pa_sink_info.is_null() {
        let vol = if (*pa_sink_info).mute == 1 {
            // Set volume to 0 if sink is muted
            0.
        } else {
            // Calculate the volume percentage
            (100. * f64::from(pa_cvolume_avg(&(*pa_sink_info).volume)) / MAX_VOL).round()
        };

        // Update the global state
        VOLUME.store(vol as usize, Relaxed);
        CHANNEL.0.send(());
    }
}
