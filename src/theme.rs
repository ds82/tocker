use ratatui::style::Color;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ThemeConfig {
    pub selected_bg: String,
    pub status_running: String,
    pub status_exited: String,
    pub status_paused: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            selected_bg: "DarkGray".into(),
            status_running: "Green".into(),
            status_exited: "Red".into(),
            status_paused: "Yellow".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub selected_bg: Color,
    pub status_running: Color,
    pub status_exited: Color,
    pub status_paused: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::from(&ThemeConfig::default())
    }
}

impl From<&ThemeConfig> for Theme {
    fn from(cfg: &ThemeConfig) -> Self {
        Self {
            selected_bg: parse_color(&cfg.selected_bg),
            status_running: parse_color(&cfg.status_running),
            status_exited: parse_color(&cfg.status_exited),
            status_paused: parse_color(&cfg.status_paused),
        }
    }
}

fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "darkgray" | "dark_gray" | "darkgrey" | "dark_grey" => Color::DarkGray,
        "lightred" | "light_red" => Color::LightRed,
        "lightgreen" | "light_green" => Color::LightGreen,
        "lightyellow" | "light_yellow" => Color::LightYellow,
        "lightblue" | "light_blue" => Color::LightBlue,
        "lightmagenta" | "light_magenta" => Color::LightMagenta,
        "lightcyan" | "light_cyan" => Color::LightCyan,
        "gray" | "grey" => Color::Gray,
        _ => Color::Reset,
    }
}
