use ratatui::style::Color as RatatuiColor;
use sdl2::pixels::Color as SDLColor;

#[derive(Clone, Copy, PartialEq)]
pub enum DoorState {
    Open,
    Closed,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    Archway { locked: bool },
    Stairs { up: bool },
    Wall,
    Floor,
    Player,
    Door { state: DoorState },
    Empty,
}

impl Tile {
    pub fn symbol(&self) -> char {
        match self {
            Tile::Archway { .. } => '∩',
            Tile::Stairs { up: false } => '>',
            Tile::Stairs { up: true } => '<',
            Tile::Wall => '#',
            Tile::Floor => '.',
            Tile::Player => '@',
            Tile::Door {
                state: DoorState::Open,
            } => '/',
            Tile::Door {
                state: DoorState::Closed,
            } => '+',
            Tile::Empty => ' ',
        }
    }

    pub fn term_fg(&self) -> RatatuiColor {
        match self {
            Tile::Archway { .. } => RatatuiColor::LightCyan,
            Tile::Stairs { up: false } => RatatuiColor::DarkGray,
            Tile::Stairs { up: true } => RatatuiColor::Gray,
            Tile::Wall => RatatuiColor::Gray,
            Tile::Floor => RatatuiColor::DarkGray,
            Tile::Player => RatatuiColor::Cyan,
            Tile::Door {
                state: DoorState::Open,
            } => RatatuiColor::LightYellow,
            Tile::Door {
                state: DoorState::Closed,
            } => RatatuiColor::LightYellow,
            Tile::Empty => RatatuiColor::Black,
        }
    }

    pub fn term_bg(&self) -> RatatuiColor {
        match self {
            Tile::Archway { .. } => RatatuiColor::Black,
            Tile::Stairs { up: false } => RatatuiColor::Black,
            Tile::Stairs { up: true } => RatatuiColor::Black,
            Tile::Wall => RatatuiColor::White,
            Tile::Floor => RatatuiColor::Black,
            Tile::Player => RatatuiColor::Black,
            Tile::Door {
                state: DoorState::Open,
            } => RatatuiColor::Black,
            Tile::Door {
                state: DoorState::Closed,
            } => RatatuiColor::Black,
            Tile::Empty => RatatuiColor::Black,
        }
    }

    pub fn color(&self) -> SDLColor {
        match self {
            Tile::Archway { .. } => SDLColor::RGB(0, 255, 255),
            Tile::Stairs { up: false } => SDLColor::RGB(100, 100, 100),
            Tile::Stairs { up: true } => SDLColor::RGB(150, 150, 150),
            Tile::Wall => SDLColor::RGB(255, 255, 255),
            Tile::Floor => SDLColor::RGB(50, 50, 50),
            Tile::Player => SDLColor::RGB(255, 255, 0),
            Tile::Door {
                state: DoorState::Open,
            } => SDLColor::RGB(150, 75, 0),
            Tile::Door {
                state: DoorState::Closed,
            } => SDLColor::RGB(150, 75, 0),
            Tile::Empty => SDLColor::RGB(0, 0, 0),
        }
    }

    pub fn is_walkable(&self) -> bool {
        match self {
            Tile::Wall => false,
            Tile::Archway { locked } => !locked,
            _ => true,
        }
    }

    pub fn from_char(c: char) -> Self {
        match c {
            '∩' => Tile::Archway { locked: true },
            '>' => Tile::Stairs { up: false },
            '<' => Tile::Stairs { up: true },
            '#' => Tile::Wall,
            '.' => Tile::Floor,
            '@' => Tile::Player,
            '+' => Tile::Door {
                state: DoorState::Open,
            },
            '/' => Tile::Door {
                state: DoorState::Closed,
            },
            _ => Tile::Empty,
        }
    }
}
