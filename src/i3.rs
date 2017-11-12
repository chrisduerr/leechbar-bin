use i3ipc::event::inner::WindowChange::{Close, Move, New};
use i3ipc::event::inner::WorkspaceChange::{Empty, Focus};
use i3ipc::event::Event::{WindowEvent, WorkspaceEvent};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use i3ipc::reply::Node;
use std::thread;
use i3ipc;

pub struct I3Change {
    pub state: Option<bool>,
    pub title: Option<String>,
}

impl I3Change {
    fn new(state: Option<bool>, title: Option<String>) -> Self {
        Self { state, title }
    }
}

pub struct I3Listener {
    old_state: bool,
    old_title: String,
    sender: Sender<I3Change>,
}

impl I3Listener {
    fn new(sender: Sender<I3Change>) -> Self {
        Self {
            sender,
            old_state: false,
            old_title: String::new(),
        }
    }
}

pub struct I3 {
    senders: Arc<Mutex<HashMap<String, I3Listener>>>,
    tree: Arc<Mutex<Node>>,
}

impl I3 {
    pub fn new() -> Self {
        let mut conn = i3ipc::I3Connection::connect().unwrap();
        let i3 = I3 {
            senders: Arc::new(Mutex::new(HashMap::new())),
            tree: Arc::new(Mutex::new(conn.get_tree().unwrap())),
        };

        i3.listen_workspace();
        i3.listen_windows();

        i3
    }

    pub fn add(&mut self, workspace: String, sender: Sender<I3Change>) {
        let _ = sender.send(I3Change::new(Some(false), None));
        let mut lock = self.senders.lock().unwrap();
        lock.insert(workspace, I3Listener::new(sender));
    }

    fn listen_workspace(&self) {
        let senders = Arc::clone(&self.senders);
        thread::spawn(move || {
            loop {
                info!("Starting i3 workspace connection");
                // Start normal connection for ws queries
                let mut conn = i3ipc::I3Connection::connect().unwrap();

                // Start event connection for ws events
                let mut event_conn = i3ipc::I3EventListener::connect().unwrap();
                let _ = event_conn.subscribe(&[i3ipc::Subscription::Workspace]);

                for event in event_conn.listen() {
                    let ws_event = match event {
                        Ok(e) => match e {
                            WorkspaceEvent(ws_event) => ws_event,
                            _ => continue,
                        },
                        Err(e) => {
                            error!("Workspace i3 connection closed: {}", e);
                            thread::sleep(Duration::from_secs(1));
                            break;
                        }
                    };

                    if let Focus = ws_event.change {
                        let workspaces = conn.get_workspaces().unwrap();
                        let mut lock = senders.lock().unwrap();
                        for workspace in workspaces.workspaces {
                            if let Some(listener) = lock.get_mut(&workspace.name) {
                                if workspace.visible != listener.old_state {
                                    listener.old_state = workspace.visible;
                                    let _ = listener
                                        .sender
                                        .send(I3Change::new(Some(workspace.visible), None));
                                }
                            }
                        }
                    } else if let Empty = ws_event.change {
                        let mut lock = senders.lock().unwrap();
                        let ws_name = ws_event.current.unwrap().name.unwrap();
                        if let Some(listener) = lock.get_mut(&ws_name) {
                            listener.old_state = false;
                            let _ = listener.sender.send(I3Change::new(Some(false), None));
                        }
                    }
                }
            }
        });
    }

    fn listen_windows(&self) {
        let senders = Arc::clone(&self.senders);
        let tree = Arc::clone(&self.tree);
        thread::spawn(move || {
            loop {
                info!("Starting i3 window connection");
                // Start normal connection for ws queries
                let mut conn = i3ipc::I3Connection::connect().unwrap();

                // Start event connection for ws events
                let mut event_conn = i3ipc::I3EventListener::connect().unwrap();
                let _ = event_conn.subscribe(&[i3ipc::Subscription::Window]);

                for event in event_conn.listen() {
                    let window_event = match event {
                        Ok(e) => match e {
                            WindowEvent(window_event) => window_event,
                            _ => continue,
                        },
                        Err(e) => {
                            error!("Window i3 connection closed: {}", e);
                            thread::sleep(Duration::from_secs(1));
                            break;
                        }
                    };

                    let mut tree = tree.lock().unwrap();

                    let mut workspace_states = Vec::new();
                    let id = window_event.container.id;
                    match window_event.change {
                        New => {
                            *tree = conn.get_tree().unwrap();
                            let ws = workspace_state(&(*tree), id);
                            if let Some(ws) = ws {
                                workspace_states.push(ws);
                            }
                        }
                        Move => {
                            let ws = workspace_state(&(*tree), id);
                            if let Some(mut ws) = ws {
                                *tree = conn.get_tree().unwrap();
                                ws.names = child_names_by_ws(&(*tree), &ws.workspace);
                                workspace_states.push(ws);
                            }

                            let ws = workspace_state(&(*tree), id);
                            if let Some(ws) = ws {
                                workspace_states.push(ws);
                            }
                        }
                        Close => {
                            let ws = workspace_state(&(*tree), id);
                            if let Some(mut ws) = ws {
                                *tree = conn.get_tree().unwrap();
                                ws.names = child_names_by_ws(&(*tree), &ws.workspace);
                                workspace_states.push(ws);
                            }
                        }
                        _ => continue,
                    };

                    for state in workspace_states {
                        let name = if state.names.len() == 1 {
                            state.names[0].clone()
                        } else if state.names.is_empty() {
                            "empty".into()
                        } else {
                            "mixed".into()
                        };

                        let mut lock = senders.lock().unwrap();
                        if let Some(listener) = lock.get_mut(&state.workspace) {
                            if listener.old_title != name {
                                listener.old_title = name.clone();
                                let _ = listener.sender.send(I3Change::new(None, Some(name)));
                            }
                        }
                    }
                }
            }
        });
    }
}

struct WorkspaceState {
    names: Vec<String>,
    workspace: String,
}

fn workspace_state(node: &Node, id: i64) -> Option<WorkspaceState> {
    if node.nodes.is_empty() {
        return None;
    }

    for n in &node.nodes {
        if n.id == id {
            let mut names = child_names(node);
            names.sort();
            names.dedup();
            return Some(WorkspaceState {
                names,
                workspace: node.name.clone().unwrap_or_default(),
            });
        }

        if let Some(workspace_state) = workspace_state(n, id) {
            return Some(workspace_state);
        }
    }

    None
}

fn child_names_by_ws(node: &Node, name: &str) -> Vec<String> {
    if node.nodes.is_empty() {
        return Vec::new();
    }

    if let Some(ref node_name) = node.name {
        if node_name == name {
            let mut names = child_names(node);
            names.sort();
            names.dedup();
            return names;
        }
    }

    let mut names = Vec::new();
    for n in &node.nodes {
        names.append(&mut child_names_by_ws(n, name));
    }

    names
}

fn child_names(node: &Node) -> Vec<String> {
    if node.nodes.is_empty() {
        let name = node.name.clone().unwrap_or_default();
        return vec![
            if name.contains("Nightly") {
                "firefox".into()
            } else {
                name
            },
        ];
    }

    let mut names = Vec::new();
    for n in &node.nodes {
        names.append(&mut child_names(n));
    }

    names
}
