use crate::map::Map;
use std::io;

pub struct Game {
    map: Map,
    player_x: usize,
    player_y: usize,
}

impl Game {
    pub fn new(map_file: &str) -> io::Result<Self> {
        let map = Map::load(map_file)?;
        let (player_x, player_y) = map.find_player().unwrap_or((1, 1));
        Ok(Game {
            map,
            player_x,
            player_y,
        })
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let new_x = (self.player_x as i32 + dx)
            .max(0)
            .min((self.map.width() - 1) as i32) as usize;
        let new_y = (self.player_y as i32 + dy)
            .max(0)
            .min((self.map.height() - 1) as i32) as usize;

        if self.map.is_walkable(new_x, new_y) {
            self.map.set_tile(self.player_x, self.player_y, '.');
            self.player_x = new_x;
            self.player_y = new_y;
            self.map.set_tile(self.player_x, self.player_y, '@');
        }
    }

    pub fn get_map(&self) -> &Vec<Vec<char>> {
        self.map.get_tiles()
    }

    pub fn get_door_message(&self) -> Option<String> {
        let directions = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        for (dx, dy) in directions.iter() {
            let x = (self.player_x as i32 + dx)
                .max(0)
                .min((self.map.width() - 1) as i32) as usize;
            let y = (self.player_y as i32 + dy)
                .max(0)
                .min((self.map.height() - 1) as i32) as usize;
            if self.map.get_tile(x, y) == '+' {
                return Some(format!("Door found at ({}, {})", x, y));
            }
        }
        None
    }
}
