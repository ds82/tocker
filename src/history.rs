use std::path::PathBuf;

const MAX_ENTRIES: usize = 200;

pub struct History {
    entries: Vec<String>,
    cursor: usize,
    fresh_input: String,
    path: PathBuf,
}

impl History {
    pub fn load() -> Self {
        Self::load_named("history")
    }

    pub fn load_named(filename: &str) -> Self {
        let path = history_dir().join(filename);
        let entries = if path.exists() {
            std::fs::read_to_string(&path)
                .unwrap_or_default()
                .lines()
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()
        } else {
            Vec::new()
        };
        let cursor = entries.len();
        Self { entries, cursor, fresh_input: String::new(), path }
    }

    pub fn last(&self) -> Option<&str> {
        self.entries.last().map(|s| s.as_str())
    }

    pub fn push(&mut self, cmd: String) {
        if cmd.is_empty() {
            return;
        }
        if self.entries.last().map(|s| s == &cmd).unwrap_or(false) {
            self.reset_cursor();
            return;
        }
        self.entries.push(cmd);
        if self.entries.len() > MAX_ENTRIES {
            self.entries.remove(0);
        }
        self.reset_cursor();
        let _ = self.save();
    }

    /// Navigate to an older entry. Returns the entry text, or None if already at the oldest.
    pub fn prev(&mut self, current_input: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        if self.cursor == self.entries.len() {
            self.fresh_input = current_input.to_string();
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        Some(&self.entries[self.cursor])
    }

    /// Navigate to a newer entry. Returns None when back at fresh input.
    pub fn next(&mut self) -> Option<&str> {
        if self.cursor >= self.entries.len() {
            return None;
        }
        self.cursor += 1;
        if self.cursor == self.entries.len() {
            Some(&self.fresh_input)
        } else {
            Some(&self.entries[self.cursor])
        }
    }

    pub fn reset_cursor(&mut self) {
        self.cursor = self.entries.len();
        self.fresh_input.clear();
    }

    fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, self.entries.join("\n"))
    }
}

fn history_dir() -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".local").join("share"))
                .unwrap_or_else(|_| PathBuf::from("."))
        });
    base.join("tocker")
}
