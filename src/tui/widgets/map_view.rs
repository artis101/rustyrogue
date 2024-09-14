use crate::map::types::{GameMapTiles, Point};
use crate::tile::Tile;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::sync::{Arc, RwLock};

pub struct MapView {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    camera_position: Point,
}

impl MapView {
    pub fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(MapView {
            terminal,
            camera_position: Point::new(0, 0),
        })
    }

    pub fn run(&mut self, dungeon: Arc<RwLock<GameMapTiles>>) -> Result<(), io::Error> {
        let (viewport_width, viewport_height) = {
            let terminal_size = self.terminal.size()?;
            (
                terminal_size.width as usize - 2,
                terminal_size.height as usize - 2,
            )
        };
        loop {
            self.draw(dungeon.clone())?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let dungeon = dungeon.read().unwrap();
                        let dungeon_height = dungeon.len();
                        let dungeon_width = dungeon.first().map_or(0, |row| row.len());
                        drop(dungeon);

                        match key.code {
                            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('a') => {
                                self.camera_position.x = self.camera_position.x.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                                self.camera_position.y = (self.camera_position.y + 1)
                                    .min(dungeon_height.saturating_sub(viewport_height));
                            }
                            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                                self.camera_position.y = self.camera_position.y.saturating_sub(1);
                            }
                            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('d') => {
                                self.camera_position.x = (self.camera_position.x + 1)
                                    .min(dungeon_width.saturating_sub(viewport_width));
                            }
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        }

                        // Ensure camera position doesn't exceed dungeon boundaries
                        self.camera_position.x = self
                            .camera_position
                            .x
                            .min(dungeon_width.saturating_sub(viewport_width));
                        self.camera_position.y = self
                            .camera_position
                            .y
                            .min(dungeon_height.saturating_sub(viewport_height));
                    }
                }
            }
        }
    }
    pub fn draw(&mut self, dungeon: Arc<RwLock<GameMapTiles>>) -> Result<(), io::Error> {
        let dungeon = dungeon.read().unwrap();
        let terminal_size = self.terminal.size()?;
        let (width, height) = (terminal_size.width, terminal_size.height);
        let viewport_width = width as usize - 2; // Subtract 2 for borders
        let viewport_height = height as usize - 2; // Subtract 2 for borders

        let dungeon_height = dungeon.len();
        let dungeon_width = dungeon.first().map_or(0, |row| row.len());

        let map_string: Vec<Line> = (0..viewport_height)
            .map(|y| {
                let row_index = (self.camera_position.y + y).min(dungeon_height.saturating_sub(1));
                Line::from(
                    (0..viewport_width)
                        .map(|x| {
                            let col_index =
                                (self.camera_position.x + x).min(dungeon_width.saturating_sub(1));
                            let tile = dungeon
                                .get(row_index)
                                .and_then(|row| row.get(col_index))
                                .unwrap_or(&Tile::Empty);
                            let style = Style::default().fg(tile.term_fg()).bg(tile.term_bg());
                            Span::styled(tile.as_char().to_string(), style)
                        })
                        .collect::<Vec<Span>>(),
                )
            })
            .collect();

        let title = format!(
            "Dungeon {}x{} | View {}x{}",
            dungeon_width, dungeon_height, viewport_width, viewport_height
        );
        let paragraph = Paragraph::new(map_string).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title(title),
        );
        self.terminal.draw(|f| {
            let size = f.area();
            f.render_widget(paragraph, size);
        })?;
        Ok(())
    }
}

impl Drop for MapView {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        self.terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)
            .unwrap();
    }
}
