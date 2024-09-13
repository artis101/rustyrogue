use crate::tile::Tile;

pub type GameTileRow = Vec<Tile>;
pub type GameMapTiles = Vec<GameTileRow>;

pub type Coordinate = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: Coordinate,
    pub y: Coordinate,
}

impl Point {
    pub fn new(x: Coordinate, y: Coordinate) -> Self {
        Point { x, y }
    }
}
