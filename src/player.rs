use ratatui::prelude::Stylize;
use ratatui::text::Span;

pub struct Player {
    pub level: u32,
    pub exp: u32,
    pub max_hp: u32,
    pub current_hp: u32,
    pub strength: u32,
    pub defense: u32,
}

impl Player {
    pub fn new() -> Self {
        Player {
            level: 1,
            exp: 0,
            max_hp: 20,
            current_hp: 20,
            strength: 5,
            defense: 2,
        }
    }

    pub fn xp_for_next_level(&self) -> u32 {
        self.level * 100
    }

    pub fn gain_exp(&mut self, amount: u32) {
        self.exp += amount;
        if self.exp >= self.xp_for_next_level() {
            self.level_up();
        }
    }

    pub fn level_up(&mut self) {
        self.level += 1;
        self.max_hp += 5;
        self.current_hp = self.max_hp;
        self.strength += 1;
        self.defense += 1;
    }

    pub fn colored_hp(&self) -> Span<'static> {
        let hp_ratio = self.current_hp as f32 / self.max_hp as f32;

        match hp_ratio {
            r if r >= 0.75 => self.current_hp.to_string().green(),
            r if r >= 0.5 => self.current_hp.to_string().yellow(),
            r if r >= 0.25 => self.current_hp.to_string().red(),
            _ => self.current_hp.to_string().gray(),
        }
    }
}
