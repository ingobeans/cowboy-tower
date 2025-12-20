use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{assets::Assets, utils::*};

mod assets;
mod player;
mod utils;

struct Game<'a> {
    assets: &'a Assets,
    camera: Camera2D,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        Self {
            assets,
            camera: Camera2D::default(),
        }
    }
    fn update(&mut self) {
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        self.camera.target += delta_time * 8.0 * get_input_axis() * 5.0;
        self.camera.zoom = vec2(
            1.0 / actual_screen_width * 2.0 * scale_factor,
            1.0 / actual_screen_height * 2.0 * scale_factor,
        );
        set_camera(&self.camera);
        clear_background(Color::from_hex(0x3c9f9c));

        let t = &self.assets.levels[0]
            .camera
            .render_target
            .as_ref()
            .unwrap()
            .texture;
        draw_texture_ex(
            t,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                //dest_size: Some(t.size() * scale_factor),
                ..Default::default()
            },
        );
        let t = &self.assets.cowboy.animations[0].get_at_time(0);
        draw_texture_ex(
            t,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                //dest_size: Some(t.size() * scale_factor),
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
