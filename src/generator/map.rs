use crate::generator::room::Room;
use crate::map::types::{Coordinate, GameMapTiles, Point};
use crate::tile::Tile;
use rand::Rng;
use rayon::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct BSPNode {
    x: Coordinate,
    y: Coordinate,
    width: Coordinate,
    height: Coordinate,
    left: Option<Box<BSPNode>>,
    right: Option<Box<BSPNode>>,
    room: Option<Room>,
}

impl BSPNode {
    fn new(x: Coordinate, y: Coordinate, width: Coordinate, height: Coordinate) -> Self {
        BSPNode {
            x,
            y,
            width,
            height,
            left: None,
            right: None,
            room: None,
        }
    }

    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

pub struct MapGenerator {
    width: Coordinate,
    height: Coordinate,
    tiles: Arc<RwLock<GameMapTiles>>,
    rooms: Vec<Room>,
    bsp_root: Option<BSPNode>,
}

impl MapGenerator {
    const TURN_PROBABILITY: f64 = 0.2; // 20% chance to turn at each step

    pub fn new(width: Coordinate, height: Coordinate) -> Self {
        MapGenerator {
            width,
            height,
            tiles: Arc::new(RwLock::new(vec![vec![Tile::Empty; width]; height])),
            rooms: Vec::new(),
            bsp_root: None,
        }
    }

    pub fn generate(&mut self, min_room_size: Coordinate, max_room_size: Coordinate) -> &mut Self {
        self.fill_with_empty();

        // Build the BSP tree
        self.build_bsp_tree(min_room_size);

        // Create rooms in the leaf nodes
        self.create_rooms_in_bsp(min_room_size, max_room_size);

        // Populate rooms in parallel using Rayon
        self.populate_all_rooms();

        // Connect rooms via depth-first traversal
        if let Some(ref root) = self.bsp_root {
            self.connect_rooms_bsp(root);
        }

        self.place_all_room_doors();

        self
    }

    pub fn get_dungeon(&self) -> Arc<RwLock<GameMapTiles>> {
        Arc::clone(&self.tiles)
    }

    pub fn get_rooms(&self) -> &Vec<Room> {
        &self.rooms
    }

    fn fill_with_empty(&self) {
        let mut tiles = self.tiles.write().unwrap();
        for y in 0..self.height {
            for x in 0..self.width {
                tiles[y][x] = Tile::Empty;
            }
        }
    }

    fn build_bsp_tree(&mut self, min_size: Coordinate) {
        let root = BSPNode::new(0, 0, self.width, self.height);
        self.bsp_root = Some(root);
        let max_depth = 5; // Adjust this value to control the depth of the tree
        if let Some(ref mut root_node) = self.bsp_root {
            Self::split_node(root_node, min_size, max_depth, 0);
        }
    }

    fn split_node(
        node: &mut BSPNode,
        min_size: Coordinate,
        max_depth: usize,
        current_depth: usize,
    ) {
        if current_depth >= max_depth {
            return;
        }

        let can_split_horizontally = node.width >= min_size * 2;
        let can_split_vertically = node.height >= min_size * 2;

        if !can_split_horizontally && !can_split_vertically {
            return;
        }

        let split_vertically = if can_split_horizontally && can_split_vertically {
            rand::random::<bool>()
        } else {
            can_split_horizontally
        };

        if split_vertically {
            // Split vertically
            let split = rand::thread_rng().gen_range(min_size..(node.width - min_size + 1));
            node.left = Some(Box::new(BSPNode::new(node.x, node.y, split, node.height)));
            node.right = Some(Box::new(BSPNode::new(
                node.x + split,
                node.y,
                node.width - split,
                node.height,
            )));
        } else {
            // Split horizontally
            let split = rand::thread_rng().gen_range(min_size..(node.height - min_size + 1));
            node.left = Some(Box::new(BSPNode::new(node.x, node.y, node.width, split)));
            node.right = Some(Box::new(BSPNode::new(
                node.x,
                node.y + split,
                node.width,
                node.height - split,
            )));
        }

        if let Some(ref mut left) = node.left {
            Self::split_node(left, min_size, max_depth, current_depth + 1);
        }
        if let Some(ref mut right) = node.right {
            Self::split_node(right, min_size, max_depth, current_depth + 1);
        }
    }

    fn create_rooms_in_bsp(&mut self, min_room_size: Coordinate, max_room_size: Coordinate) {
        let mut rooms = Vec::new();
        if let Some(ref mut root) = self.bsp_root {
            Self::collect_rooms_in_node(root, &mut rooms, min_room_size, max_room_size);
        }
        self.rooms = rooms;
    }

