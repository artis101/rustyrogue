pub mod widgets;

use crate::game::{Game, MessageType};
use crate::tile::Tile;
use crate::tui::widgets::InventoryWidget;
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

const QUIT_KEY: KeyCode = KeyCode::Char('q');
const LEFT_MOVEMENT_KEYS: [KeyCode; 3] = [KeyCode::Left, KeyCode::Char('h'), KeyCode::Char('a')];
const DOWN_MOVEMENT_KEYS: [KeyCode; 3] = [KeyCode::Down, KeyCode::Char('j'), KeyCode::Char('s')];
const UP_MOVEMENT_KEYS: [KeyCode; 3] = [KeyCode::Up, KeyCode::Char('k'), KeyCode::Char('w')];
const RIGHT_MOVEMENT_KEYS: [KeyCode; 3] = [KeyCode::Right, KeyCode::Char('l'), KeyCode::Char('d')];
const INTERACT_KEYS: [KeyCode; 2] = [KeyCode::Char(' '), KeyCode::Char('e')];
const HINT_KEYS: [KeyCode; 2] = [KeyCode::Char('?'), KeyCode::Tab];

// Declare constant for the game log height
const GAME_LOG_HEIGHT: u16 = 7;
const INVENTORY_WIDTH: u16 = 25;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    map_area_size: (usize, usize),
}

impl Tui {
    pub fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Tui {
            terminal,
            map_area_size: (9999, 9999), // default to something big to avoid flashing on first draw
        })
    }

    pub fn run(&mut self, game: &mut Game) -> Result<(), io::Error> {
        loop {
            self.draw(game)?;

            if game.is_game_over() {
                // Display the final game state and wait for the player to quit
                self.draw(game)?;
                loop {
                    if event::poll(std::time::Duration::from_millis(100))? {
                        if let event::Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press && key.code == QUIT_KEY {
                                return Ok(());
                            }
                        }
                    }
                }
            } else if event::poll(std::time::Duration::from_millis(100))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            code if code == QUIT_KEY => return Ok(()),
                            code if LEFT_MOVEMENT_KEYS.contains(&code) => game.move_player(-1, 0),
                            code if RIGHT_MOVEMENT_KEYS.contains(&code) => game.move_player(1, 0),
                            code if UP_MOVEMENT_KEYS.contains(&code) => game.move_player(0, -1),
                            code if DOWN_MOVEMENT_KEYS.contains(&code) => game.move_player(0, 1),
                            code if INTERACT_KEYS.contains(&code) => game.interact(),
                            code if HINT_KEYS.contains(&code) => game.show_hint(),
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    fn draw(&mut self, game: &Game) -> Result<(), io::Error> {
        let map_widget = Self::prepare_map_widget(game, self.map_area_size);
        let info_widget = Self::prepare_inventory_widget(game);
        let game_log_widget = Self::prepare_game_log_widget(game);

        self.terminal.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(0),                  // For Map and Info
                        Constraint::Length(GAME_LOG_HEIGHT), // For Log
                    ]
                    .as_ref(),
                )
                .split(f.area());

            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Min(0),               // Map is fluid and fills the container
                        Constraint::Max(INVENTORY_WIDTH), // Info takes 30% of the width
                    ]
                    .as_ref(),
                )
                .split(main_chunks[0]);

            let map_area = top_chunks[0];
            self.map_area_size = (map_area.columns().count(), map_area.rows().count());

            f.render_widget(map_widget, top_chunks[0]);
            f.render_widget(info_widget, top_chunks[1]);
            f.render_widget(game_log_widget, main_chunks[1]);

            if game.is_game_over() {
                let game_over_message = Paragraph::new("Game Over! Press 'q' to quit.")
                    .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(game_over_message, top_chunks[0]);
            }
        })?;

        Ok(())
    }

    fn prepare_map_widget(game: &Game, map_size: (usize, usize)) -> Paragraph<'static> {
        let map: &Vec<Vec<Tile>> = game.get_map();
        let player_pos = game.get_player_position();

        let visible_width = map_size.0;
        let visible_height = map_size.1;

        let start_x = player_pos.x.saturating_sub(visible_width / 2);
        let start_y = player_pos.y.saturating_sub(visible_height / 2);

        let end_x = (start_x + visible_width).min(map[0].len());
        let end_y = (start_y + visible_height).min(map.len());

        let map_string: Vec<Line> = map[start_y..end_y]
            .iter()
            .map(|row| {
                Line::from(
                    row[start_x..end_x]
                        .iter()
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
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default())
    }

    fn prepare_inventory_widget(game: &Game) -> InventoryWidget {
        InventoryWidget::new(game)
    }

    fn prepare_game_log_widget(game: &Game) -> Paragraph<'static> {
        let log_messages: Vec<Line> = game
            .get_game_log_messages()
            .iter()
            .map(|message| {
                let color = match message.message_type {
                    MessageType::Info => Color::Gray,
                    MessageType::Damage => Color::Red,
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
