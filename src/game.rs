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

const DIRECTIONS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

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
            previous_tile: Tile::Floor {
                visible: false,
                cursed: false,
            },
            turns: 0,
            log_messages: Vec::with_capacity(5),
        };

        // Perform initial FOV update
        game.tick();

        Ok(game)
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let search_x = (self.player_x as i32 + dx)
            .max(0)
            .min((self.map.width() - 1) as i32) as usize;
        let search_y = (self.player_y as i32 + dy)
            .max(0)
            .min((self.map.height() - 1) as i32) as usize;

        self.walk_to_tile(search_x, search_y);

        self.tick();
    }

    fn walk_to_tile(&mut self, search_x: usize, search_y: usize) {
        if self.map.is_walkable(search_x, search_y) {
            // Restore the previous tile
            self.map
                .set_tile(self.player_x, self.player_y, self.previous_tile);

            // Store the new tile before moving onto it
            self.previous_tile = self.map.get_tile(search_x, search_y);

            // Update player position
            self.player_x = search_x;
            self.player_y = search_y;

            // Place the player on the new tile
            self.map.set_tile(
                self.player_x,
                self.player_y,
                Tile::Player {
                    is_dead: false,
                    is_cursed: false, // don't need to check for curses here
                                      // curses are checked in update_fov
                },
            );
        }
    }

    fn tick(&mut self) {
        self.turns += 1;
        self.map.apply_obelisk_curses();
        self.update_fov();
        self.check_effects();
    }

    fn check_effects(&mut self) {
        if self.is_player_cursed() {
            self.apply_curse_effects();
        } else {
            self.player.gain_exp(1); // @TODO implement normal exp gain
        }
    }

    fn apply_curse_effects(&mut self) {
        let cursing_tile = self
            .map
            .get_obelisk_cursing_tile(self.player_x, self.player_y);
        if let Some(Tile::Obelisk {
            visible: true,
            damage_hp,
            ..
        }) = cursing_tile
        {
            self.log_damage_message(format!(
                "You take {} damage from the Obelisk curse",
                damage_hp
            ));
            self.player.take_damage(damage_hp);
        }
    }

    fn is_player_cursed(&self) -> bool {
        matches!(
            self.map.get_tile(self.player_x, self.player_y),
            Tile::Player {
                is_cursed: true,
                ..
            }
        )
    }

    fn get_player_fov_radius(&self) -> u32 {
        if self.is_player_cursed() {
            if let Some(Tile::Obelisk {
                reduce_fov_radius, ..
            }) = self
                .map
                .get_obelisk_cursing_tile(self.player_x, self.player_y)
            {
                reduce_fov_radius
            } else {
                self.player.fov_radius
            }
        } else {
            self.player.fov_radius
        }
    }

    fn update_fov(&mut self) {
        let fov_radius = self.get_player_fov_radius();
        self.map
            .update_fov(self.player_x, self.player_y, fov_radius);
    }

    pub fn get_map(&self) -> &Vec<Vec<Tile>> {
        self.map.get_tiles()
    }

    pub fn get_player(&self) -> &Player {
        &self.player
    }

    pub fn is_game_over(&self) -> bool {
        self.player.is_dead()
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
        for (dx, dy) in DIRECTIONS.iter() {
            let search_x = max(
                0,
                min(self.map.width() as i32 - 1, self.player_x as i32 + dx),
            ) as usize;
            let search_y = max(
                0,
                min(self.map.height() as i32 - 1, self.player_y as i32 + dy),
            ) as usize;

            if self.map.is_interactable(search_x, search_y) {
                self.map.interact_tile(search_x, search_y);
                self.tick();
                return;
            }
        }

        self.log_info_message("No interactable tiles found nearby.".to_string());
    }
}
