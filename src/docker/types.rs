use bollard::models::ContainerSummary;

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

impl ContainerState {
    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        match self {
            Self::Running => "running",
            Self::Exited => "exited",
            Self::Paused => "paused",
            Self::Restarting => "restarting",
            Self::Dead => "dead",
            Self::Created => "created",
            Self::Unknown(s) => s.as_str(),
        }
    }
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

        let ports = s
            .ports
            .unwrap_or_default()
            .iter()
            .filter_map(|p| p.public_port.map(|pub_port| format!("{}→{}", pub_port, p.private_port)))
            .collect::<Vec<_>>()
            .join(" ");

        Self { id, full_id, name, image, state, status_text, ports }
    }
}
