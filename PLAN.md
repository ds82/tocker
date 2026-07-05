# tocker — Docker TUI Client

A vim-inspired terminal UI for Docker, built with Rust and Ratatui.

---

## Vision

`tocker` feels like home for anyone who lives in Neovim. Modal navigation, mnemonic keybindings, command-line mode, and a composable layout. No mouse required. No surprises.

---

## UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│ tocker                              [normal] containers (42)  │
├──────────┬──────────────────────────────────────────────────┤
│ SIDEBAR  │ MAIN PANEL                                        │
│          │                                                   │
│ Containers│ NAME        IMAGE      STATUS    PORTS           │
│ Images   │ ▶ nginx      nginx:latest  running  0.0.0.0:80->80│
│ Volumes  │   postgres   pg:15     exited    5432             │
│ Networks │   redis      redis:7   running   6379             │
│          │                                                   │
│          │                                                   │
├──────────┴──────────────────────────────────────────────────┤
│ DETAIL PANEL (toggleable)                                    │
│ ID: abc123  Created: 2d ago  Mounts: /data -> /var/lib/pg   │
├─────────────────────────────────────────────────────────────┤
│ :                                        [status messages]   │
└─────────────────────────────────────────────────────────────┘
```

Panels are resizable with `<` / `>`. Detail panel toggles with `K` (think `K` for "info" in vim).

---

## Modes

| Mode    | Trigger         | Purpose                           |
| ------- | --------------- | --------------------------------- |
| Normal  | `<Esc>`         | Navigate, select, act             |
| Command | `:`             | Run `:pull nginx`, `:rm <id>`     |
| Filter  | `/`             | Fuzzy-filter the current list     |
| Visual  | `v`             | Select multiple items             |
| Log     | `Enter` on ctr. | Stream container logs (read-only) |

---

## Keybindings

### Navigation (Normal mode)

| Key       | Action                            |
| --------- | --------------------------------- |
| `j` / `k` | Move down / up in list            |
| `g` / `G` | Jump to top / bottom              |
| `h` / `l` | Switch sidebar focus / main panel |
| `Tab`     | Cycle sidebar sections            |
| `Ctrl-d`  | Half-page down                    |
| `Ctrl-u`  | Half-page up                      |

### Actions (Normal mode)

| Key | Action                            |
| --- | --------------------------------- |
| `s` | Start / stop container (toggle)   |
| `r` | Restart container                 |
| `d` | Delete (with confirmation prompt) |
| `e` | Exec into container (opens shell) |
| `l` | View logs (enters Log mode)       |
| `i` | Inspect (opens detail panel)      |
| `p` | Pull image (command pre-filled)   |
| `R` | Refresh current view              |
| `q` | Quit                              |
| `?` | Open keybinding help overlay      |

### Visual mode (multi-select)

| Key       | Action                  |
| --------- | ----------------------- |
| `j` / `k` | Extend selection        |
| `d`       | Delete all selected     |
| `s`       | Start/stop all selected |
| `<Esc>`   | Return to Normal mode   |

### Log mode

| Key         | Action                |
| ----------- | --------------------- |
| `j` / `k`   | Scroll                |
| `G`         | Jump to latest (tail) |
| `f`         | Toggle follow mode    |
| `/`         | Search within logs    |
| `q`/`<Esc>` | Close logs            |

### Command mode

| Key       | Action          |
| --------- | --------------- |
| `<Enter>` | Execute command |
| `<Esc>`   | Cancel          |
| `<Tab>`   | Autocomplete    |
| `↑` / `↓` | Command history |

#### Built-in commands

```
:q, :quit         Quit
:pull <image>     Pull an image
:rm <id>          Remove container/image
:run <image>      Run a new container (opens form)
:prune            Remove all stopped containers (confirm)
:logs <id>        Open logs for container
:exec <id>        Exec into container
```

---

## Architecture

```
tocker/
├── src/
│   ├── main.rs              Entry point, event loop
│   ├── app.rs               App state, mode FSM
│   ├── docker/
│   │   ├── mod.rs
│   │   ├── client.rs        Thin async wrapper over bollard
│   │   └── types.rs         Domain types (Container, Image, …)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs        Root layout composition
│   │   ├── sidebar.rs       Sidebar widget
│   │   ├── list.rs          Generic scrollable list widget
│   │   ├── detail.rs        Detail panel widget
│   │   ├── logs.rs          Log viewer widget
│   │   ├── command.rs       Command bar widget
│   │   └── help.rs          Help overlay widget
│   ├── input/
│   │   ├── mod.rs
│   │   ├── keybindings.rs   Key → Action mapping
│   │   └── command.rs       Command parser
│   └── config.rs            Config file (~/.config/tocker/config.toml)
├── Cargo.toml
└── PLAN.md
```

---

## Tech Stack

| Concern          | Crate                       | Notes                               |
| ---------------- | --------------------------- | ----------------------------------- |
| TUI rendering    | `ratatui`                   | Widgets, layout, styling            |
| Terminal backend | `crossterm`                 | Cross-platform terminal control     |
| Docker API       | `bollard`                   | Async Docker client (Unix socket)   |
| Async runtime    | `tokio`                     | Multi-threaded, drives Docker calls |
| Config           | `toml` + `serde`            | `~/.config/tocker/config.toml`      |
| Error handling   | `anyhow`                    | Ergonomic error propagation         |
| Fuzzy filter     | `nucleo` or `fuzzy-matcher` | Fast in-process fuzzy search        |
| PTY exec         | `portable-pty`              | Proper PTY for `e` exec-into-shell  |

---

## State Machine

```
          ┌────────────────────────────────────────────┐
          │                  Normal                     │◀─────┐
          └──┬──────┬──────┬───────┬──────┬────────────┘      │
             │      │      │       │      │                    │
             ▼      ▼      ▼       ▼      ▼                  Esc
          Visual  Filter  Cmd    Log    Help                   │
                   (/)    (:)   (Enter) (?)                    │
             │      │      │       │      │                    │
             └──────┴──────┴───────┴──────┴────────────────────┘
