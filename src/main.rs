use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

mod app;
mod clipboard;
mod config;
mod docker;
mod history;
mod input;
mod theme;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::Config::load();
    let (docker, tunnel) = docker::client::connect().await?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Restore terminal on panic so the user's shell isn't left broken
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        orig_hook(info);
    }));

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let default_section = app::Section::from_str(&cfg.general.default_section);
    let theme = theme::Theme::from(&cfg.theme);
    let mut app = app::App::new(docker, tunnel, default_section, theme);
    let result = run(&mut terminal, &mut app, cfg.general.refresh_interval_ms).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(ref e) = result {
        eprintln!("error: {e:#}");
    }
    result
}

async fn run<B>(
    terminal: &mut Terminal<B>,
    app: &mut app::App,
    refresh_interval_ms: u64,
) -> Result<()>
where
    B: ratatui::backend::Backend + std::io::Write,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    use crossterm::event::{Event, EventStream};
    use futures::StreamExt;
    use tokio::time::{interval, Duration, MissedTickBehavior};

    let mut events = EventStream::new();
    let mut docker_tick = interval(Duration::from_millis(refresh_interval_ms));
    docker_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    // Spinner animation tick — only active while commands are pending
    let mut spin_tick = interval(Duration::from_millis(100));
    spin_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    app.refresh_current_section().await;

    loop {
        app.maybe_clear_status();
        terminal.draw(|f| ui::render(f, app))?;

        tokio::select! {
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        if app.handle_key(key).await? {
                            break;
                        }
                        if app.needs_refresh {
                            app.needs_refresh = false;
                            app.refresh_current_section().await;
                        }
                        if let Some((id, cmd, interactive)) = app.pending_exec.take() {
                            exec_container(&id, &cmd, interactive, terminal).await?;
                            app.refresh_containers().await;
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {}
                    _ => {}
                }
            }
            _ = docker_tick.tick() => {
                if matches!(
                    app.mode,
                    app::Mode::Normal | app::Mode::Filter(_) | app::Mode::Visual { .. }
                ) {
                    app.refresh_current_section().await;
                }
            }
            // Advance spinner frame — disabled when nothing is pending
            _ = spin_tick.tick(), if app.pending_count > 0 => {
                app.spinner_frame = app.spinner_frame.wrapping_add(1);
            }
            // Background task results: Docker command completions and log lines
            maybe_msg = app.recv_msg() => {
                if let Some(msg) = maybe_msg {
                    match msg {
                        app::AppMsg::LogLine(line) => app.push_log_line(line),
                        app::AppMsg::Cmd(result) => app.handle_cmd_result(result).await,
                    }
                }
            }
        }
    }

    Ok(())
}

async fn exec_container<B>(
    id: &str,
    cmd: &str,
    interactive: bool,
    terminal: &mut Terminal<B>,
) -> Result<()>
where
    B: ratatui::backend::Backend + std::io::Write,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    let cmd_args: Vec<&str> = cmd.split_whitespace().collect();
    let docker_flags: &[&str] = if interactive { &["exec", "-it"] } else { &["exec"] };
    let _ = tokio::process::Command::new("docker")
        .args(docker_flags)
        .arg(id)
        .args(&cmd_args)
        .status()
        .await;

    print!("\n[press enter to return]");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut buf = String::new();
    let _ = std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut buf);

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;

    Ok(())
}
