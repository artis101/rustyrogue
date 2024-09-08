use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub struct Map {
    tiles: Vec<Vec<char>>,
}

impl Map {
    pub fn load(filename: &str) -> io::Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut tiles = Vec::new();

        for line in reader.lines() {
            let line = line?;
            tiles.push(line.chars().collect());
        }

        Ok(Map { tiles })
    }

    pub fn width(&self) -> usize {
        self.tiles.get(0).map(|row| row.len()).unwrap_or(0)
    }

    pub fn height(&self) -> usize {
        self.tiles.len()
    }

    pub fn get_tiles(&self) -> &Vec<Vec<char>> {
        &self.tiles
    }

    pub fn get_tile(&self, x: usize, y: usize) -> char {
        self.tiles
            .get(y)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or('.')
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: char) {
        if let Some(row) = self.tiles.get_mut(y) {
            if let Some(t) = row.get_mut(x) {
                *t = tile;
            }
        }
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        match self.get_tile(x, y) {
            '#' | 'o' => false,
            _ => true,
        }
    }

    pub fn find_player(&self) -> Option<(usize, usize)> {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, &tile) in row.iter().enumerate() {
                if tile == '@' {
                    return Some((x, y));
                }
            }
        }
        None
    }
}

