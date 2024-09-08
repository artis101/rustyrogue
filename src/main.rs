mod game;
mod map;
mod sdl;
mod tile;
mod tui;

use game::Game;
use sdl::SDL;
use std::env;
use std::io;
use tui::TUI;

fn main() -> Result<(), io::Error> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let use_sdl = args.contains(&"--sdl".to_string());

    // Create game instance
    let mut game = Game::new("example_map.txt")?;

    if use_sdl {
        // Run the game with SDL renderer
        let mut sdl = SDL::new()?;
        sdl.run(&mut game)?;
    } else {
        // Run the game with TUI renderer (default)
        let mut tui = TUI::new()?;
        tui.run(&mut game)?;
    }

    Ok(())
}
