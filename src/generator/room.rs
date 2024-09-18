use crate::map::types::{Coordinate, GameMapTiles, Point};
use crate::tile::Tile;
use rand::seq::SliceRandom;
use rand::Rng;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub enum RoomType {
    Normal,
    Obelisk,
    Secret,
}

#[derive(Clone)]
pub struct Room {
    pub location: Point,
    pub width: Coordinate,
    pub height: Coordinate,
    pub room_type: RoomType,
}

// Possible mob types
#[derive(Clone)]
enum MobType {
    Wither,
    Bat,
    Brute,
}

impl Room {
    const OBELISK_PROB_IN_NORMAL_ROOM: f64 = 0.25; // 25% chance or
    const SECRET_PROB_IN_NORMAL_ROOM: f64 = 0.5; // 50% chance

    pub fn new(location: Point, width: Coordinate, height: Coordinate) -> Self {
        Room {
            location,
            width,
            height,
            room_type: RoomType::Normal,
        }
    }

    pub fn populate(&mut self, tiles: &Arc<RwLock<GameMapTiles>>) {
        self.fill_with_floor(tiles);
        self.surround_with_walls(tiles);
        self.place_columns(tiles);
        self.determine_room_type(tiles);

        let mut rng = rand::thread_rng();

        match self.room_type {
            RoomType::Obelisk => {
                self.place_obelisk(tiles);
            }
            RoomType::Secret => {
                self.place_secret(tiles, true); // `true` indicates it's a secret room

                // Brutes have a 50% chance of appearing in secret rooms
                if rng.gen_bool(0.5) {
                    self.place_mobs(tiles);
                    self.place_brute(tiles);
                }
            }
            RoomType::Normal => {
                // Decide whether to place an obelisk
                if rng.gen_bool(Self::OBELISK_PROB_IN_NORMAL_ROOM) {
                    self.place_obelisk(tiles);
                }
                // Decide whether to place a secret
                if rng.gen_bool(Self::SECRET_PROB_IN_NORMAL_ROOM) {
                    self.place_secret(tiles, false); // `false` indicates it's a normal room
                }
                // Mobs have a 95% chance of appearing in normal rooms, it's a dungeon after all
                if rng.gen_bool(0.95) {
                    self.place_mobs(tiles);
                }
            }
        }

        self.place_doors(tiles);
    }

    fn determine_room_type(&mut self, tiles: &Arc<RwLock<GameMapTiles>>) {
        let entrances = self.find_room_entrances(tiles);
        let num_entrances = entrances.len();
        let mut rng = rand::thread_rng();

        // Define probabilities based on the number of entrances
        let (obelisk_chance, secret_chance, normal_chance) = if num_entrances == 1 {
            // Probabilities for dead-end rooms
            (0.7, 0.2, 0.1)
        } else {
            // Probabilities for other rooms
            (0.3, 0.4, 0.3)
        };

        // Ensure the probabilities sum to 1.0
        let total_chance: f64 = obelisk_chance + secret_chance + normal_chance;
        assert!(
            (total_chance - 1.0).abs() < f64::EPSILON,
            "Probabilities must sum to 1.0"
        );

        let random_value: f64 = rng.gen();

        if random_value < obelisk_chance {
            self.room_type = RoomType::Obelisk;
        } else if random_value < obelisk_chance + secret_chance {
            self.room_type = RoomType::Secret;
        } else {
            self.room_type = RoomType::Normal;
        }
    }

    fn place_obelisk(&self, tiles: &Arc<RwLock<GameMapTiles>>) {
        let center = self.center();
        let mut tiles_write = tiles.write().unwrap();
        tiles_write[center.y][center.x] = Tile::Obelisk {
            visible: false,
            curse: true,
            fov: 8,
            damage_hp: 1,
            reduce_fov_radius: 3,
        };
    }

    fn place_secret(&self, tiles: &Arc<RwLock<GameMapTiles>>, is_secret_room: bool) {
        let mut rng = rand::thread_rng();

        // Find positions adjacent to columns where secrets can be placed
        let mut potential_positions = Vec::new();

        let x_start = self.location.x + 1;
        let x_end = self.location.x + self.width - 1;
        let y_start = self.location.y + 1;
        let y_end = self.location.y + self.height - 1;

        let tiles_read = tiles.read().unwrap();

        // Scan the room for columns and collect adjacent floor positions
        for y in y_start..y_end {
            for x in x_start..x_end {
                if let Tile::Column { .. } = tiles_read[y][x] {
                    // Adjacent positions to the column
                    let adjacent_positions = vec![
                        (x.wrapping_sub(1), y),
                        (x + 1, y),
                        (x, y.wrapping_sub(1)),
                        (x, y + 1),
                    ];

                    for &(ax, ay) in &adjacent_positions {
                        if ax >= x_start && ax < x_end && ay >= y_start && ay < y_end {
                            if let Tile::Floor { .. } = tiles_read[ay][ax] {
                                potential_positions.push((ax, ay));
                            }
                        }
                    }
                }
            }
        }

        drop(tiles_read); // Release the read lock

        // Determine the rarity of the secret
        let rarity = if is_secret_room {
            if rng.gen_bool(0.9) {
                100
            } else {
                1000
            }
        } else {
            let rarity_values = [1, 10, 100];
            *rarity_values.choose(&mut rng).unwrap()
        };

        let mut tiles_write = tiles.write().unwrap();

        if !potential_positions.is_empty() {
            // Place the secret behind a column
            let &(x, y) = potential_positions.choose(&mut rng).unwrap();
            tiles_write[y][x] = Tile::Secret {
                visible: false,
                rarity,
            };
        } else {
            // If no columns exist, place the secret at a random floor position
            let x = rng.gen_range(x_start..x_end);
            let y = rng.gen_range(y_start..y_end);
            tiles_write[y][x] = Tile::Secret {
                visible: false,
                rarity,
            };
        }
    }

