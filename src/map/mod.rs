use types::{Coordinate, GameMapTiles, Point};

use crate::tile::Tile;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub mod types;

pub struct Map {
    tiles: GameMapTiles,
    visible_tiles: HashSet<Point>,
}

impl Map {
    pub fn load(filename: &str, hint_filename: &str) -> io::Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut tiles = Vec::new();

        for line in reader.lines() {
            let line = line?;
            tiles.push(line.chars().map(Tile::from_char).collect());
        }

        let hint_file = File::open(hint_filename);
        if let Ok(hint_file) = hint_file {
            // read json file
            let _reader = BufReader::new(hint_file);
            // resolve secrets in map
        }

        Ok(Map {
            tiles,
            visible_tiles: HashSet::new(),
        })
    }

    pub fn width(&self) -> Coordinate {
        self.tiles.first().map(|row| row.len()).unwrap_or(0)
    }

    pub fn height(&self) -> Coordinate {
        self.tiles.len()
    }

    pub fn get_tiles(&self) -> &GameMapTiles {
        &self.tiles
    }

    pub fn get_tile(&self, point: Point) -> Tile {
        self.tiles
            .get(point.y)
            .and_then(|row| row.get(point.x))
            .copied()
            .unwrap_or(Tile::Empty)
    }

    pub fn set_tile(&mut self, point: Point, tile: Tile) {
        if let Some(row) = self.tiles.get_mut(point.y) {
            if let Some(t) = row.get_mut(point.x) {
                *t = tile;
            }
        }
    }

    pub fn is_walkable(&self, point: Point) -> bool {
        self.get_tile(point).is_walkable()
    }

    pub fn is_deadly(&self, point: Point) -> bool {
        matches!(self.get_tile(point), Tile::Pit { .. })
    }

    pub fn find_player(&self) -> Option<Point> {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, &tile) in row.iter().enumerate() {
                if let Tile::Player { .. } = tile {
                    return Some(Point { x, y });
                }
            }
        }
        None
    }

    pub fn is_interactable(&self, point: Point) -> bool {
        matches!(
            self.tiles[point.y][point.x],
            Tile::Door { .. } | Tile::Secret { visible: true } // you can interact with visible
                                                               // things for now
        )
    }

    pub fn interact_tile(&mut self, point: Point) {
        if let Tile::Door { open, visible } = self.tiles[point.y][point.x] {
            self.tiles[point.y][point.x] = Tile::Door {
                open: !open,
                visible,
            };
        }
    }

    fn clear_visible_tiles(&mut self) {
        let points_to_clear: Vec<Point> = self.visible_tiles.drain().collect();

        for point in points_to_clear {
            self.update_tile_visibility(point, false);
        }
    }

    pub fn update_fov(&mut self, pov: Point, fov_radius: u32) {
        self.clear_visible_tiles();
        let radius_squared = (fov_radius * fov_radius) as i32;

        for dy in -(fov_radius as i32)..=fov_radius as i32 {
            for dx in -(fov_radius as i32)..=fov_radius as i32 {
                if dx * dx + dy * dy <= radius_squared {
                    let x = pov.x as i32 + dx;
                    let y = pov.y as i32 + dy;

                    if (0..self.width() as i32).contains(&x)
                        && (0..self.height() as i32).contains(&y)
                        && self.has_line_of_sight(pov.x, pov.y, x as usize, y as usize)
                    {
                        let point = Point::new(x as Coordinate, y as Coordinate);
                        self.visible_tiles.insert(point);
                        self.update_tile_visibility(point, true);
                    }
                }
            }
        }
    }

    fn has_line_of_sight(&self, x0: usize, y0: usize, x1: usize, y1: usize) -> bool {
        let (mut x0, mut y0) = (x0 as i32, y0 as i32);
        let (x1, y1) = (x1 as i32, y1 as i32);
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let mut err = dx + dy;

        loop {
            if x0 == x1 && y0 == y1 {
                return true;
            }

            let point = Point::new(x0 as Coordinate, y0 as Coordinate);

            if self.is_opaque(point) {
                return false;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn is_opaque(&self, point: Point) -> bool {
        matches!(
            self.get_tile(point),
            Tile::Wall { .. } | Tile::Door { open: false, .. }
        )
    }

    fn update_tile_visibility(&mut self, point: Point, visible: bool) {
        let tile = self.get_tile(point);
        let new_tile = match tile {
            Tile::Wall { .. } => Tile::Wall { visible },
            Tile::Floor { cursed, .. } => Tile::Floor { cursed, visible },
            Tile::Pit { .. } => Tile::Pit { visible },
            Tile::Secret { .. } => Tile::Secret { visible },
            Tile::SecretFloor { .. } => Tile::SecretFloor { visible },
            Tile::Door { open, .. } => Tile::Door { open, visible },
            Tile::Obelisk {
                curse,
                fov,
                damage_hp,
                reduce_fov_radius,
                ..
            } => Tile::Obelisk {
                visible,
                curse,
                fov,
                damage_hp,
                reduce_fov_radius,
            },
            // Update other tile types as needed
            _ => tile,
        };
        self.set_tile(point, new_tile);
    }

    fn clear_curse_from_all_tiles(&mut self) {
        for y in 0..self.height() {
            for x in 0..self.width() {
                let point = Point::new(x, y);

                if let Tile::Floor {
                    cursed: true,
                    visible,
                } = self.get_tile(point)
                {
                    self.set_tile(
                        point,
                        Tile::Floor {
                            cursed: false,
                            visible,
                        },
                    );
                } else if let Tile::Player {
                    is_cursed: true,
                    is_dead,
                } = self.get_tile(point)
                {
                    self.set_tile(
                        point,
                        Tile::Player {
                            is_cursed: false,
                            is_dead,
                        },
                    );
                }
            }
        }
    }

    pub fn apply_obelisk_curses(&mut self) {
        self.clear_curse_from_all_tiles();
        let mut cursed_tiles = HashSet::new();

        // First pass: identify all Obelisks and their curse areas
        for y in 0..self.height() {
            for x in 0..self.width() {
                let point = Point::new(x, y);

                if let Tile::Obelisk {
                    curse: true, fov, ..
                } = self.get_tile(point)
                {
                    self.calculate_curse_area(point, fov, &mut cursed_tiles);
                }
            }
        }

        // Second pass: apply curses to the identified tiles
        for point in cursed_tiles {
            if let Tile::Floor { visible, .. } = self.get_tile(point) {
                self.set_tile(
                    point,
                    Tile::Floor {
                        visible,
                        cursed: true,
                    },
                );
            } else if let Tile::Player { is_dead, .. } = self.get_tile(point) {
                self.set_tile(
                    point,
                    Tile::Player {
                        is_dead,
                        is_cursed: true,
                    },
                );
            }
        }
    }

    pub fn get_obelisk_cursing_tile(&self, pov: Point) -> Option<Tile> {
        // get nearest obelisk with direct line of sight to the player
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let point = Point::new(x, y);

                if let Tile::Obelisk { curse: true, .. } = self.get_tile(point) {
                    if self.has_line_of_sight(point.x, point.y, pov.x, pov.y) {
                        return Some(*tile);
                    }
                }
            }
        }

        // Return None if no cursing obelisk is found or the player is not on a cursed tile
        None
    }

    fn calculate_curse_area(&self, center: Point, radius: u32, cursed_tiles: &mut HashSet<Point>) {
        let radius = radius as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let x = center.x as i32 + dx;
                    let y = center.y as i32 + dy;

                    if (0..self.width() as i32).contains(&x)
                        && (0..self.height() as i32).contains(&y)
                        && self.has_line_of_sight(x as usize, y as usize, center.x, center.y)
                    {
                        let point = Point::new(x as Coordinate, y as Coordinate);

                        cursed_tiles.insert(point);
                    }
                }
            }
        }
    }
}
