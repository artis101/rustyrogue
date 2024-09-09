use crate::game::Game;
use crate::tile::Tile;
use crate::widgets::inventory::InventoryWidget;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use std::io;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Tui { terminal })
    }

    pub fn run(&mut self, game: &mut Game) -> Result<(), io::Error> {
        loop {
            let map_widget = Self::prepare_map_widget(game);
            let info_widget = Self::prepare_inventory_widget(game);
            let game_log_widget = Self::prepare_game_log_widget(game);

            self.terminal.draw(|f| {
                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Min(0),    // For Map and Info
                            Constraint::Length(7), // For Log
                        ]
                        .as_ref(),
                    )
                    .split(f.area());

                let top_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(70), // Map takes 70% of the width
                            Constraint::Percentage(30), // Info takes 30% of the width
                        ]
                        .as_ref(),
                    )
                    .split(main_chunks[0]);

                f.render_widget(map_widget, top_chunks[0]);
                f.render_widget(info_widget, top_chunks[1]);
                f.render_widget(game_log_widget, main_chunks[1]);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Left | KeyCode::Char('h') => game.move_player(-1, 0),
                            KeyCode::Right | KeyCode::Char('l') => game.move_player(1, 0),
                            KeyCode::Up | KeyCode::Char('k') => game.move_player(0, -1),
                            KeyCode::Down | KeyCode::Char('j') => game.move_player(0, 1),
                            KeyCode::Char(' ') | KeyCode::Char('e') => game.interact(),
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
                        .map(|&tile: &Tile| {
                            let style = Style::default().fg(tile.term_fg()).bg(tile.term_bg());
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

    fn prepare_inventory_widget(game: &Game) -> InventoryWidget {
        InventoryWidget::new(game)
    }

    fn prepare_game_log_widget(game: &Game) -> Paragraph<'static> {
        let log_messages: Vec<_> = game
            .get_log_messages()
            .iter()
            .map(|message| {
                let color = match message.message_type {
                    crate::game::MessageType::Info => Color::Gray,
                    crate::game::MessageType::Damage => Color::Red,
                };
                let style = Style::default().fg(color);
                Line::from(vec![Span::styled(message.message.clone(), style)])
            })
            .collect();

        Paragraph::new(log_messages)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Game log"),
            )
            .style(Style::default().fg(Color::Yellow))
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        self.terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)
            .unwrap();
    }
}
