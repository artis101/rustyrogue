use crate::map::Map;
use crate::player::Player;
use crate::tile::Tile;
use std::cmp::{max, min};
use std::io;

pub enum MessageType {
    Info,
    Damage,
}

pub struct GameMessage {
    pub message: String,
    pub message_type: MessageType,
}

pub struct Game {
    map: Map,
    player: Player,
    player_x: usize,
    player_y: usize,
    previous_tile: Tile,
    turns: u32,
    log_messages: Vec<GameMessage>, // Game log is a FIFO queue of 5 messages
}

impl Game {
    pub fn new(map_file: &str) -> io::Result<Self> {
        let full_map_file = format!("maps/{}.txt", map_file);
        let map_hint_file = format!("maps/{}_hint.txt", map_file);
        let map = Map::load(&full_map_file, &map_hint_file)?;
        let player = Player::new();
        let (player_x, player_y) = map.find_player().unwrap_or((1, 1));
        let mut game = Game {
            map,
            player,
            player_x,
            player_y,
            previous_tile: Tile::Floor { visible: false },
            turns: 0,
            log_messages: Vec::with_capacity(5),
        };

        // Perform initial FOV update
        game.update_fov();

        Ok(game)
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

            self.update_fov();

            // Increment turn counter
            self.turns += 1;

            // Simulate gaining experience (you can modify this logic later)
            self.player.gain_exp(1);
        }
    }

    fn update_fov(&mut self) {
        let fov_radius = self.player.fov_radius;
        self.map
            .update_fov(self.player_x, self.player_y, fov_radius);
    }

    pub fn get_map(&self) -> &Vec<Vec<Tile>> {
        self.map.get_tiles()
    }

    pub fn get_player(&self) -> &Player {
        &self.player
    }

    pub fn log_message(&mut self, message: String, message_type: MessageType) {
        if self.log_messages.len() == 5 {
            self.log_messages.remove(0);
        }
        self.log_messages.push(GameMessage {
            message,
            message_type,
        });
    }

    pub fn log_info_message(&mut self, message: String) {
        self.log_message(message, MessageType::Info);
    }

    pub fn log_damage_message(&mut self, message: String) {
        self.log_message(message, MessageType::Damage);
    }

    pub fn get_log_messages(&self) -> &Vec<GameMessage> {
        &self.log_messages
    }

    pub fn interact(&mut self) {
        let directions = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        for (dx, dy) in directions.iter() {
            let new_x = max(
                0,
                min(self.map.width() as i32 - 1, self.player_x as i32 + dx),
            ) as usize;
            let new_y = max(
                0,
                min(self.map.height() as i32 - 1, self.player_y as i32 + dy),
            ) as usize;

            if self.map.is_interactable(new_x, new_y) {
                self.map.interact_tile(new_x, new_y);
                self.update_fov(); // upddate FOV after interacting to reveal hidden tiles

                if let Tile::Door { open, .. } = self.map.get_tile(new_x, new_y) {
                    if open {
                        self.log_info_message("You close the door".to_string());
                    } else {
                        self.log_info_message("You open the door".to_string());
                    }

                    // hurt the player for now
                    self.player.take_damage(1);
                    self.log_damage_message("You take 1 damage from the door".to_string());
                }

                return;
            }
        }

        self.log_info_message("No interactable tiles found nearby.".to_string());
    }
}
