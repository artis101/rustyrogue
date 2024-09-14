mod game;
mod generator;
mod map;
mod player;
mod sdl;
mod tile;
mod tui;

use game::Game;
use generator::map::MapGenerator;
use sdl::SDL;
use std::env;
use std::io;
use tui::Tui;

fn main() -> Result<(), io::Error> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let use_sdl = args.contains(&"--sdl".to_string());
    let use_generator = args.contains(&"--generate".to_string());

    // Create game instance
    let mut game = Game::new("tutorial")?;

    if use_sdl {
        // Run the game with SDL renderer
        let mut sdl = SDL::new()?;
        sdl.run(&mut game)?;
    } else if use_generator {
        let mut map_generator = MapGenerator::new(163, 71);
        map_generator.generate(5); // Generate 5 rooms
        map_generator.print(true);
    } else {
        // Run the game with Tui renderer (default)
        let mut tui = Tui::new()?;
        tui.run(&mut game)?;
    }

    Ok(())
}
