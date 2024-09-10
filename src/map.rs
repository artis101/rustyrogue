use crate::tile::Tile;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub struct Map {
    tiles: Vec<Vec<Tile>>,
    visible_tiles: HashSet<(usize, usize)>,
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

    pub fn width(&self) -> usize {
        self.tiles.first().map(|row| row.len()).unwrap_or(0)
    }

    pub fn height(&self) -> usize {
        self.tiles.len()
    }

    pub fn get_tiles(&self) -> &Vec<Vec<Tile>> {
        &self.tiles
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Tile {
        self.tiles
            .get(y)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or(Tile::Empty)
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if let Some(row) = self.tiles.get_mut(y) {
            if let Some(t) = row.get_mut(x) {
                *t = tile;
            }
        }
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        self.get_tile(x, y).is_walkable()
    }

    pub fn is_deadly(&self, x: usize, y: usize) -> bool {
        matches!(self.get_tile(x, y), Tile::Pit { .. })
    }

    pub fn find_player(&self) -> Option<(usize, usize)> {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, &tile) in row.iter().enumerate() {
                if let Tile::Player { .. } = tile {
                    return Some((x, y));
                }
            }
        }
        None
    }

    pub fn is_interactable(&self, x: usize, y: usize) -> bool {
        matches!(
            self.tiles[y][x],
            Tile::Door { .. } | Tile::Secret { visible: true } // you can interact with visible
                                                               // things for now
        )
    }

    pub fn interact_tile(&mut self, x: usize, y: usize) {
        if let Tile::Door { open, visible } = self.tiles[y][x] {
            self.tiles[y][x] = Tile::Door {
                open: !open,
                visible,
            };
        }
    }

    fn clear_visible_tiles(&mut self) {
        let points_to_clear: Vec<(usize, usize)> = self.visible_tiles.drain().collect();

        for (x, y) in points_to_clear {
            self.update_tile_visibility(x, y, false);
        }
    }

    pub fn update_fov(&mut self, pov_x: usize, pov_y: usize, fov_radius: u32) {
        self.clear_visible_tiles();
        let radius_squared = (fov_radius * fov_radius) as i32;

        for dy in -(fov_radius as i32)..=fov_radius as i32 {
            for dx in -(fov_radius as i32)..=fov_radius as i32 {
                if dx * dx + dy * dy <= radius_squared {
                    let x = pov_x as i32 + dx;
                    let y = pov_y as i32 + dy;

                    if (0..self.width() as i32).contains(&x)
                        && (0..self.height() as i32).contains(&y)
                        && self.has_line_of_sight(pov_x, pov_y, x as usize, y as usize)
                    {
                        let (x, y) = (x as usize, y as usize);
                        self.visible_tiles.insert((x, y));
                        self.update_tile_visibility(x, y, true);
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
            if self.is_opaque(x0 as usize, y0 as usize) {
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

    fn is_opaque(&self, x: usize, y: usize) -> bool {
        matches!(
            self.get_tile(x, y),
            Tile::Wall { .. } | Tile::Door { open: false, .. }
        )
    }

    fn update_tile_visibility(&mut self, x: usize, y: usize, visible: bool) {
        let tile = self.get_tile(x, y);
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
        self.set_tile(x, y, new_tile);
    }

    fn clear_curse_from_all_tiles(&mut self) {
        for y in 0..self.height() {
            for x in 0..self.width() {
                if let Tile::Floor {
                    cursed: true,
                    visible,
                } = self.get_tile(x, y)
                {
                    self.set_tile(
                        x,
                        y,
                        Tile::Floor {
                            cursed: false,
                            visible,
                        },
                    );
                } else if let Tile::Player {
                    is_cursed: true,
                    is_dead,
                } = self.get_tile(x, y)
                {
                    self.set_tile(
                        x,
                        y,
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
                if let Tile::Obelisk {
                    curse: true, fov, ..
                } = self.get_tile(x, y)
                {
                    self.calculate_curse_area(x, y, fov, &mut cursed_tiles);
                }
            }
        }

        // Second pass: apply curses to the identified tiles
        for (x, y) in cursed_tiles {
            if let Tile::Floor { visible, .. } = self.get_tile(x, y) {
                self.set_tile(
                    x,
                    y,
                    Tile::Floor {
                        visible,
                        cursed: true,
                    },
                );
            } else if let Tile::Player { is_dead, .. } = self.get_tile(x, y) {
                self.set_tile(
                    x,
                    y,
                    Tile::Player {
                        is_dead,
                        is_cursed: true,
                    },
                );
            }
        }
    }

    pub fn get_obelisk_cursing_tile(&self, pov_x: usize, pov_y: usize) -> Option<Tile> {
        // get nearest obelisk with direct line of sight to the player
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if let Tile::Obelisk { curse: true, .. } = self.get_tile(x, y) {
                    if self.has_line_of_sight(x, y, pov_x, pov_y) {
                        return Some(*tile);
                    }
                }
            }
        }

        // Return None if no cursing obelisk is found or the player is not on a cursed tile
        None
    }

    fn calculate_curse_area(
        &self,
        center_x: usize,
        center_y: usize,
        radius: u32,
        cursed_tiles: &mut HashSet<(usize, usize)>,
    ) {
        let radius = radius as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let x = center_x as i32 + dx;
                    let y = center_y as i32 + dy;

                    if (0..self.width() as i32).contains(&x)
                        && (0..self.height() as i32).contains(&y)
                        && self.has_line_of_sight(x as usize, y as usize, center_x, center_y)
                    {
                        cursed_tiles.insert((x as usize, y as usize));
                    }
                }
            }
        }
    }
}
