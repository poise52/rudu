use crate::app::App;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use std::io;

pub fn run(app: &mut App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Percentage(100)])
                .split(f.area());

            let path_block = Block::default()
                .title(app.current_path.to_str().unwrap_or(""))
                .borders(Borders::ALL);
            f.render_widget(path_block, chunks[0]);

            let items: Vec<ListItem> = app.entries
                .iter()
                .map(|e| {
                    let name = e.path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    let size_mb = e.size / 1024 / 1024;
                    let icon = if e.is_dir { "📁" } else { "📄" };
                    ListItem::new(format!("{} {:<40} {} MB", icon, name, size_mb))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("duwatch").borders(Borders::ALL))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[1], &mut app.list_state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => app.move_up(),
                KeyCode::Down => app.move_down(),
                KeyCode::Enter => app.navigate_into(),
                KeyCode::Backspace => app.navigate_back(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}