```

Mode is stored as an enum on `App`. Each mode intercepts key events before the normal handler.

---

## Event Loop

```
tokio::select! {
    key_event   => handle_input(key, &mut app)
    docker_tick => refresh_docker_state(&mut app)   // every 2s
    log_chunk   => append_to_log_view(&mut app)     // streaming
}
terminal.draw(|f| ui::render(f, &app));
```

Docker state is refreshed on a background interval. Log streaming uses a bounded channel fed by a spawned task.

---

## Docker Host Resolution

Priority order (first match wins):

1. `DOCKER_HOST` environment variable
2. `[remote] host` in `~/.config/tocker/config.toml`
3. Local Unix socket (`/var/run/docker.sock`)

### `DOCKER_HOST` parsing

| Value                        | Interpreted as                  |
| ---------------------------- | ------------------------------- |
| `hostname`                   | SSH to `hostname`               |
| `user@hostname`              | SSH to `hostname` as `user`     |
| `ssh://user@hostname:port`   | SSH, explicit scheme            |
| `unix:///path/to/sock`       | Custom local socket             |
| `tcp://hostname:port`        | Unencrypted TCP (Docker daemon) |

A bare hostname (no scheme) is always treated as SSH — consistent with how you'd think of a remote box. `bollard`'s `ClientBuilder` handles the SSH tunnel; it reuses the system SSH agent and `~/.ssh/config` so no credentials need to be configured in tocker itself.

---

## Config (`~/.config/tocker/config.toml`)

```toml
[general]
refresh_interval_ms = 2000
default_section = "containers"   # containers | images | volumes | networks

[remote]
# Overridden by DOCKER_HOST env var. Bare hostname = SSH.
# host = "user@myserver"

[theme]
selected_fg = "Yellow"
selected_bg = "DarkGray"
status_running = "Green"
status_exited  = "Red"
status_paused  = "Yellow"

[keybindings]
# Override defaults — same action names as the table above
quit = "q"
exec = "e"
```

---

## Implementation Phases

### Phase 1 — Skeleton (MVP)

- [x] Project scaffold (`cargo new`, dependencies)
- [x] `DOCKER_HOST` env var parsing → local socket or SSH via `bollard`
- [x] Basic event loop with Ratatui + crossterm
- [x] Normal / Command / Filter mode FSM
- [x] Container list view with live refresh
- [x] Start / stop / remove actions
- [x] Log viewer (tail + follow)

### Phase 2 — Full Resource Coverage

- [x] Images view (list, remove)
- [x] Volumes view
- [x] Networks view
- [x] Tab navigation between sections

### Phase 3 — Power Features

- [x] Visual mode (multi-select with bulk start/stop/delete)
- [x] Exec into container (`docker exec -it … sh` via subprocess)
- [x] Filter with `/` (substring match, clears on Esc)
- [x] Command history (`~/.local/share/tocker/history`, up/down navigation)
- [x] Config file (`~/.config/tocker/config.toml` — `refresh_interval_ms`, `default_section`)
- [x] Async Docker operations — UI never blocks; spinner shows in-flight commands
- [x] Clipboard yank via OSC 52 (`yy` name, `yi` secondary, `yd` ID)

### Phase 4 — Polish

- [ ] Help overlay (`?`)
- [ ] Detail / inspect panel
- [ ] Error display (non-fatal toasts, fatal full-screen)
- [ ] Theme customization
- [ ] Docker Compose awareness (group containers by compose project)

---

## Decisions

| Topic               | Decision                                                                                              |
| ------------------- | ----------------------------------------------------------------------------------------------------- |
| **Exec shell**      | `portable-pty` — proper PTY, cleaner UX                                                               |
| **Remote daemons**  | `DOCKER_HOST` env var (Phase 1); config file `[remote]` block as fallback; env var always wins       |
| **Config location** | XDG — `~/.config/tocker/config.toml`                                                                  |
| **Compose groups**  | Opt-in for now; auto-detect `com.docker.compose.project` labels is a Phase 4 stretch goal             |
