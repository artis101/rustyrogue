use ratatui::style::Color as RatatuiColor;
use sdl2::pixels::Color as SDLColor;

#[derive(Clone, Copy, PartialEq)]
pub enum DoorState {
    Open,
    Closed,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
    Player,
    Door { state: DoorState },
    Empty,
}

impl Tile {
    pub fn symbol(&self) -> char {
        match self {
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
            // Tile::Wall | Tile::Door => false,
            _ => true,
        }
    }

    pub fn from_char(c: char) -> Self {
        match c {
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
