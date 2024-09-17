use crate::map::types::{Coordinate, GameMapTiles, Point};
use crate::tile::Tile;
use rand::seq::SliceRandom;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
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
        self.place_doors();
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.location.x + self.width / 2,
            self.location.y + self.height / 2,
        )
    }

    fn fill_with_floor(&self) {
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

    fn surround_with_walls(&self) {
        let tiles = self.tiles.read().unwrap();
        let max_y = tiles.len();
        let max_x = tiles.first().map_or(0, |row| row.len());
        drop(tiles); // Release the read lock

        let mut tiles = self.tiles.write().unwrap();

        for y in self.location.y..=self.location.y.saturating_add(self.height) {
            for x in self.location.x..=self.location.x.saturating_add(self.width) {
                if y < max_y
                    && x < max_x
                    && (y == self.location.y
                        || y == self.location.y.saturating_add(self.height)
                        || x == self.location.x
                        || x == self.location.x.saturating_add(self.width))
                {
                    tiles[y][x] = Tile::Wall { visible: false };
                }
            }
        }
    }

    fn get_left_wall(&self) -> Vec<Point> {
        (self.location.y..self.location.y.saturating_add(self.height))
            .map(|y| Point::new(self.location.x, y))
            .collect()
    }

    fn get_bottom_wall(&self) -> Vec<Point> {
        (self.location.x..self.location.x.saturating_add(self.width))
            .map(|x| Point::new(x, self.location.y.saturating_add(self.height)))
            .collect()
    }

    fn get_top_wall(&self) -> Vec<Point> {
        (self.location.x..self.location.x.saturating_add(self.width))
            .map(|x| Point::new(x, self.location.y))
            .collect()
    }

    fn get_right_wall(&self) -> Vec<Point> {
        (self.location.y..self.location.y.saturating_add(self.height))
            .map(|y| Point::new(self.location.x.saturating_add(self.width), y))
            .collect()
    }

    fn get_walls(&self) -> [Vec<Point>; 4] {
        [
            self.get_left_wall(),
            self.get_bottom_wall(),
            self.get_top_wall(),
            self.get_right_wall(),
        ]
    }

    fn place_doors(&self) {
        let walls: [Vec<Point>; 4] = self.get_walls();

        while self.num_doors() < 2 {
            let random_wall = walls.choose(&mut rand::thread_rng()).unwrap();
            let random_point = random_wall.choose(&mut rand::thread_rng()).unwrap();
            let mut tiles = self.tiles.write().unwrap();
            tiles[random_point.y][random_point.x] = Tile::Door {
                visible: false,
                open: false,
            };
        }
    }

    pub fn num_doors(&self) -> u8 {
        self.get_walls()
            .iter()
            .map(|wall| {
                wall.iter()
                    .filter(|point| {
                        let tiles = self.tiles.read().unwrap();
                        let tile = tiles[point.y][point.x];
                        drop(tiles); // Release the read lock
                        matches![tile, Tile::Door { .. }]
                    })
                    .count() as u8
            })
            .sum()
    }

    pub fn get_doors(&self) -> Vec<Point> {
        self.get_walls()
            .iter()
            .flat_map(|wall| {
                wall.iter()
                    .filter(|point| {
                        let tiles = self.tiles.read().unwrap();
                        let tile = tiles[point.y][point.x];
                        drop(tiles); // Release the read lock
                        matches![tile, Tile::Door { .. }]
                    })
                    .copied()
                    .collect::<Vec<Point>>()
            })
            .collect()
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