    fn collect_rooms_in_node(
        node: &mut BSPNode,
        rooms: &mut Vec<Room>,
        min_room_size: Coordinate,
        max_room_size: Coordinate,
    ) {
        if node.is_leaf() {
            let mut rng = rand::thread_rng();
            let padding = 2; // Padding between the room and the partition edges

            let max_room_width = (node.width - padding * 2).min(max_room_size);
            let max_room_height = (node.height - padding * 2).min(max_room_size);

            if max_room_width < min_room_size || max_room_height < min_room_size {
                // If the partition is too small, skip room creation
                return;
            }

            let room_width = rng.gen_range(min_room_size..=max_room_width);
            let room_height = rng.gen_range(min_room_size..=max_room_height);

            let x_range = node.x + padding..=node.x + node.width - room_width - padding;
            let y_range = node.y + padding..=node.y + node.height - room_height - padding;

            let room_x = if x_range.is_empty() {
                node.x + padding
            } else {
                rng.gen_range(x_range)
            };

            let room_y = if y_range.is_empty() {
                node.y + padding
            } else {
                rng.gen_range(y_range)
            };

            let location = Point::new(room_x, room_y);

            let room = Room::new(location, room_width, room_height);
            node.room = Some(room.clone());
            rooms.push(room);
        } else {
            if let Some(ref mut left) = node.left {
                Self::collect_rooms_in_node(left, rooms, min_room_size, max_room_size);
            }
            if let Some(ref mut right) = node.right {
                Self::collect_rooms_in_node(right, rooms, min_room_size, max_room_size);
            }
        }
    }

    fn populate_all_rooms(&mut self) {
        let tiles = Arc::clone(&self.tiles);

        // Use Rayon to populate rooms in parallel
        self.rooms.par_iter_mut().for_each(|room| {
            let tiles_clone = Arc::clone(&tiles);
            room.populate(&tiles_clone);
        });
    }

    fn place_all_room_doors(&self) {
        let tiles = Arc::clone(&self.tiles);
        // Use Rayon to place doors in parallel
        self.rooms.par_iter().for_each(|room| {
            let tiles_clone = Arc::clone(&tiles);
            room.place_doors(&tiles_clone);
        });
    }

    fn connect_rooms_bsp(&self, node: &BSPNode) {
        if !node.is_leaf() {
            if let (Some(left), Some(right)) = (node.left.as_ref(), node.right.as_ref()) {
                self.connect_rooms_bsp(left);
                self.connect_rooms_bsp(right);

                let left_room = Self::get_room_in_subtree(left);
                let right_room = Self::get_room_in_subtree(right);
                if let (Some(lr), Some(rr)) = (left_room, right_room) {
                    self.drunken_walk_corridor(lr.center(), rr.center());
                }
            }
        }
    }

    fn get_room_in_subtree(node: &BSPNode) -> Option<&Room> {
        if let Some(ref room) = node.room {
            Some(room)
        } else {
            node.left
                .as_ref()
                .and_then(|left| Self::get_room_in_subtree(left))
                .or_else(|| {
                    node.right
                        .as_ref()
                        .and_then(|right| Self::get_room_in_subtree(right))
                })
        }
    }

    fn drunken_walk_corridor(&self, start: Point, end: Point) {
        let mut rng = rand::thread_rng();
        let mut current = start;

        // Randomly choose the initial direction
        let mut direction = if rng.gen_bool(0.5) { 'x' } else { 'y' };

        while current != end {
            self.carve_corridor(current.x, current.y);

            let dx = end.x as isize - current.x as isize;
            let dy = end.y as isize - current.y as isize;

            // Check if we need to change direction
            let at_end_in_direction = match direction {
                'x' => dx == 0,
                'y' => dy == 0,
                _ => false,
            };

            // Decide whether to turn based on TURN_PROBABILITY
            if at_end_in_direction || rng.gen_bool(Self::TURN_PROBABILITY) {
                // Change direction
                direction = if direction == 'x' { 'y' } else { 'x' };
            }

            // Move in the current direction
            match direction {
                'x' => {
                    if dx != 0 {
                        current.x = (current.x as isize + dx.signum())
                            .clamp(0, (self.width - 1) as isize)
                            as Coordinate;
                    }
                }
                'y' => {
                    if dy != 0 {
                        current.y = (current.y as isize + dy.signum())
                            .clamp(0, (self.height - 1) as isize)
                            as Coordinate;
                    }
                }
                _ => {}
            }
        }
    }

    fn carve_corridor(&self, x: Coordinate, y: Coordinate) {
        let mut tiles = self.tiles.write().unwrap();
        let max_x = self.width;
        let max_y = self.height;

        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = (x as isize + dx).clamp(0, (max_x - 1) as isize) as Coordinate;
                let ny = (y as isize + dy).clamp(0, (max_y - 1) as isize) as Coordinate;
                if dx == 0 && dy == 0 {
                    tiles[ny][nx] = Tile::Floor {
                        visible: false,
                        cursed: false,
                    };
                } else if tiles[ny][nx] == Tile::Empty {
                    tiles[ny][nx] = Tile::Wall { visible: false };
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn print(&self, with_border: bool) {
        println!("{}x{}", self.width, self.height);
        if with_border {
            println!("{}", "-".repeat(self.width));
        }
        let tiles = self.tiles.read().unwrap();
        println!(
            "{}",
            tiles
                .iter()
                .map(|row| {
                    let row_str: String = row.iter().map(|tile| tile.as_char()).collect();
                    if with_border {
                        format!("|{}|", row_str)
                    } else {
                        row_str
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        );
        if with_border {
            println!("{}", "-".repeat(self.width));
        }
    }
}
