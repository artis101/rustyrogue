use crate::generator::room::Room;
use crate::map::types::{Coordinate, GameMapTiles, Point};
use crate::tile::Tile;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MapGenerator {
    width: Coordinate,
    height: Coordinate,
    tiles: Rc<RefCell<GameMapTiles>>,
    rooms: Vec<Room>,
}

impl MapGenerator {
    pub fn new(width: Coordinate, height: Coordinate) -> Self {
        MapGenerator {
            width,
            height,
            tiles: Rc::new(RefCell::new(vec![vec![Tile::Empty; width]; height])),
            rooms: vec![],
        }
    }

    pub fn generate(&mut self, num_rooms: usize) -> &mut Self {
        self.fill_with_empty();

        let mut rng = rand::thread_rng();

        for _ in 0..num_rooms {
            let mut attempts = 0;
            while attempts < 100 {
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..self.height);
                let width = rng.gen_range(10..35).min(self.width - x);
                let height = rng.gen_range(5..15).min(self.height - y);

                if self.can_add_room(Point::new(x, y), width, height) {
                    self.add_room(Point::new(x, y), width, height);
                    break;
                }
                attempts += 1;
            }
        }

        self.populate_all_rooms();
        self
    }

    fn add_room(&mut self, location: Point, width: Coordinate, height: Coordinate) {
        let room = Room::new(location, width, height, Rc::clone(&self.tiles));
        self.rooms.push(room);
    }

    fn can_add_room(&self, location: Point, width: Coordinate, height: Coordinate) -> bool {
        // check if room is within bounds
        if location.x + width >= self.width || location.y + height >= self.height {
            return false;
        }

        // check if room intersects with other rooms
        for room in &self.rooms {
            if room.intersects(location, width, height) {
                return false;
            }
        }

        // room can be added
        true
    }

    fn fill_with_empty(&self) {
        let mut tiles = self.tiles.borrow_mut();
        for y in 0..self.height {
            for x in 0..self.width {
                tiles[y][x] = Tile::Empty;
            }
        }
    }

    fn populate_all_rooms(&self) {
        for room in &self.rooms {
            room.populate();
        }
    }

    pub fn print(&self, with_border: bool) {
        println!("{}x{}", self.width, self.height);
        if with_border {
            println!("{}", "-".repeat(self.width));
        }
        println!(
            "{}",
            self.tiles
                .borrow()
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
