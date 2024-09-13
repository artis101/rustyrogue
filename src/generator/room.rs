use crate::map::types::{Coordinate, GameMapTiles, Point};
use crate::tile::Tile;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Room {
    pub location: Point,
    pub width: Coordinate,
    pub height: Coordinate,
    tiles: Rc<RefCell<GameMapTiles>>,
}

impl Room {
    pub fn new(
        location: Point,
        width: Coordinate,
        height: Coordinate,
        tiles: Rc<RefCell<GameMapTiles>>,
    ) -> Self {
        Room {
            location,
            width,
            height,
            tiles,
        }
    }

    pub fn populate(&self) {
        self.fill_with_floor();
        self.surround_with_walls();
    }

    pub fn fill_with_floor(&self) {
        let mut tiles = self.tiles.borrow_mut();
        let max_y = tiles.len();
        let max_x = tiles.first().map_or(0, |row| row.len());

        for y in self.location.y..self.location.y.saturating_add(self.height) {
            for x in self.location.x..self.location.x.saturating_add(self.width) {
                if y < max_y && x < max_x {
                    tiles[y][x] = Tile::Floor {
                        visible: false,
                        cursed: false,
                    };
                }
            }
        }
    }

    pub fn surround_with_walls(&self) {
        let mut tiles = self.tiles.borrow_mut();
        let max_y = tiles.len();
        let max_x = tiles.first().map_or(0, |row| row.len());

        for y in self.location.y.saturating_sub(1)..=self.location.y.saturating_add(self.height) {
            for x in self.location.x.saturating_sub(1)..=self.location.x.saturating_add(self.width)
            {
                if y < max_y
                    && x < max_x
                    && (y == self.location.y.saturating_sub(1)
                        || y == self.location.y.saturating_add(self.height)
                        || x == self.location.x.saturating_sub(1)
                        || x == self.location.x.saturating_add(self.width))
                {
                    tiles[y][x] = Tile::Wall { visible: false };
                }
            }
        }
    }

    pub fn intersects(
        &self,
        other_location: Point,
        other_width: Coordinate,
        other_height: Coordinate,
    ) -> bool {
        let buffer = 1; // Add a buffer zone around rooms

        let self_left = self.location.x.saturating_sub(buffer);
        let self_right = self.location.x + self.width + buffer;
        let self_top = self.location.y.saturating_sub(buffer);
        let self_bottom = self.location.y + self.height + buffer;

        let other_left = other_location.x.saturating_sub(buffer);
        let other_right = other_location.x + other_width + buffer;
        let other_top = other_location.y.saturating_sub(buffer);
        let other_bottom = other_location.y + other_height + buffer;

        self_left < other_right
            && self_right > other_left
            && self_top < other_bottom
            && self_bottom > other_top
    }
}
