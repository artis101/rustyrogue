use crate::game::Game;
use crate::sprite_sheet::SpriteSheet;
use crate::tile::Tile;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::FullscreenType;
use std::io;

pub struct SDL {
    context: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    sprite_sheet: SpriteSheet,
}

impl SDL {
    pub fn new() -> Result<Self, io::Error> {
        let context = sdl2::init().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let video_subsystem = context
            .video()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let window = video_subsystem
            .window("Rustyrogue", 800, 600)
            .position_centered()
            .resizable()
            .maximized()
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let canvas = window
            .into_canvas()
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let texture_creator = canvas.texture_creator();
        let sprite_sheet =
            SpriteSheet::new(&texture_creator, "assets/colored_tilemap.png", 8, 1, 16)?;

        Ok(SDL {
            context,
            canvas,
            sprite_sheet,
        })
    }

    pub fn run(&mut self, game: &mut Game) -> Result<(), io::Error> {
        let mut event_pump = self
            .context
            .event_pump()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => game.move_player(-1, 0),
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => game.move_player(1, 0),
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => game.move_player(0, -1),
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => game.move_player(0, 1),
                    Event::KeyDown {
                        keycode: Some(Keycode::F),
                        ..
                    } => {
                        self.toggle_fullscreen()?;
                    }
                    _ => {}
                }
            }

            self.draw(game)?;
        }

        Ok(())
    }

    fn draw(&mut self, game: &Game) -> Result<(), String> {
        self.canvas.clear();

        let (width, height) = self.canvas.output_size()?;
        let map = game.get_map();
        let map_width = map[0].len();
        let map_height = map.len();

        let tile_width = width / map_width as u32;
        let tile_height = height / map_height as u32;
        let tile_size = tile_width.min(tile_height);

        let offset_x = (width - tile_size * map_width as u32) / 2;
        let offset_y = (height - tile_size * map_height as u32) / 2;

        for (y, row) in map.iter().enumerate() {
            for (x, &tile) in row.iter().enumerate() {
                let dest_rect = Rect::new(
                    (x as u32 * tile_size + offset_x) as i32,
                    (y as u32 * tile_size + offset_y) as i32,
                    tile_size,
                    tile_size,
                );
                let src_rect = self.sprite_sheet.get_tile_rect(tile.sprite_index());
                self.canvas
                    .copy(&self.sprite_sheet.texture, src_rect, dest_rect)?;
            }
        }

        self.canvas.present();
        Ok(())
    }

    fn toggle_fullscreen(&mut self) -> Result<(), io::Error> {
        let window = self.canvas.window_mut();
        if window.fullscreen_state() == FullscreenType::Desktop {
            window.set_fullscreen(FullscreenType::Off)
        } else {
            window.set_fullscreen(FullscreenType::Desktop)
        }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
