use types::{Coordinate, GameMapTiles, Point};

use crate::tile::Tile;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::sync::{Arc, RwLock};

pub mod types;

pub struct Map {
    tiles: Arc<RwLock<GameMapTiles>>,
    visible_tiles: HashSet<Point>,
}

impl Map {
    #[allow(dead_code)]
    pub fn load(filename: &str) -> io::Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut tiles = Vec::new();

        for line in reader.lines() {
            let line = line?;
            tiles.push(line.chars().map(Tile::from_char).collect());
        }

        Ok(Map {
            tiles: Arc::new(RwLock::new(tiles)),
            visible_tiles: HashSet::new(),
        })
    }

    pub fn width(&self) -> usize {
        let tiles = self.tiles.read().unwrap();
        tiles[0].len()
    }

    pub fn height(&self) -> usize {
        let tiles = self.tiles.read().unwrap();
        tiles.len()
    }

    pub fn from_tiles(tiles: Arc<RwLock<GameMapTiles>>) -> Self {
        Map {
            tiles,
            visible_tiles: HashSet::new(),
        }
    }

    pub fn get_tiles(&self) -> &Arc<RwLock<GameMapTiles>> {
        &self.tiles
    }

    pub fn is_walkable(&self, position: Point) -> bool {
        let tiles = self.tiles.read().unwrap();
        let tile = tiles[position.y][position.x];
        tile.is_walkable()
    }

    pub fn set_tile(&mut self, position: Point, tile: Tile) {
        let mut tiles = self.tiles.write().unwrap();
        tiles[position.y][position.x] = tile;
    }

    pub fn get_tile(&self, position: Point) -> Tile {
        let tiles = self.tiles.read().unwrap();
        tiles[position.y][position.x]
    }

    pub fn is_deadly(&self, point: Point) -> bool {
        matches!(self.get_tile(point), Tile::Pit { .. })
    }

    #[allow(dead_code)]
    pub fn find_player(&self) -> Option<Point> {
        let tiles = self.tiles.read().unwrap();
        for (y, row) in tiles.iter().enumerate() {
            for (x, &tile) in row.iter().enumerate() {
                if let Tile::Player { .. } = tile {
                    return Some(Point { x, y });
                }
            }
        }
        None
    }

    pub fn is_interactable(&self, point: Point) -> bool {
        let tiles = self.tiles.read().unwrap();
        matches!(
            tiles[point.y][point.x],
            Tile::Door { .. } | Tile::Secret { visible: true, .. }
        )
    }

    pub fn interact_tile(&mut self, point: Point) {
        let mut tiles = self.tiles.write().unwrap();
        if let Tile::Door { open, visible } = tiles[point.y][point.x] {
            tiles[point.y][point.x] = Tile::Door {
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

        let is_player_cursed = matches!(
            self.get_tile(pov),
            Tile::Player {
                is_cursed: true,
                ..
            }
        );

        let radius_squared = (fov_radius * fov_radius) as i32;

        for dy in -(fov_radius as i32)..=fov_radius as i32 {
            for dx in -(fov_radius as i32)..=fov_radius as i32 {
                if dx * dx + dy * dy <= radius_squared {
                    let x = pov.x as i32 + dx;
                    let y = pov.y as i32 + dy;

                    if (0..self.width() as i32).contains(&x)
                        && (0..self.height() as i32).contains(&y)
                        && self.has_line_of_sight(
                            pov.x,
                            pov.y,
                            x as usize,
                            y as usize,
                            false,
                            is_player_cursed,
                        )
                    {
                        let point = Point::new(x as Coordinate, y as Coordinate);
                        self.visible_tiles.insert(point);
                        self.update_tile_visibility(point, true);
                    }
                }
            }
        }
    }

    fn has_line_of_sight(
        &self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        by_obelisk: bool,
        is_player_cursed: bool,
    ) -> bool {
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

            if self.is_opaque(point, by_obelisk, is_player_cursed) {
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

    fn is_opaque(&self, point: Point, by_obelisk: bool, is_player_cursed: bool) -> bool {
        let tile = self.get_tile(point);
        if by_obelisk {
            matches!(
                tile,
                Tile::Wall { .. }
                    | Tile::Door { open: false, .. }
                    | Tile::Column { .. }
                    | Tile::Pit { .. }
            )
        } else if is_player_cursed {
            // you can see around inside the curse area
            matches!(
                tile,
                Tile::Wall { .. }
                    | Tile::Door { open: false, .. }
                    | Tile::Column { .. }
                    | Tile::Secret { .. }
                    | Tile::Wither { .. }
                    | Tile::Bat { .. }
                    | Tile::Brute { .. }
            )
        } else {
            // you cannot see inside cursed areas
            matches!(
                tile,
                Tile::Wall { .. }
                    | Tile::Floor { cursed: true, .. }
                    | Tile::Door { open: false, .. }
                    | Tile::Column { .. }
                    | Tile::Secret { .. }
                    | Tile::Wither { .. }
                    | Tile::Bat { .. }
                    | Tile::Brute { .. }
            )
        }
    }

    fn update_tile_visibility(&mut self, point: Point, visible: bool) {
        let mut tiles_write = self.tiles.write().unwrap();
        let tile = &mut tiles_write[point.y][point.x];
        match tile {
            Tile::Wall { .. } => {
                *tile = Tile::Wall { visible };
            }
            Tile::Pit { .. } => {
                *tile = Tile::Pit { visible };
            }
            Tile::Floor { cursed, .. } => {
                *tile = Tile::Floor {
                    cursed: *cursed,
                    visible,
                };
            }
            Tile::Secret { rarity, .. } => {
                *tile = Tile::Secret {
                    rarity: *rarity,
                    visible,
                };
            }
            Tile::SecretFloor { .. } => {
                *tile = Tile::SecretFloor { visible };
            }
            Tile::Door { open, .. } => {
                *tile = Tile::Door {
                    open: *open,
                    visible,
                };
            }
            Tile::Obelisk {
                curse,
                fov,
                damage_hp,
                reduce_fov_radius,
                ..
            } => {
                *tile = Tile::Obelisk {
                    curse: *curse,
                    fov: *fov,
                    damage_hp: *damage_hp,
                    reduce_fov_radius: *reduce_fov_radius,
                    visible,
                };
            }
            Tile::Wither {
                hp, damage, fov, ..
            } => {
                *tile = Tile::Wither {
                    hp: *hp,
                    damage: *damage,
                    fov: *fov,
                    visible,
                };
            }
            Tile::Bat {
                hp, damage, fov, ..
            } => {
                *tile = Tile::Bat {
                    hp: *hp,
                    damage: *damage,
                    fov: *fov,
                    visible,
                };
            }
            Tile::Brute {
                hp, damage, fov, ..
            } => {
                *tile = Tile::Brute {
                    hp: *hp,
                    damage: *damage,
                    fov: *fov,
                    visible,
                };
            }
            Tile::Player { is_dead, .. } => {
                *tile = Tile::Player {
                    is_dead: *is_dead,
                    is_cursed: false,
                };
            }
            _ => {}
        }
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
        let tiles = self.tiles.read().unwrap();
        // get nearest obelisk with direct line of sight to the player
        for (y, row) in tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let point = Point::new(x, y);

                if let Tile::Obelisk { curse: true, .. } = self.get_tile(point) {
                    if self.has_line_of_sight(point.x, point.y, pov.x, pov.y, true, false) {
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
                        && self.has_line_of_sight(
                            x as usize, y as usize, center.x, center.y, true, false,
                        )
                    {
                        let point = Point::new(x as Coordinate, y as Coordinate);

                        cursed_tiles.insert(point);
                    }
                }
            }
        }
    }
}
