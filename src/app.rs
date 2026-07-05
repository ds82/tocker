use anyhow::Result;
use bollard::query_parameters::{
    ListContainersOptionsBuilder, ListImagesOptionsBuilder, LogsOptionsBuilder,
    RemoveContainerOptionsBuilder, RemoveImageOptionsBuilder, StopContainerOptionsBuilder,
};
use bollard::Docker;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

use crate::docker::client::SshTunnel;
use crate::docker::types::{Container, ContainerState, Image, Network, Volume};
use crate::history::History;
use crate::input::{map_key, Action};

// ── Section ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Section {
    Containers,
    Images,
    Volumes,
    Networks,
}

impl Section {
    pub const ALL: [Section; 4] = [
        Section::Containers,
        Section::Images,
        Section::Volumes,
        Section::Networks,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Containers => "Containers",
            Self::Images => "Images",
            Self::Volumes => "Volumes",
            Self::Networks => "Networks",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Containers => 0,
            Self::Images => 1,
            Self::Volumes => 2,
            Self::Networks => 3,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Containers,
            1 => Self::Images,
            2 => Self::Volumes,
            _ => Self::Networks,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "images" => Self::Images,
            "volumes" => Self::Volumes,
            "networks" => Self::Networks,
            _ => Self::Containers,
        }
    }
}

// ── Mode ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Command(String),
    Filter(String),
    Log { scroll: usize, follow: bool },
    Confirm(PendingAction),
    Menu { cursor: usize },
    Visual { anchor: usize, cursor: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    Remove { key: String, display: String },
    BulkRemoveContainers { ids: Vec<String>, count: usize },
}

// ── Async messaging ───────────────────────────────────────────────────────────

/// Events arriving from background tasks, delivered via a single channel.
pub enum AppMsg {
    LogLine(String),
    Cmd(CmdResult),
}

pub enum CmdResult {
    Done { refresh: Section },
    Failed { message: String },
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    pub docker: Arc<Docker>,
    pub _tunnel: Option<SshTunnel>,
    pub mode: Mode,
    pub section: Section,
    pub needs_refresh: bool,
    pub pending_exec: Option<String>,
    // Section data
    pub containers: Vec<Container>,
    pub images: Vec<Image>,
    pub volumes: Vec<Volume>,
    pub networks: Vec<Network>,
    pub selected: usize,
    // Log viewer
    pub log_lines: Vec<String>,
    pub status: Option<String>,
    log_stop_tx: Option<oneshot::Sender<()>>,
    // Async task messaging
    msg_tx: mpsc::Sender<AppMsg>,
    msg_rx: mpsc::Receiver<AppMsg>,
    // Spinner state (visible while pending_count > 0)
    pub pending_count: usize,
    pub spinner_frame: u8,
    pub spinner_label: String,
    // Command history
    history: History,
}

impl App {
    pub fn new(docker: Docker, tunnel: Option<SshTunnel>, default_section: Section) -> Self {
        let (msg_tx, msg_rx) = mpsc::channel(256);
        Self {
            docker: Arc::new(docker),
            _tunnel: tunnel,
            mode: Mode::Normal,
            section: default_section,
            needs_refresh: default_section != Section::Containers,
            pending_exec: None,
            containers: Vec::new(),
            images: Vec::new(),
            volumes: Vec::new(),
            networks: Vec::new(),
            selected: 0,
            log_lines: Vec::new(),
            status: None,
            log_stop_tx: None,
            msg_tx,
            msg_rx,
            pending_count: 0,
            spinner_frame: 0,
            spinner_label: String::new(),
            history: History::load(),
        }
    }

    // ── Visual mode helpers ──────────────────────────────────────────────────

    pub fn visual_range(&self) -> Option<(usize, usize)> {
        if let Mode::Visual { anchor, cursor } = self.mode {
            Some((anchor.min(cursor), anchor.max(cursor)))
        } else {
            None
        }
    }

    pub fn visual_cursor(&self) -> Option<usize> {
        if let Mode::Visual { cursor, .. } = self.mode {
            Some(cursor)
        } else {
            None
        }
    }

