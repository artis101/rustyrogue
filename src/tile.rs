use ratatui::style::Color as RatatuiColor;
use sdl2::pixels::Color as SDLColor;

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    // archway is the plot device that starts the game
    Archway { locked: bool },
    // normal map tiles
    Stairs { visible: bool, up: bool },
    Wall { visible: bool },
    Floor { visible: bool },
    // (you)
    Player,
    // interactable tiles
    Door { visible: bool, open: bool },
    Secret { visible: bool }, // secrets reveal themselves when you interact
    SecretFloor { visible: bool }, // secret tiles reveal themeselves when you step on them
    // deadly tiles
    Pit { visible: bool }, // falling into a pit kills the player
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
            Tile::Floor { .. } | Tile::SecretFloor { visible: false } => '·',
            Tile::SecretFloor { visible: true } => '_',
            Tile::Player => '@',
            Tile::Door { open: true, .. } => '+',
            Tile::Door { open: false, .. } => '/',
            Tile::Secret { .. } => '?',
            Tile::Pit { .. } => 'V',
            Tile::Empty => ' ',
        }
    }

    pub fn term_fg(&self) -> RatatuiColor {
        match self {
            Tile::Archway { .. } => RatatuiColor::LightCyan,
            // dark gray/gray when visible tiles
            Tile::Stairs { visible, .. }
            | Tile::Wall { visible }
            | Tile::Floor { visible }
            | Tile::Pit { visible }
            | Tile::SecretFloor { visible } => {
                if *visible {
                    RatatuiColor::White
                } else {
                    RatatuiColor::DarkGray
                }
            }
            Tile::Player => RatatuiColor::Cyan,
            Tile::Door { open: true, .. } => RatatuiColor::LightYellow,
            Tile::Door { open: false, .. } => RatatuiColor::LightYellow,
            Tile::Secret { visible: false } => RatatuiColor::Yellow,
            Tile::Secret { visible: true } => RatatuiColor::LightYellow,
            Tile::Empty => RatatuiColor::Reset,
        }
    }

    pub fn term_bg(&self) -> RatatuiColor {
        match self {
            Tile::Wall { .. } => RatatuiColor::Gray,
            Tile::Floor { .. }
            | Tile::Player
            | Tile::Archway { .. }
            | Tile::Door { .. }
            | Tile::Secret { .. }
            | Tile::SecretFloor { .. }
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
            Tile::Player => SDLColor::RGB(255, 255, 0),
            Tile::Door { open: true, .. } => SDLColor::RGB(150, 75, 0),
            Tile::Door { open: false, .. } => SDLColor::RGB(150, 75, 0),
            Tile::Pit { .. } => SDLColor::RGB(50, 50, 50),
            Tile::Secret { .. } => SDLColor::RGB(255, 255, 0),
            Tile::Empty => SDLColor::RGB(0, 0, 0),
        }
    }

    pub fn is_walkable(&self) -> bool {
        match self {
            Tile::Wall { .. } | Tile::Secret { .. } => false,
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
            '.' => Tile::Floor { visible: false },
            '@' => Tile::Player,
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
            _ => Tile::Empty,
        }
    }
}
