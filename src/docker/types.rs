use bollard::models::{
    ContainerSummary, ImageSummary, Network as BollardNetwork, Volume as BollardVolume,
};

// ── Containers ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Container {
    #[allow(dead_code)]
    pub id: String,
    pub full_id: String,
    pub name: String,
    pub image: String,
    pub state: ContainerState,
    pub status_text: String,
    pub ports: String,
    pub compose_project: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerState {
    Running,
    Exited,
    Paused,
    Restarting,
    Dead,
    Created,
    Unknown(String),
}

impl From<&str> for ContainerState {
    fn from(s: &str) -> Self {
        match s {
            "running" => Self::Running,
            "exited" => Self::Exited,
            "paused" => Self::Paused,
            "restarting" => Self::Restarting,
            "dead" => Self::Dead,
            "created" => Self::Created,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl From<ContainerSummary> for Container {
    fn from(s: ContainerSummary) -> Self {
        let full_id = s.id.unwrap_or_default();
        let id: String = full_id.chars().take(12).collect();

        let name = s
            .names
            .unwrap_or_default()
            .into_iter()
            .next()
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| id.clone());

        let image = s.image.unwrap_or_default();
        let state = s
            .state
            .as_ref()
            .map(|e| ContainerState::from(e.to_string().as_str()))
            .unwrap_or(ContainerState::Unknown(String::new()));
        let status_text = s.status.unwrap_or_default();

        let mut port_pairs: Vec<(u16, u16)> = s
            .ports
            .unwrap_or_default()
            .iter()
            .filter_map(|p| p.public_port.map(|pp| (pp, p.private_port)))
            .collect();
        port_pairs.sort_unstable();
        let ports = port_pairs
            .iter()
            .map(|(pub_port, priv_port)| format!("{pub_port}→{priv_port}"))
            .collect::<Vec<_>>()
            .join(" ");

        let compose_project = s
            .labels
            .as_ref()
            .and_then(|l| l.get("com.docker.compose.project"))
            .cloned();

        Self {
            id,
            full_id,
            name,
            image,
            state,
            status_text,
            ports,
            compose_project,
        }
    }
}

// ── Images ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Image {
    pub id: String,
    /// Used for the remove API call — "repo:tag" or short ID for dangling images
    pub remove_key: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
    pub created: String,
}

impl From<ImageSummary> for Image {
    fn from(s: ImageSummary) -> Self {
        let full_id = s.id;
        let id: String = full_id
            .strip_prefix("sha256:")
            .unwrap_or(&full_id)
            .chars()
            .take(12)
            .collect();

        let first_tag = s.repo_tags.into_iter().next();

        let (repository, tag) = match first_tag.as_deref() {
            Some("<none>:<none>") | None => ("<none>".into(), "<none>".into()),
            Some(t) => match t.rsplit_once(':') {
                Some((repo, tag)) => (repo.to_string(), tag.to_string()),
                None => (t.to_string(), String::new()),
            },
        };

        let remove_key = if repository == "<none>" {
            id.clone()
        } else {
            format!("{repository}:{tag}")
        };

        Self {
            id,
            remove_key,
            repository,
            tag,
            size: format_bytes(s.size),
            created: format_age(s.created),
        }
    }
}

// ── Volumes ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Volume {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub scope: String,
}

impl From<BollardVolume> for Volume {
    fn from(v: BollardVolume) -> Self {
        let scope = v.scope.map(|s| s.to_string()).unwrap_or_default();

        Self {
            name: v.name,
            driver: v.driver,
            mountpoint: v.mountpoint,
            scope,
        }
    }
}

// ── Networks ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Network {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub subnet: String,
}

impl From<BollardNetwork> for Network {
    fn from(n: BollardNetwork) -> Self {
        let id = n.id.unwrap_or_default();
        let name = n.name.unwrap_or_default();
        let driver = n.driver.unwrap_or_default();
        let scope = n.scope.unwrap_or_default();

        let subnet = n
            .ipam
            .and_then(|ipam| ipam.config)
            .and_then(|cfgs| cfgs.into_iter().next())
            .and_then(|cfg| cfg.subnet)
            .unwrap_or_default();

        Self {
            id,
            name,
            driver,
            scope,
            subnet,
        }
    }
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn format_bytes(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes}B")
    }
}

fn format_age(unix_ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let diff = (now - unix_ts).max(0);
    if diff < 60 {
        format!("{diff}s")
    } else if diff < 3600 {
        format!("{}m", diff / 60)
    } else if diff < 86400 {
        format!("{}h", diff / 3600)
    } else {
        format!("{}d", diff / 86400)
    }
}