    // ── Visible item helpers ─────────────────────────────────────────────────

    pub fn visible_containers(&self) -> Vec<&Container> {
        self.filter_list(&self.containers, |c| {
            format!("{} {}", c.name, c.image)
        })
    }

    pub fn visible_images(&self) -> Vec<&Image> {
        self.filter_list(&self.images, |img| {
            format!("{}:{}", img.repository, img.tag)
        })
    }

    pub fn visible_volumes(&self) -> Vec<&Volume> {
        self.filter_list(&self.volumes, |v| v.name.clone())
    }

    pub fn visible_networks(&self) -> Vec<&Network> {
        self.filter_list(&self.networks, |n| format!("{} {}", n.name, n.driver))
    }

    fn filter_list<'a, T, F>(&'a self, list: &'a [T], key_fn: F) -> Vec<&'a T>
    where
        F: Fn(&T) -> String,
    {
        if let Mode::Filter(ref q) = self.mode {
            if !q.is_empty() {
                let q = q.to_lowercase();
                return list.iter().filter(|item| key_fn(item).to_lowercase().contains(&q)).collect();
            }
        }
        list.iter().collect()
    }

    pub fn current_visible_len(&self) -> usize {
        match self.section {
            Section::Containers => self.visible_containers().len(),
            Section::Images => self.visible_images().len(),
            Section::Volumes => self.visible_volumes().len(),
            Section::Networks => self.visible_networks().len(),
        }
    }

    pub fn selected_container(&self) -> Option<&Container> {
        self.visible_containers().get(self.selected).copied()
    }

    pub fn selected_image(&self) -> Option<&Image> {
        self.visible_images().get(self.selected).copied()
    }

    pub fn selected_volume(&self) -> Option<&Volume> {
        self.visible_volumes().get(self.selected).copied()
    }

    pub fn selected_network(&self) -> Option<&Network> {
        self.visible_networks().get(self.selected).copied()
    }

    // ── Refresh ──────────────────────────────────────────────────────────────

    pub async fn refresh_current_section(&mut self) {
        match self.section {
            Section::Containers => self.refresh_containers().await,
            Section::Images => self.refresh_images().await,
            Section::Volumes => self.refresh_volumes().await,
            Section::Networks => self.refresh_networks().await,
        }
    }

    pub async fn refresh_containers(&mut self) {
        match self
            .docker
            .list_containers(Some(
                ListContainersOptionsBuilder::default().all(true).build(),
            ))
            .await
        {
            Ok(list) => {
                self.containers = list.into_iter().map(Container::from).collect();
                self.containers.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                self.clamp_selected();
                self.status = None;
            }
            Err(e) => self.status = Some(format!("error: {e}")),
        }
    }

    async fn refresh_images(&mut self) {
        match self
            .docker
            .list_images(Some(
                ListImagesOptionsBuilder::default().all(false).build(),
            ))
            .await
        {
            Ok(list) => {
                self.images = list.into_iter().map(Image::from).collect();
                self.images.sort_unstable_by(|a, b| {
                    a.repository.cmp(&b.repository).then(a.tag.cmp(&b.tag))
                });
                self.clamp_selected();
                self.status = None;
            }
            Err(e) => self.status = Some(format!("error: {e}")),
        }
    }

    async fn refresh_volumes(&mut self) {
        match self.docker.list_volumes(None::<bollard::query_parameters::ListVolumesOptions>).await {
            Ok(resp) => {
                self.volumes = resp
                    .volumes
                    .unwrap_or_default()
                    .into_iter()
                    .map(Volume::from)
                    .collect();
                self.volumes.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                self.clamp_selected();
                self.status = None;
            }
            Err(e) => self.status = Some(format!("error: {e}")),
        }
    }

    async fn refresh_networks(&mut self) {
        match self.docker.list_networks(None::<bollard::query_parameters::ListNetworksOptions>).await {
            Ok(list) => {
                self.networks = list.into_iter().map(Network::from).collect();
                self.networks.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                self.clamp_selected();
                self.status = None;
            }
            Err(e) => self.status = Some(format!("error: {e}")),
        }
    }

