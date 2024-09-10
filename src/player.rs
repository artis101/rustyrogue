pub struct Player {
    pub level: u32,
    pub exp: u32,
    // stats that can be changed by leveling up or through curses
    pub max_hp: u32,
    pub current_hp: u32,
    pub strength: u32,
    pub defense: u32,
    pub fov_radius: u32,
}

impl Player {
    pub fn new() -> Self {
        Player {
            level: 1,
            exp: 0,
            max_hp: 20,
            current_hp: 2,
            strength: 5,
            defense: 2,
            fov_radius: 4,
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
        self.exp = 0;
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.current_hp = self.current_hp.saturating_sub(amount);
    }

    pub fn is_dead(&self) -> bool {
        self.current_hp == 0
    }
}
