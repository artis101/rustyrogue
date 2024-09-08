use crate::game::Game;
use crate::tile::Tile;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use std::io;

pub struct TUI {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl TUI {
    pub fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(TUI { terminal })
    }

    pub fn run(&mut self, game: &mut Game) -> Result<(), io::Error> {
        loop {
            let map_widget = Self::prepare_map_widget(game);
            let info_widget = Self::prepare_info_widget(game);

            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                    .split(f.area());

                f.render_widget(map_widget, chunks[0]);
                f.render_widget(info_widget, chunks[1]);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Left => game.move_player(-1, 0),
                            KeyCode::Right => game.move_player(1, 0),
                            KeyCode::Up => game.move_player(0, -1),
                            KeyCode::Down => game.move_player(0, 1),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn prepare_map_widget(game: &Game) -> Paragraph<'static> {
        let map = game.get_map();
        let map_string: Vec<Line> = map
            .iter()
            .map(|row| {
                Line::from(
                    row.iter()
                        .map(|&tile| {
                            let style = Style::default().fg(tile.term_color());
                            Span::styled(tile.symbol().to_string(), style)
                        })
                        .collect::<Vec<Span>>(),
                )
            })
            .collect();

        Paragraph::new(map_string)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Rustyrogue"),
            )
            .style(Style::default().bg(Color::Black))
    }

    fn prepare_info_widget(game: &Game) -> Paragraph<'static> {
        let door_message = game
            .get_door_message()
            .unwrap_or_else(|| "No door nearby".to_string());
        Paragraph::new(door_message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Info"),
            )
            .style(Style::default().fg(Color::Yellow))
    }
}

impl Drop for TUI {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        self.terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)
            .unwrap();
    }
}