    fn clamp_selected(&mut self) {
        let len = self.current_visible_len();
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }

    // ── Async message channel ─────────────────────────────────────────────────

    /// Receive the next background task message. Blocks until a message arrives.
    /// Safe to use in a tokio::select! arm alongside other arms.
    pub async fn recv_msg(&mut self) -> Option<AppMsg> {
        self.msg_rx.recv().await
    }

    /// Process a completed command result, updating spinner and triggering a refresh.
    pub async fn handle_cmd_result(&mut self, result: CmdResult) {
        self.pending_count = self.pending_count.saturating_sub(1);
        match result {
            CmdResult::Done { refresh } => {
                self.status = None;
                match refresh {
                    Section::Containers => self.refresh_containers().await,
                    Section::Images => self.refresh_images().await,
                    Section::Volumes => self.refresh_volumes().await,
                    Section::Networks => self.refresh_networks().await,
                }
            }
            CmdResult::Failed { message } => {
                self.status = Some(message);
            }
        }
    }

    // ── Log streaming ─────────────────────────────────────────────────────────

    pub fn start_log_stream(&mut self, full_id: String) {
        self.log_lines.clear();
        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        self.log_stop_tx = Some(stop_tx);

        let msg_tx = self.msg_tx.clone();
        let docker = Arc::clone(&self.docker);

        tokio::spawn(async move {
            let opts = LogsOptionsBuilder::default()
                .follow(true)
                .stdout(true)
                .stderr(true)
                .tail("200")
                .build();
            let mut stream = docker.logs(&full_id, Some(opts)).boxed();
            let mut stop_rx = stop_rx;
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    chunk = stream.next() => {
                        match chunk {
                            Some(Ok(c)) => {
                                if msg_tx.send(AppMsg::LogLine(c.to_string())).await.is_err() {
                                    break;
                                }
                            }
                            _ => break,
                        }
                    }
                }
            }
        });
    }

    pub fn stop_log_stream(&mut self) {
        self.log_stop_tx = None; // drop signals the task to stop
    }

    pub fn push_log_line(&mut self, raw: String) {
        for line in raw.lines() {
            self.log_lines.push(line.to_string());
        }
        if self.log_lines.len() > 5_000 {
            let excess = self.log_lines.len() - 5_000;
            self.log_lines.drain(0..excess);
        }
        if let Mode::Log { follow: true, scroll } = &mut self.mode {
            *scroll = self.log_lines.len().saturating_sub(1);
        }
    }

    // ── Input dispatch ────────────────────────────────────────────────────────

    pub async fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        let action = map_key(&self.mode, key);
        self.dispatch(action).await
    }

    async fn dispatch(&mut self, action: Action) -> Result<bool> {
        match self.mode.clone() {
            Mode::Normal => self.dispatch_normal(action).await,
            Mode::Filter(_) => self.dispatch_filter(action),
            Mode::Command(_) => self.dispatch_command(action).await,
            Mode::Log { .. } => self.dispatch_log(action),
            Mode::Confirm(pending) => self.dispatch_confirm(action, pending),
            Mode::Menu { cursor } => self.dispatch_menu(action, cursor),
            Mode::Visual { anchor, cursor } => self.dispatch_visual(action, anchor, cursor),
        }
    }

    async fn dispatch_normal(&mut self, action: Action) -> Result<bool> {
        let len = self.current_visible_len();
        match action {
            Action::Quit => return Ok(true),
            Action::MoveDown => {
                if len > 0 {
                    self.selected = (self.selected + 1).min(len - 1);
                }
            }
            Action::MoveUp => {
                self.selected = self.selected.saturating_sub(1);
            }
            Action::Top => self.selected = 0,
            Action::Bottom => self.selected = len.saturating_sub(1),
            Action::HalfPageDown => {
                self.selected = (self.selected + 10).min(len.saturating_sub(1));
            }
            Action::HalfPageUp => {
                self.selected = self.selected.saturating_sub(10);
            }
            Action::Delete => self.begin_delete(),
            Action::ToggleStartStop => {
                if self.section == Section::Containers {
                    if let Some(c) = self.selected_container().cloned() {
                        let name = c.name.clone();
                        self.spawn_toggle(c.full_id, c.state, name);
                    }
                }
            }
            Action::Restart => {
                if self.section == Section::Containers {
                    if let Some(c) = self.selected_container().cloned() {
                        let name = c.name.clone();
                        self.spawn_restart(c.full_id, name);
                    }
                }
            }
            Action::OpenLogs | Action::Enter => {
                if self.section == Section::Containers {
                    if let Some(c) = self.selected_container().cloned() {
                        self.start_log_stream(c.full_id);
                        self.mode = Mode::Log { scroll: 0, follow: true };
                    }
                }
            }
            Action::Exec => {
                if self.section == Section::Containers {
                    if let Some(c) = self.selected_container() {
                        if c.state == ContainerState::Running {
                            self.pending_exec = Some(c.full_id.clone());
                        } else {
                            self.status = Some(format!("{} is not running", c.name));
                        }
                    }
                }
            }
            Action::Visual => {
                if self.section == Section::Containers && !self.visible_containers().is_empty() {
                    self.mode = Mode::Visual { anchor: self.selected, cursor: self.selected };
                }
            }
            Action::Refresh => self.refresh_current_section().await,
            Action::EnterCommand => self.mode = Mode::Command(String::new()),
            Action::EnterFilter => self.mode = Mode::Filter(String::new()),
            Action::OpenMenu => self.mode = Mode::Menu { cursor: self.section.index() },
            _ => {}
        }
        Ok(false)
    }

    fn dispatch_visual(&mut self, action: Action, anchor: usize, cursor: usize) -> Result<bool> {
        let len = self.current_visible_len();
        match action {
            Action::Escape => {
                self.selected = cursor;
                self.mode = Mode::Normal;
            }
            Action::MoveDown => {
                let new = (cursor + 1).min(len.saturating_sub(1));
                self.mode = Mode::Visual { anchor, cursor: new };
            }
            Action::MoveUp => {
                let new = cursor.saturating_sub(1);
                self.mode = Mode::Visual { anchor, cursor: new };
            }
            Action::Top => {
                self.mode = Mode::Visual { anchor, cursor: 0 };
            }
            Action::Bottom => {
                self.mode = Mode::Visual { anchor, cursor: len.saturating_sub(1) };
            }
            Action::Delete => {
                let lo = anchor.min(cursor);
                let hi = anchor.max(cursor).min(len.saturating_sub(1));
                let visible = self.visible_containers();
                let ids: Vec<String> = visible[lo..=hi]
                    .iter()
                    .map(|c| c.full_id.clone())
                    .collect();
                let count = ids.len();
                self.mode = Mode::Confirm(PendingAction::BulkRemoveContainers { ids, count });
                self.selected = lo;
            }
            Action::ToggleStartStop => {
                let lo = anchor.min(cursor);
                let hi = anchor.max(cursor).min(len.saturating_sub(1));
                let visible = self.visible_containers();
                let targets: Vec<(String, String, ContainerState)> = visible[lo..=hi]
                    .iter()
                    .map(|c| (c.full_id.clone(), c.name.clone(), c.state.clone()))
                    .collect();
                self.mode = Mode::Normal;
                self.selected = lo;
                for (id, name, state) in targets {
                    self.spawn_toggle(id, state, name);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn begin_delete(&mut self) {
        match self.section {
            Section::Containers => {
                if let Some(c) = self.selected_container() {
                    let (key, display) = (c.full_id.clone(), c.name.clone());
                    self.mode = Mode::Confirm(PendingAction::Remove { key, display });
                }
            }
            Section::Images => {
                if let Some(img) = self.selected_image() {
                    let display = format!("{}:{}", img.repository, img.tag);
                    let key = img.remove_key.clone();
                    self.mode = Mode::Confirm(PendingAction::Remove { key, display });
                }
            }
            Section::Volumes => {
                if let Some(v) = self.selected_volume() {
                    let (key, display) = (v.name.clone(), v.name.clone());
                    self.mode = Mode::Confirm(PendingAction::Remove { key, display });
                }
            }
            Section::Networks => {
                if let Some(n) = self.selected_network() {
                    let (key, display) = (n.id.clone(), n.name.clone());
                    self.mode = Mode::Confirm(PendingAction::Remove { key, display });
                }
            }
        }
    }

    fn dispatch_filter(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Escape | Action::Enter => {
                self.mode = Mode::Normal;
                self.selected = 0;
            }
            Action::Char(c) => {
                if let Mode::Filter(ref mut s) = self.mode {
                    s.push(c);
                }
                self.selected = 0;
            }
            Action::Backspace => {
                if let Mode::Filter(ref mut s) = self.mode {
                    s.pop();
                }
                self.selected = 0;
            }
            _ => {}
        }
        Ok(false)
    }

    async fn dispatch_command(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Escape => {
                self.history.reset_cursor();
                self.mode = Mode::Normal;
            }
            Action::Enter => {
                let cmd = if let Mode::Command(ref s) = self.mode {
                    s.trim().to_string()
                } else {
                    String::new()
                };
                self.history.reset_cursor();
                self.mode = Mode::Normal;
                if !cmd.is_empty() {
                    self.history.push(cmd.clone());
                }
                return self.execute_command(&cmd).await;
            }
            Action::Char(c) => {
                self.history.reset_cursor();
                if let Mode::Command(ref mut s) = self.mode {
                    s.push(c);
                }
            }
            Action::Backspace => {
                if let Mode::Command(ref mut s) = self.mode {
                    s.pop();
                }
            }
            Action::HistoryPrev => {
                let current = if let Mode::Command(ref s) = self.mode {
                    s.clone()
                } else {
                    String::new()
                };
                if let Some(entry) = self.history.prev(&current) {
                    let entry = entry.to_string();
                    self.mode = Mode::Command(entry);
                }
            }
            Action::HistoryNext => {
                if let Some(entry) = self.history.next() {
                    let entry = entry.to_string();
                    self.mode = Mode::Command(entry);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn dispatch_log(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Quit | Action::Escape => {
                self.stop_log_stream();
                self.mode = Mode::Normal;
            }
            Action::MoveDown => {
                if let Mode::Log { scroll, follow } = &mut self.mode {
                    *follow = false;
                    *scroll = (*scroll + 1).min(self.log_lines.len().saturating_sub(1));
                }
            }
            Action::MoveUp => {
                if let Mode::Log { scroll, follow } = &mut self.mode {
                    *follow = false;
                    *scroll = scroll.saturating_sub(1);
                }
            }
            Action::Bottom => {
                if let Mode::Log { scroll, follow } = &mut self.mode {
                    *scroll = self.log_lines.len().saturating_sub(1);
                    *follow = true;
                }
            }
            Action::ToggleFollow => {
                if let Mode::Log { follow, scroll } = &mut self.mode {
                    *follow = !*follow;
                    if *follow {
                        *scroll = self.log_lines.len().saturating_sub(1);
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn dispatch_confirm(&mut self, action: Action, pending: PendingAction) -> Result<bool> {
        match action {
            Action::Confirm => {
                self.mode = Mode::Normal;
                match pending {
                    PendingAction::Remove { key, display } => {
                        match self.section {
                            Section::Containers => self.spawn_remove_container(key, display),
                            Section::Images => self.spawn_remove_image(key, display),
                            Section::Volumes => self.spawn_remove_volume(key),
                            Section::Networks => self.spawn_remove_network(key, display),
                        }
                    }
                    PendingAction::BulkRemoveContainers { ids, count } => {
                        for (i, id) in ids.into_iter().enumerate() {
                            let label = format!("removing container {} of {count}", i + 1);
                            self.spawn_remove_container(id, label);
                        }
                    }
                }
            }
            Action::Cancel | Action::Escape => {
                self.mode = Mode::Normal;
                self.status = Some("cancelled".into());
            }
            _ => {}
        }
        Ok(false)
    }

    fn dispatch_menu(&mut self, action: Action, cursor: usize) -> Result<bool> {
        let n = Section::ALL.len();
        match action {
            Action::Escape | Action::OpenMenu => {
                self.mode = Mode::Normal;
            }
            Action::MoveDown => {
                self.mode = Mode::Menu { cursor: (cursor + 1) % n };
            }
            Action::MoveUp => {
                self.mode = Mode::Menu { cursor: (cursor + n - 1) % n };
            }
            Action::Enter | Action::Confirm => {
                self.section = Section::from_index(cursor);
                self.mode = Mode::Normal;
                self.selected = 0;
                self.status = None;
                self.needs_refresh = true;
            }
            _ => {}
        }
        Ok(false)
    }

    async fn execute_command(&mut self, cmd: &str) -> Result<bool> {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        match parts.as_slice() {
            ["q"] | ["quit"] | [":q"] | [":quit"] => return Ok(true),
            _ => self.status = Some(format!("unknown command: :{cmd}")),
        }
        Ok(false)
    }

    // ── Background task spawning ──────────────────────────────────────────────

    /// Spawn a Docker command as a background task. The task sends a CmdResult
    /// through the app message channel when done, which the event loop processes.
    fn spawn_cmd<F, Fut>(&mut self, label: impl Into<String>, refresh: Section, f: F)
    where
        F: FnOnce(Arc<Docker>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), String>> + Send + 'static,
    {
        let label = label.into();
        let docker = Arc::clone(&self.docker);
        let tx = self.msg_tx.clone();
        self.pending_count += 1;
        self.spinner_label = label;

        tokio::spawn(async move {
            let result = match f(docker).await {
                Ok(()) => CmdResult::Done { refresh },
                Err(msg) => CmdResult::Failed { message: format!("error: {msg}") },
            };
            let _ = tx.send(AppMsg::Cmd(result)).await;
        });
    }

    fn spawn_toggle(&mut self, full_id: String, state: ContainerState, name: String) {
        let label = if state == ContainerState::Running {
            format!("stopping {name}")
        } else {
            format!("starting {name}")
        };
        self.spawn_cmd(label, Section::Containers, move |docker| async move {
            let r = if state == ContainerState::Running {
                docker
                    .stop_container(
                        &full_id,
                        Some(StopContainerOptionsBuilder::default().t(10).build()),
                    )
                    .await
            } else {
                docker.start_container(&full_id, None).await
            };
            r.map_err(|e| e.to_string())
        });
    }

    fn spawn_restart(&mut self, full_id: String, name: String) {
        self.spawn_cmd(format!("restarting {name}"), Section::Containers, move |docker| async move {
            docker.restart_container(&full_id, None).await.map_err(|e| e.to_string())
        });
    }

    fn spawn_remove_container(&mut self, full_id: String, display: String) {
        self.spawn_cmd(format!("removing {display}"), Section::Containers, move |docker| async move {
            docker
                .remove_container(
                    &full_id,
                    Some(RemoveContainerOptionsBuilder::default().force(true).build()),
                )
                .await
                .map_err(|e| e.to_string())
        });
    }

    fn spawn_remove_image(&mut self, key: String, display: String) {
        self.spawn_cmd(format!("removing {display}"), Section::Images, move |docker| async move {
            docker
                .remove_image(
                    &key,
                    Some(RemoveImageOptionsBuilder::default().force(true).build()),
                    None,
                )
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
    }

    fn spawn_remove_volume(&mut self, name: String) {
        let label = format!("removing {name}");
        self.spawn_cmd(label, Section::Volumes, move |docker| async move {
            docker
                .remove_volume(&name, None::<bollard::query_parameters::RemoveVolumeOptions>)
                .await
                .map_err(|e| e.to_string())
        });
    }

    fn spawn_remove_network(&mut self, id: String, name: String) {
        self.spawn_cmd(format!("removing {name}"), Section::Networks, move |docker| async move {
            docker.remove_network(&id).await.map_err(|e| e.to_string())
        });
    }
}
