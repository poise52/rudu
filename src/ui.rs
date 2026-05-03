use crate::app::{App, ScanState};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::time::Duration;

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

pub fn run(app: &mut App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        app.tick();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(f.area());

            let title = match app.scan_state {
                ScanState::Scanning => {
                    let spinner = SPINNER[app.spinner_tick as usize % SPINNER.len()];
                    format!(" {} scanning {}... ", spinner, app.current_path.to_str().unwrap_or(""))
                }
                ScanState::Done => {
                    format!(" {} ", app.current_path.to_str().unwrap_or(""))
                }
            };

            let path_block = Block::default()
                .title(title)
                .borders(Borders::ALL);
            f.render_widget(path_block, chunks[0]);

            let items: Vec<ListItem> = app.entries
                .iter()
                .map(|e| {
                    let name = e.path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    let icon = if e.is_dir { "📁" } else { "📄" };
                    ListItem::new(format!("{} {:<40} {:>12}", icon, name, format_size(e.size)))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("rudu").borders(Borders::ALL))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[1], &mut app.list_state);

            let hints = Paragraph::new(Line::from(
                "↑↓ выбор  ·  Enter — в папку  ·  Backspace — назад  ·  r — обновить  ·  q — выход",
            ))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" клавиши ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL),
            );
            f.render_widget(hints, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => app.move_up(),
                    KeyCode::Down => app.move_down(),
                    KeyCode::Enter => app.navigate_into(),
                    KeyCode::Backspace => app.navigate_back(),
                    KeyCode::Char('r') => app.refresh_scan(),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}