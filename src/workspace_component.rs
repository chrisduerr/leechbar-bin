use i3ipc::reply::Workspace as I3Workspace;
use i3ipc::I3Connection;
use std::time::Duration;
use leechbar::*;

pub struct Workspace {
    bar: Bar,
    id: String,
    redraw: bool,
    bg_img: Image,
    bg_sec_img: Image,
    old_state: String,
}

impl Workspace {
    pub fn new(bar: Bar, bg_img: Image, bg_sec_img: Image, id: String) -> Self {
        Self {
            id,
            bar,
            bg_img,
            bg_sec_img,
            redraw: true,
            old_state: String::new(),
        }
    }
}

impl Component for Workspace {
    fn background(&mut self) -> Background {
        // Get the workspace
        let ws_res = workspace_by_id(&self.id);
        if ws_res.is_err() {
            if self.old_state != "missing" {
                self.old_state = "missing".into();
            } else {
                self.redraw = false;
            }
            return Background::new().image(&self.bg_img);
        }
        let ws = ws_res.unwrap();

        // Construct a string with the current state of the ws
        let state = ws.visible.to_string();

        // Only keep going if redraw is required
        if state != self.old_state {
            self.old_state = state;
            if ws.visible {
                Background::new().image(&self.bg_sec_img)
            } else {
                Background::new().image(&self.bg_img)
            }
        } else {
            self.redraw = false;
            Background::new()
        }
    }

    fn foreground(&mut self) -> Option<Foreground> {
        let text = Text::new(&self.bar, &self.id, None, None).unwrap();
        Some(Foreground::new(text))
    }

    fn timeout(&mut self) -> Option<Duration> {
        Some(Duration::from_millis(100))
    }

    fn alignment(&mut self) -> Alignment {
        Alignment::LEFT
    }

    fn width(&mut self) -> Width {
        Width::new().fixed(60)
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

// Get an i3 workspace with a matching id
// TODO: Keep the connection up
fn workspace_by_id(id: &str) -> Result<I3Workspace, String> {
    // Connect to i3
    let mut conn = I3Connection::connect().map_err(|e| format!("Unable to connect to i3: {}", e))?;

    // Get all workspaces
    let workspaces = conn.get_workspaces()
        .map_err(|e| format!("Unable to get workspaces: {}", e))?;

    // Return workspace with matching id
    for workspace in workspaces.workspaces {
        if workspace.name == id {
            return Ok(workspace);
        }
    }

    Err(format!("Unable to find workspace with ID {}", id))
}
