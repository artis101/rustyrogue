use crate::tile::Tile;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub struct Map {
    tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn load(filename: &str) -> io::Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut tiles = Vec::new();

        for line in reader.lines() {
            let line = line?;
            tiles.push(line.chars().map(Tile::from_char).collect());
        }

        Ok(Map { tiles })
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
                if tile == Tile::Player {
                    return Some((x, y));
                }
            }
        }
        None
    }

    pub fn is_interactable(&self, x: usize, y: usize) -> bool {
        matches!(self.tiles[y][x], Tile::Door { .. })
    }

    pub fn interact_tile(&mut self, x: usize, y: usize) {
        if let Tile::Door { open } = self.tiles[y][x] {
            self.tiles[y][x] = Tile::Door { open: !open };
        }
    }
}
