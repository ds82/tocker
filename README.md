# rocker

A vim-inspired terminal UI for Docker, built with Rust and [Ratatui](https://github.com/ratatui-org/ratatui).

```
 [normal] containers                                          rocker
 NAME              IMAGE              STATUS    PORTS
 nginx             nginx:latest       running   0.0.0.0:80->80/tcp
 postgres          postgres:15        exited    5432/tcp
 redis             redis:7-alpine     running   6379/tcp

  s start/stop  r restart  d delete  l logs  e exec  / filter  Tab sections  q quit
```

## Features

- Modal navigation (Normal / Command / Filter / Visual / Log / Yank)
- Containers, images, volumes, and networks views
- Async Docker operations with spinner — the UI never blocks
- Log streaming with follow mode
- Exec into a container shell (`e`)
- Visual mode multi-select for bulk start/stop/delete
- Vim-style yank to clipboard (`yy`, `yi`, `yd`) via OSC 52
- Command history with up/down arrow navigation
- Filter with `/` — substring match, clears on Esc
- Remote Docker daemon via SSH (`DOCKER_HOST`)
- Config file at `~/.config/rocker/config.toml`
- Command history persisted at `~/.local/share/rocker/history`

## Install

```sh
cargo install --path .
```

Requires Rust 1.75+ and a running Docker daemon.

## Usage

```sh
# Local Docker socket
rocker

# Remote host over SSH
DOCKER_HOST=user@myserver rocker
DOCKER_HOST=ssh://user@myserver:2222 rocker
```

## Keybindings

### Normal mode

| Key | Action |
|-----|--------|
| `j` / `k` or `↓` / `↑` | Move down / up |
| `g` / `G` | Jump to top / bottom |
| `Ctrl-d` / `Ctrl-u` | Half-page down / up |
| `Tab` | Cycle sections (containers → images → volumes → networks) |
| `s` | Start / stop (toggle) |
| `r` | Restart |
| `d` | Delete (confirmation required) |
| `l` or `Enter` | View logs |
| `e` | Exec into container shell |
| `v` | Enter visual (multi-select) mode |
| `y` | Enter yank mode |
| `/` | Filter list |
| `:` | Command mode |
| `R` | Force refresh |
| `q` | Quit |

### Visual mode

| Key | Action |
|-----|--------|
| `j` / `k` | Extend selection |
| `g` / `G` | Extend to top / bottom |
| `s` | Start / stop all selected |
| `d` | Delete all selected |
| `Esc` | Return to Normal |

### Yank mode (`y` → …)

| Key | Containers | Images | Volumes | Networks |
|-----|-----------|--------|---------|---------|
| `y` | name | `repo:tag` | name | name |
| `i` | image | short ID | mountpoint | subnet |
| `d` | full container ID | short ID | — | — |

Copied via OSC 52 — works locally and over SSH. For tmux, add `set -g set-clipboard on` to `~/.tmux.conf`.

### Log mode

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll |
| `G` | Jump to latest |
| `f` | Toggle follow mode |
| `q` / `Esc` | Close |

### Command mode

| Key | Action |
|-----|--------|
| `Enter` | Execute |
| `↑` / `↓` | Navigate history |
| `Esc` | Cancel |

Built-in commands: `:q` / `:quit`.

## Config

`~/.config/rocker/config.toml` (all fields optional):

```toml
[general]
refresh_interval_ms = 2000                 # default: 2000
default_section = "containers"             # containers | images | volumes | networks
```

## Docker host resolution

Priority order (first match wins):

1. `DOCKER_HOST` environment variable
2. Local Unix socket at `/var/run/docker.sock`

| `DOCKER_HOST` value | Interpreted as |
|--------------------|---------------|
| `hostname` | SSH to hostname |
| `user@hostname` | SSH as user |
| `ssh://user@hostname:port` | SSH, explicit port |
| `unix:///path/to/sock` | Custom local socket |

SSH connections reuse `~/.ssh/config` and the system SSH agent — no credentials need to be configured in rocker.

## Tech stack

| Concern | Crate |
|---------|-------|
| TUI rendering | `ratatui 0.30` |
| Terminal backend | `crossterm 0.29` |
| Docker API | `bollard 0.21` |
| Async runtime | `tokio` (full) |
| Config | `toml` + `serde` |
| Error handling | `anyhow` |
