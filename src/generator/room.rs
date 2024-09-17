use crate::map::types::{Coordinate, GameMapTiles, Point};
use crate::tile::Tile;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Room {
    pub location: Point,
    pub width: Coordinate,
    pub height: Coordinate,
}

impl Room {
    pub fn new(location: Point, width: Coordinate, height: Coordinate) -> Self {
        Room {
            location,
            width,
            height,
        }
    }

    pub fn populate(&self, tiles: &Arc<RwLock<GameMapTiles>>) {
        self.fill_with_floor(tiles);
        self.surround_with_walls(tiles);
        // You can decide whether to place doors here
        // self.place_doors(tiles); // Uncomment if you want to place doors
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.location.x + self.width / 2,
            self.location.y + self.height / 2,
        )
    }

    fn fill_with_floor(&self, tiles_arc: &Arc<RwLock<GameMapTiles>>) {
        let tiles_read = tiles_arc.read().unwrap();
        let max_y = tiles_read.len();
        let max_x = tiles_read.first().map_or(0, |row| row.len());
        drop(tiles_read); // Release the read lock

        let mut tiles_write = tiles_arc.write().unwrap();
        for y in self.location.y..self.location.y.saturating_add(self.height) {
            for x in self.location.x..self.location.x.saturating_add(self.width) {
                if y < max_y && x < max_x {
                    tiles_write[y][x] = Tile::Floor {
                        visible: false,
                        cursed: false,
                    };
                }
            }
        }
    }

    fn surround_with_walls(&self, tiles_arc: &Arc<RwLock<GameMapTiles>>) {
        let tiles_read = tiles_arc.read().unwrap();
        let max_y = tiles_read.len();
        let max_x = tiles_read.first().map_or(0, |row| row.len());
        drop(tiles_read); // Release the read lock

        let mut tiles_write = tiles_arc.write().unwrap();

        for y in self.location.y..=self.location.y.saturating_add(self.height) {
            for x in self.location.x..=self.location.x.saturating_add(self.width) {
                if y < max_y
                    && x < max_x
                    && (y == self.location.y
                        || y == self.location.y.saturating_add(self.height)
                        || x == self.location.x
                        || x == self.location.x.saturating_add(self.width))
                {
                    tiles_write[y][x] = Tile::Wall { visible: false };
                }
            }
        }
    }
}

