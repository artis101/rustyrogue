use crate::map::types::{Coordinate, GameMapTiles, Point};
use crate::tile::Tile;
use rand::Rng;
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
    }

    fn find_room_entrances(&self, tiles: &Arc<RwLock<GameMapTiles>>) -> Vec<Point> {
        let tiles_read = tiles.read().unwrap();
        let max_y = tiles_read.len();
        let max_x = if max_y > 0 { tiles_read[0].len() } else { 0 };
        let mut entrances = Vec::new();

        let left = self.location.x;
        let right = self.location.x + self.width;
        let top = self.location.y;
        let bottom = self.location.y + self.height;

        // Check top and bottom walls
        for x in left..=right {
            // Top wall
            if top < max_y && x < max_x {
                if let Tile::Floor { .. } = tiles_read[top][x] {
                    // Check neighbors to the left and right
                    let left_wall = x == 0 || matches!(tiles_read[top][x - 1], Tile::Wall { .. });
                    let right_wall =
                        x + 1 >= max_x || matches!(tiles_read[top][x + 1], Tile::Wall { .. });
                    if left_wall && right_wall {
                        entrances.push(Point::new(x, top));
                    }
                }
            }
            // Bottom wall
            if bottom < max_y && x < max_x {
                if let Tile::Floor { .. } = tiles_read[bottom][x] {
                    // Check neighbors to the left and right
                    let left_wall =
                        x == 0 || matches!(tiles_read[bottom][x - 1], Tile::Wall { .. });
                    let right_wall =
                        x + 1 >= max_x || matches!(tiles_read[bottom][x + 1], Tile::Wall { .. });
                    if left_wall && right_wall {
                        entrances.push(Point::new(x, bottom));
                    }
                }
            }
        }

        // Check left and right walls
        for y in top..=bottom {
            // Left wall
            if y < max_y && left < max_x {
                if let Tile::Floor { .. } = tiles_read[y][left] {
                    // Check neighbors above and below
                    let top_wall = y == 0 || matches!(tiles_read[y - 1][left], Tile::Wall { .. });
                    let bottom_wall =
                        y + 1 >= max_y || matches!(tiles_read[y + 1][left], Tile::Wall { .. });
                    if top_wall && bottom_wall {
                        entrances.push(Point::new(left, y));
                    }
                }
            }
            // Right wall
            if y < max_y && right < max_x {
                if let Tile::Floor { .. } = tiles_read[y][right] {
                    // Check neighbors above and below
                    let top_wall = y == 0 || matches!(tiles_read[y - 1][right], Tile::Wall { .. });
                    let bottom_wall =
                        y + 1 >= max_y || matches!(tiles_read[y + 1][right], Tile::Wall { .. });
                    if top_wall && bottom_wall {
                        entrances.push(Point::new(right, y));
                    }
                }
            }
        }

        entrances
    }

    pub fn place_doors(&self, tiles: &Arc<RwLock<GameMapTiles>>) {
        let entrances = self.find_room_entrances(tiles);

        let mut tiles_write = tiles.write().unwrap();
        let mut rng = rand::thread_rng();

        for entrance in entrances {
            // Randomly decide to place a door at this entrance (e.g., 50% chance)
            if rng.gen_bool(0.5) {
                tiles_write[entrance.y][entrance.x] = Tile::Door {
                    visible: false,
                    open: false,
                };
            }
        }
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
