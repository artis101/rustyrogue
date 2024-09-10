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

    pub fn clear_visible_tiles(&mut self) {
        // Collect the coordinates into a separate Vec to avoid borrowing issues
        let coords: Vec<(usize, usize)> = self.visible_tiles.iter().cloned().collect();

        // Clear the visible_tiles set
        self.visible_tiles.clear();

        // Update tile visibility
        for (x, y) in coords {
            self.update_tile_visibility(x, y, false);
        }
    }

    pub fn update_fov(&mut self, player_x: usize, player_y: usize, fov_radius: u32) {
        self.clear_visible_tiles();
        let radius_squared = (fov_radius * fov_radius) as i32;

        for dy in -(fov_radius as i32)..=(fov_radius as i32) {
            for dx in -(fov_radius as i32)..=(fov_radius as i32) {
                let distance_squared = dx * dx + dy * dy;
                if distance_squared <= radius_squared {
                    let x = player_x as i32 + dx;
                    let y = player_y as i32 + dy;
                    if x >= 0
                        && x < self.width() as i32
                        && y >= 0
                        && y < self.height() as i32
                        && self.has_line_of_sight(player_x, player_y, x as usize, y as usize)
                    {
                        self.visible_tiles.insert((x as usize, y as usize));
                        self.update_tile_visibility(x as usize, y as usize, true);
                    }
                }
            }
        }
    }

    fn has_line_of_sight(&self, x0: usize, y0: usize, x1: usize, y1: usize) -> bool {
        // Implement Bresenham's line algorithm
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
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
            Tile::Floor { .. } => Tile::Floor { visible },
            Tile::Pit { .. } => Tile::Pit { visible },
            Tile::Secret { .. } => Tile::Secret { visible },
            Tile::SecretFloor { .. } => Tile::SecretFloor { visible },
            Tile::Door { open, .. } => Tile::Door { open, visible },
            Tile::Obelisk {
                curse,
                proximity,
                damage_hp,
                reduce_max_hp,
                reduce_strength,
                reduce_fov_radius,
                reduce_defense,
                ..
            } => Tile::Obelisk {
                visible,
                curse,
                proximity,
                damage_hp,
                reduce_max_hp,
                reduce_strength,
                reduce_fov_radius,
                reduce_defense,
            },
            // Update other tile types as needed
            _ => tile,
        };
        self.set_tile(x, y, new_tile);
    }
}
