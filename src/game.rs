use crate::map::Map;
use crate::tile::Tile;
use std::io;

pub struct Game {
    map: Map,
    player_x: usize,
    player_y: usize,
    previous_tile: Tile,
}

impl Game {
    pub fn new(map_file: &str) -> io::Result<Self> {
        let map = Map::load(map_file)?;
        let (player_x, player_y) = map.find_player().unwrap_or((1, 1));
        Ok(Game {
            map,
            player_x,
            player_y,
            previous_tile: Tile::Floor,
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
            // Restore the previous tile
            self.map
                .set_tile(self.player_x, self.player_y, self.previous_tile);

            // Store the new tile before moving onto it
            self.previous_tile = self.map.get_tile(new_x, new_y);

            // Update player position
            self.player_x = new_x;
            self.player_y = new_y;

            // Place the player on the new tile
            self.map
                .set_tile(self.player_x, self.player_y, Tile::Player);
        }
    }

    pub fn get_map(&self) -> &Vec<Vec<Tile>> {
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
            if self.map.get_tile(x, y) == Tile::Door {
                return Some(format!("Door found at ({}, {})", x, y));
            }
        }
        None
    }
}
