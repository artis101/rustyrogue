mod game;
mod map;
mod player;
mod sdl;
mod sprite_sheet;
mod tile;
mod tui;
mod widgets;

use game::Game;
use sdl::SDL;
use std::env;
use std::io;
use tui::Tui;

fn main() -> Result<(), io::Error> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let use_sdl = args.contains(&"--sdl".to_string());

    // Create game instance
    let mut game = Game::new("tutorial")?;

    if use_sdl {
        // Run the game with SDL renderer
        let mut sdl = SDL::new()?;
        sdl.run(&mut game)?;
    } else {
        // Run the game with Tui renderer (default)
        let mut tui = Tui::new()?;
        tui.run(&mut game)?;
    }

    Ok(())
}
