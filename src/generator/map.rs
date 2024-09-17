use crate::generator::room::Room;
use crate::map::types::{Coordinate, GameMapTiles, Point};
use crate::tile::Tile;
use rand::seq::SliceRandom;
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
    pub fn new(width: Coordinate, height: Coordinate) -> Self {
        MapGenerator {
            width,
            height,
            tiles: Arc::new(RwLock::new(vec![vec![Tile::Empty; width]; height])),
            rooms: Vec::new(),
            bsp_root: None,
        }
    }

    pub fn generate(&mut self, min_room_size: Coordinate) -> &mut Self {
        self.fill_with_empty();

        // Build the BSP tree
        self.build_bsp_tree(min_room_size);

        // Create rooms in the leaf nodes
        self.create_rooms_in_bsp();

        // Connect rooms via depth-first traversal
        if let Some(ref root) = self.bsp_root {
            self.connect_rooms_bsp(root);
        }

        self
    }

    pub fn get_dungeon(&self) -> Arc<RwLock<GameMapTiles>> {
        Arc::clone(&self.tiles)
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
        if let Some(ref mut root_node) = self.bsp_root {
            self.split_node(root_node, min_size);
        }
    }

    fn split_node(&mut self, node: &mut BSPNode, min_size: Coordinate) {
        if node.width > min_size * 2 || node.height > min_size * 2 {
            let split_vertically = if node.width > node.height {
                true
            } else if node.height > node.width {
                false
            } else {
                rand::random::<bool>()
            };

            if split_vertically && node.width > min_size * 2 {
                // Split vertically
                let split = rand::thread_rng().gen_range(min_size..(node.width - min_size));
                node.left = Some(Box::new(BSPNode::new(node.x, node.y, split, node.height)));
                node.right = Some(Box::new(BSPNode::new(
                    node.x + split,
                    node.y,
                    node.width - split,
                    node.height,
                )));
            } else if node.height > min_size * 2 {
                // Split horizontally
                let split = rand::thread_rng().gen_range(min_size..(node.height - min_size));
                node.left = Some(Box::new(BSPNode::new(node.x, node.y, node.width, split)));
                node.right = Some(Box::new(BSPNode::new(
                    node.x,
                    node.y + split,
                    node.width,
                    node.height - split,
                )));
            }

            if let Some(ref mut left) = node.left {
                self.split_node(left, min_size);
            }
            if let Some(ref mut right) = node.right {
                self.split_node(right, min_size);
            }
        }
    }

    fn create_rooms_in_bsp(&mut self) {
        let mut rooms = Vec::new();
        if let Some(ref mut root) = self.bsp_root {
            self.create_rooms_in_node(root, &mut rooms);
        }
        self.rooms = rooms;
    }

    fn create_rooms_in_node(&mut self, node: &mut BSPNode, rooms: &mut Vec<Room>) {
        if node.is_leaf() {
            // Create a room in this leaf node
            let mut rng = rand::thread_rng();
            let room_width = rng.gen_range((node.width / 2)..(node.width - 2));
            let room_height = rng.gen_range((node.height / 2)..(node.height - 2));
            let room_x = rng.gen_range((node.x + 1)..=(node.x + node.width - room_width - 1));
            let room_y = rng.gen_range((node.y + 1)..=(node.y + node.height - room_height - 1));
            let location = Point::new(room_x, room_y);

            let room = Room::new(location, room_width, room_height, Arc::clone(&self.tiles));
            room.populate();
            node.room = Some(room.clone());
            rooms.push(room);
        } else {
            if let Some(ref mut left) = node.left {
                self.create_rooms_in_node(left, rooms);
            }
            if let Some(ref mut right) = node.right {
                self.create_rooms_in_node(right, rooms);
            }
        }
    }

    fn connect_rooms_bsp(&self, node: &BSPNode) {
        if !node.is_leaf() {
            if let (Some(ref left), Some(ref right)) = (node.left, node.right) {
                self.connect_rooms_bsp(left);
                self.connect_rooms_bsp(right);

                let left_room = self.get_room_in_subtree(left);
                let right_room = self.get_room_in_subtree(right);
                if let (Some(lr), Some(rr)) = (left_room, right_room) {
                    self.connect_points(lr.center(), rr.center());
                }
            }
        }
    }

    fn get_room_in_subtree<'a>(&'a self, node: &'a BSPNode) -> Option<&'a Room> {
        if let Some(ref room) = node.room {
            Some(room)
        } else {
            node.left
                .as_ref()
                .and_then(|left| self.get_room_in_subtree(left))
                .or_else(|| {
                    node.right
                        .as_ref()
                        .and_then(|right| self.get_room_in_subtree(right))
                })
        }
    }

    fn connect_points(&self, start: Point, end: Point) {
        let mut rng = rand::thread_rng();
        let mut current = start;

        while current != end {
            self.carve_corridor(current.x, current.y);

            let dx = end.x as isize - current.x as isize;
            let dy = end.y as isize - current.y as isize;

            let mut directions = Vec::new();
            if dx != 0 {
                directions.push((dx.signum(), 0));
            }
            if dy != 0 {
                directions.push((0, dy.signum()));
            }
            // Random directions for drunkard's walk
            directions.push((-1, 0));
            directions.push((1, 0));
            directions.push((0, -1));
            directions.push((0, 1));

            let (dx, dy) = *directions.choose(&mut rng).unwrap();

            current.x = (current.x as isize + dx).clamp(0, (self.width - 1) as isize) as Coordinate;
            current.y =
                (current.y as isize + dy).clamp(0, (self.height - 1) as isize) as Coordinate;
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

    fn populate_all_rooms(&self) {
        self.rooms.par_iter().for_each(|room| {
            room.populate();
        });
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
