use crate::Game;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    widgets::{Block, Gauge, Paragraph, Widget},
};

pub struct InventoryWidget<'a> {
    game: &'a Game,
}

impl<'a> InventoryWidget<'a> {
    pub fn new(game: &'a Game) -> Self {
        Self { game }
    }
}

impl Widget for InventoryWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let player = self.game.get_player();

        let create_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Level
                Constraint::Length(2), // Player Info
                Constraint::Length(1), // HP
                Constraint::Length(1), // HP Gauge
                Constraint::Length(1), // empty space
                Constraint::Length(1), // XP
                Constraint::Length(1), // XP Gauge
            ]);

        let chunks = create_layout.split(area);

        let level = Paragraph::new(format!("Level: {}", player.level))
            .alignment(ratatui::layout::Alignment::Center);
        level.render(chunks[0], buf);

        let player_info = Paragraph::new(format!(
            "STR: {} | DEF: {}",
            player.strength, player.defense
        ))
        .alignment(ratatui::layout::Alignment::Center);
        player_info.render(chunks[1], buf);

        let hp = Paragraph::new("HP").alignment(ratatui::layout::Alignment::Center);
        hp.render(chunks[2], buf);

        // HP Gauge
        let hp_ratio = player.current_hp as f64 / player.max_hp as f64;
        let hp_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Red))
            .ratio(hp_ratio)
            .label(format!("{} / {}", player.current_hp, player.max_hp));

        hp_gauge.render(chunks[3], buf);

        let xp = Paragraph::new("XP").alignment(ratatui::layout::Alignment::Center);
        xp.render(chunks[5], buf);

        // XP Gauge
        let xp_ratio = player.exp as f64 / player.xp_for_next_level() as f64;
        let xp_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(xp_ratio)
            .label(format!("{} / {}", player.exp, player.xp_for_next_level()));

        xp_gauge.render(chunks[6], buf);
    }
}
