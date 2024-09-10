use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

pub struct SpriteSheet<'a> {
    pub texture: Texture<'a>,
    tile_size: u32,
    spacing: u32,
    columns: u32,
}

impl SpriteSheet {
    pub fn new(
        texture_creator: &TextureCreator<WindowContext>,
        path: &str,
        tile_size: u32,
        spacing: u32,
        columns: u32,
    ) -> Result<Self, String> {
        let texture = texture_creator.load_texture(path)?;
        Ok(SpriteSheet {
            texture,
            tile_size,
            spacing,
            columns,
        })
    }

    pub fn get_tile_rect(&self, index: usize) -> Rect {
        let row = (index as u32 / self.columns) as i32;
        let col = (index as u32 % self.columns) as i32;
        Rect::new(
            col * (self.tile_size as i32 + self.spacing as i32),
            row * (self.tile_size as i32 + self.spacing as i32),
            self.tile_size,
            self.tile_size,
        )
    }
}
