use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{assets::Assets, utils::*};

mod assets;
mod utils;

struct Game<'a> {
    assets: &'a Assets,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        Self { assets }
    }
    fn update(&mut self) {
        clear_background(Color::from_hex(0x3c9f9c));
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);

        let t = &self.assets.cowboy.animations[0].get_at_time(0);
        draw_texture_ex(
            t,
            64.0,
            64.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(t.size() * scale_factor),
                ..Default::default()
            },
        );
    }
}

#[macroquad::main("cowboy tower")]
async fn main() {
    let assets = Assets::load();
    let mut game = Game::new(&assets);

    loop {
        game.update();
        next_frame().await;
    }
}
