use ratatui::style::Color as RatatuiColor;
use sdl2::pixels::Color as SDLColor;

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
    Player,
    Door,
    Empty,
}

impl Tile {
    pub fn symbol(&self) -> char {
        match self {
            Tile::Wall => '#',
            Tile::Floor => '.',
            Tile::Player => '@',
            Tile::Door => '+',
            Tile::Empty => ' ',
        }
    }

    pub fn color(&self) -> SDLColor {
        match self {
            Tile::Wall => SDLColor::RGB(255, 255, 255),
            Tile::Floor => SDLColor::RGB(50, 50, 50),
            Tile::Player => SDLColor::RGB(255, 255, 0),
            Tile::Door => SDLColor::RGB(150, 75, 0),
            Tile::Empty => SDLColor::RGB(0, 0, 0),
        }
    }

    pub fn term_color(&self) -> RatatuiColor {
        match self {
            Tile::Wall => RatatuiColor::White,
            Tile::Floor => RatatuiColor::DarkGray,
            Tile::Player => RatatuiColor::Cyan,
            Tile::Door => RatatuiColor::LightYellow,
            Tile::Empty => RatatuiColor::Black,
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
            '+' => Tile::Door,
            _ => Tile::Empty,
        }
    }
}