    fn place_mobs(&self, tiles: &Arc<RwLock<GameMapTiles>>) {
        let mut rng = rand::thread_rng();

        // Decide the number of mobs to place, e.g., 1 to 3 mobs per room
        let num_mobs = rng.gen_range(1..=3);

        let mob_types = [MobType::Wither, MobType::Bat, MobType::Brute];

        // Get the dimensions of the room
        let x_start = self.location.x + 1;
        let x_end = self.location.x + self.width - 1;
        let y_start = self.location.y + 1;
        let y_end = self.location.y + self.height - 1;

        // Positions where mobs have been placed to avoid overlap
        let mut occupied_positions = std::collections::HashSet::new();

        // Lock the tiles for writing
        let mut tiles_write = tiles.write().unwrap();

        for _ in 0..num_mobs {
            // Randomly select a mob type
            let mob_type = mob_types[rng.gen_range(0..mob_types.len())].clone();

            // Find a random position within the room that is not occupied
            let mut attempts = 0;
            let position = loop {
                if attempts > 10 {
                    // Avoid infinite loops in case the room is too full
                    break None;
                }
                let x = rng.gen_range(x_start..x_end);
                let y = rng.gen_range(y_start..y_end);

                if occupied_positions.contains(&(x, y)) {
                    attempts += 1;
                    continue;
                }

                // Check if the tile is a floor and not occupied by other features
                match tiles_write[y][x] {
                    Tile::Floor { .. } => {
                        occupied_positions.insert((x, y));
                        break Some((x, y));
                    }
                    _ => {
                        attempts += 1;
                        continue;
                    }
                }
            };

            if let Some((x, y)) = position {
                // Create the mob tile based on the selected mob type
                let mob_tile = match mob_type {
                    MobType::Wither => Tile::Wither {
                        visible: false,
                        hp: 50,     // Example HP
                        damage: 10, // Example damage
                        fov: 6,     // Example field of view
                    },
                    MobType::Bat => Tile::Bat {
                        visible: false,
                        hp: 20,
                        damage: 5,
                        fov: 8,
                    },
                    MobType::Brute => Tile::Brute {
                        visible: false,
                        hp: 80,
                        damage: 15,
                        fov: 4,
                    },
                };

                // Place the mob on the map
                tiles_write[y][x] = mob_tile;
            }
        }
    }

    fn place_brute(&self, tiles: &Arc<RwLock<GameMapTiles>>) {
        let mut rng = rand::thread_rng();
        // Get the dimensions of the room
        let x_start = self.location.x + 1;
        let x_end = self.location.x + self.width - 1;
        let y_start = self.location.y + 1;
        let y_end = self.location.y + self.height - 1;
        // Positions where mobs have been placed to avoid overlap
        let mut occupied_positions = std::collections::HashSet::new();
        // Lock the tiles for writing
        let mut tiles_write = tiles.write().unwrap();
        // Find a random position within the room that is not occupied
        let mut attempts = 0;
        let position = loop {
            if attempts > 10 {
                // Avoid infinite loops in case the room is too full
                break None;
            }
            let x = rng.gen_range(x_start..x_end);
            let y = rng.gen_range(y_start..y_end);
            if occupied_positions.contains(&(x, y)) {
                attempts += 1;
                continue;
            }
            // Check if the tile is a floor and not occupied by other features
            match tiles_write[y][x] {
                Tile::Floor { .. } => {
                    occupied_positions.insert((x, y));
                    break Some((x, y));
                }
                _ => {
                    attempts += 1;
                    continue;
                }
            }
        };
        if let Some((x, y)) = position {
            // Create the mob tile based on the selected mob type
            let mob_tile = Tile::Brute {
                visible: false,
                hp: 20,
                damage: 10,
                fov: 4,
            };
            // Place the mob on the map
            tiles_write[y][x] = mob_tile;
        }
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

    fn place_columns(&self, tiles_arc: &Arc<RwLock<GameMapTiles>>) {
        let mut rng = rand::thread_rng();

        // 50% chance to place columns in the corners of the room
        if rng.gen_bool(0.5) {
            let mut tiles_write = tiles_arc.write().unwrap();

            let x1 = self.location.x + 2;
            let x2 = self.location.x + self.width - 2;
            let y1 = self.location.y + 2;
            let y2 = self.location.y + self.height - 2;

            // Place columns in corners
            let positions = vec![(x1, y1), (x2, y1), (x1, y2), (x2, y2)];

            for &(x, y) in &positions {
                tiles_write[y][x] = Tile::Column { visible: false };
            }
        }
    }

    pub fn reset(&self, tiles_arc: &Arc<RwLock<GameMapTiles>>) {
        // Flood fill the room inside the walls with floor tiles
        let tiles_read = tiles_arc.read().unwrap();
        let max_y = tiles_read.len();
        let max_x = tiles_read.first().map_or(0, |row| row.len());
        drop(tiles_read); // Release the read lock
                          // Lock the tiles for writing
        let mut tiles_write = tiles_arc.write().unwrap();
        for y in self.location.y + 1..self.location.y + self.height {
            for x in self.location.x + 1..self.location.x + self.width {
                if y < max_y && x < max_x {
                    tiles_write[y][x] = Tile::Floor {
                        visible: false,
                        cursed: false,
                    };
                }
            }
        }
    }
}
