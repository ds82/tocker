use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

mod app;
mod docker;
mod input;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
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

    let mut app = app::App::new(docker, tunnel);
    let result = run(&mut terminal, &mut app).await;

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
) -> Result<()>
where
    B: ratatui::backend::Backend,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    use crossterm::event::{Event, EventStream};
    use futures::StreamExt;
    use tokio::time::{interval, Duration, MissedTickBehavior};

    let mut events = EventStream::new();
    let mut docker_tick = interval(Duration::from_secs(2));
    docker_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    app.refresh_containers().await;

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        tokio::select! {
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        if app.handle_key(key).await? {
                            break;
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {} // ratatui redraws on next loop
                    _ => {}
                }
            }
            _ = docker_tick.tick() => {
                if matches!(app.mode, app::Mode::Normal | app::Mode::Filter(_))
                    && app.section == app::Section::Containers
                {
                    app.refresh_containers().await;
                }
            }
            maybe_line = app.recv_log_line() => {
                if let Some(line) = maybe_line {
                    app.push_log_line(line);
                }
            }
        }
    }

    Ok(())
}
