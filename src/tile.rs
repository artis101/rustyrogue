use ratatui::style::Color as RatatuiColor;
use sdl2::pixels::Color as SDLColor;

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    // archway is the plot device that starts the game
    Archway {
        locked: bool,
    },
    // normal map tiles
    Stairs {
        visible: bool,
        up: bool,
    },
    Wall {
        visible: bool,
    },
    Floor {
        visible: bool,
        cursed: bool,
    },
    // (you)
    Player {
        is_dead: bool,
        is_cursed: bool,
    },
    // interactable tiles
    Door {
        visible: bool,
        open: bool,
    },
    Secret {
        visible: bool,
    }, // secrets reveal themselves when you interact
    SecretFloor {
        visible: bool,
    }, // secret tiles reveal themeselves when you step on them
    // hurtful tiles
    Obelisk {
        visible: bool,
        curse: bool,
        fov: u32,
        damage_hp: u32,
        reduce_fov_radius: u32,
    }, // obelisks curse players and should be avoided
    // deadly tiles
    Pit {
        visible: bool,
    }, // falling into a pit kills the player
    // empty
    Empty,
}

impl Tile {
    pub fn symbol(&self) -> char {
        match self {
            Tile::Archway { .. } => '∩',
            Tile::Stairs { up: false, .. } => '>',
            Tile::Stairs { up: true, .. } => '<',
            Tile::Wall { .. } => '#',
            Tile::SecretFloor { visible: true } => '_',
            Tile::Player { .. } => '@',
            Tile::Door { open: true, .. } => '+',
            Tile::Door { open: false, .. } => '/',
            Tile::Secret { visible: true } => '?',
            Tile::Pit { visible: true } => 'V',
            Tile::Obelisk { visible: true, .. } => '|',
            // FOV tiles + enemies
            Tile::Floor { .. }
            | Tile::SecretFloor { visible: false }
            | Tile::Secret { visible: false }
            | Tile::Pit { visible: false }
            | Tile::Obelisk { visible: false, .. } => '·',
            Tile::Empty => ' ',
        }
    }

    pub fn sprite_index(&self) -> usize {
        match self {
            Tile::Archway { .. } => 0,
            Tile::Stairs { up: false, .. } => 1,
            Tile::Stairs { up: true, .. } => 2,
            Tile::Wall { .. } => 3,
            Tile::Floor { .. } => 4,
            Tile::Player { .. } => 5,
            Tile::Door { open: true, .. } => 6,
            Tile::Door { open: false, .. } => 7,
            Tile::Secret { visible: true } => 8,
            Tile::SecretFloor { visible: true } => 9,
            Tile::Obelisk { visible: true, .. } => 10,
            Tile::Pit { visible: true } => 11,
            Tile::Empty => 12,
            _ => 4,
        }
    }

    pub fn term_fg(&self) -> RatatuiColor {
        match self {
            Tile::Archway { .. } => RatatuiColor::LightCyan,
            // dark gray/gray when visible tiles
            Tile::Stairs { visible, .. }
            | Tile::Wall { visible }
            | Tile::Pit { visible }
            | Tile::SecretFloor { visible } => {
                if *visible {
                    RatatuiColor::White
                } else {
                    RatatuiColor::DarkGray
                }
            }
            Tile::Floor { visible, cursed } => {
                if *visible {
                    if *cursed {
                        RatatuiColor::Magenta
                    } else {
                        RatatuiColor::White
                    }
                } else {
                    RatatuiColor::DarkGray
                }
            }
            Tile::Player { is_dead, is_cursed } => {
                if *is_dead {
                    RatatuiColor::Red
                } else if *is_cursed {
                    RatatuiColor::Magenta
                } else {
                    RatatuiColor::Cyan
                }
            }
            Tile::Obelisk { visible: false, .. } => RatatuiColor::DarkGray,
            Tile::Secret { visible: false, .. } => RatatuiColor::DarkGray,
            Tile::Door { visible: true, .. } => RatatuiColor::Yellow,
            Tile::Secret { visible: true } => RatatuiColor::LightYellow,
            Tile::Obelisk { visible: true, .. } => RatatuiColor::Magenta,
            _ => RatatuiColor::Reset,
        }
    }

    pub fn term_bg(&self) -> RatatuiColor {
        match self {
            Tile::Wall { .. } => RatatuiColor::Gray,
            Tile::Floor { .. }
            | Tile::Player { .. }
            | Tile::Archway { .. }
            | Tile::Door { .. }
            | Tile::Secret { .. }
            | Tile::SecretFloor { .. }
            | Tile::Obelisk { .. }
            | Tile::Pit { visible: false } => RatatuiColor::Black,
            Tile::Pit { visible: true } => RatatuiColor::DarkGray,
            _ => RatatuiColor::Reset,
        }
    }

    pub fn color(&self) -> SDLColor {
        match self {
            Tile::Archway { .. } => SDLColor::RGB(0, 255, 255),
            Tile::Stairs { up: false, .. } => SDLColor::RGB(100, 100, 100),
            Tile::Stairs { up: true, .. } => SDLColor::RGB(150, 150, 150),
            Tile::Wall { .. } | Tile::SecretFloor { .. } => SDLColor::RGB(255, 255, 255),
            Tile::Floor { .. } => SDLColor::RGB(50, 50, 50),
            Tile::Player { .. } => SDLColor::RGB(255, 255, 0),
            Tile::Door { open: true, .. } => SDLColor::RGB(150, 75, 0),
            Tile::Door { open: false, .. } => SDLColor::RGB(150, 75, 0),
            Tile::Pit { .. } => SDLColor::RGB(50, 50, 50),
            Tile::Secret { .. } => SDLColor::RGB(255, 255, 0),
            Tile::Obelisk { .. } => SDLColor::RGB(255, 0, 255),
            Tile::Empty => SDLColor::RGB(0, 0, 0),
        }
    }

    pub fn is_walkable(&self) -> bool {
        match self {
            Tile::Wall { .. } | Tile::Secret { .. } | Tile::Obelisk { .. } => false,
            Tile::Archway { locked } => !locked,
            Tile::Door { open, .. } => *open,
            _ => true,
        }
    }

    pub fn from_char(c: char) -> Self {
        match c {
            '∩' => Tile::Archway { locked: true },
            '>' => Tile::Stairs {
                up: false,
                visible: false,
            },
            '<' => Tile::Stairs {
                up: true,
                visible: false,
            },
            '#' => Tile::Wall { visible: false },
            '.' => Tile::Floor {
                visible: false,
                cursed: false,
            },
            '@' => Tile::Player {
                is_dead: false,
                is_cursed: false,
            },
            '+' => Tile::Door {
                open: true,
                visible: false,
            },
            '/' => Tile::Door {
                open: false,
                visible: false,
            },
            'V' => Tile::Pit { visible: false },
            '?' => Tile::Secret { visible: false },
            '_' => Tile::SecretFloor { visible: false },
            '|' => Tile::Obelisk {
                visible: false,
                fov: 6,
                curse: true,
                damage_hp: 1,
                reduce_fov_radius: 2, // essentially halves the FOV
            },
            _ => Tile::Empty,
        }
    }
}
