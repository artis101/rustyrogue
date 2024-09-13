use crate::map::types::{Coordinate, GameMapTiles, Point};
use crate::tile::Tile;
use std::sync::{Arc, RwLock};

pub struct Room {
    pub location: Point,
    pub width: Coordinate,
    pub height: Coordinate,
    tiles: Arc<RwLock<GameMapTiles>>,
}

impl Room {
    pub fn new(
        location: Point,
        width: Coordinate,
        height: Coordinate,
        tiles: Arc<RwLock<GameMapTiles>>,
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
        let tiles = self.tiles.read().unwrap();
        let max_y = tiles.len();
        let max_x = tiles.first().map_or(0, |row| row.len());
        drop(tiles); // Release the read lock

        let mut tiles = self.tiles.write().unwrap();
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
        let tiles = self.tiles.read().unwrap();
        let max_y = tiles.len();
        let max_x = tiles.first().map_or(0, |row| row.len());
        drop(tiles); // Release the read lock

        let mut tiles = self.tiles.write().unwrap();
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
        let buffer = 4; // Add a buffer zone around rooms

        let self_left = self.location.x;
        let self_right = self.location.x + self.width;
        let self_top = self.location.y;
        let self_bottom = self.location.y + self.height;

        let other_left = other_location.x;
        let other_right = other_location.x + other_width;
        let other_top = other_location.y;
        let other_bottom = other_location.y + other_height;

        // Check if the rectangles overlap, including the buffer
        (self_left.saturating_sub(buffer) <= other_right)
            && (self_right + buffer >= other_left)
            && (self_top.saturating_sub(buffer) <= other_bottom)
            && (self_bottom + buffer >= other_top)
    }
}
