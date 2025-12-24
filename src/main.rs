use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{assets::Assets, player::Player, utils::*};

mod assets;
mod player;
mod utils;

struct Game<'a> {
    assets: &'a Assets,
    camera: Camera2D,
    player: Player,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        Self {
            assets,
            player: Player::new(vec2(0.0, -10.0 * 8.0)),
            camera: Camera2D::default(),
        }
    }
    fn update(&mut self) {
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        self.player.update(delta_time, &self.assets.levels[0]);
        self.camera.target = self.player.pos.floor() - vec2(0.0, 32.0);
        self.camera.zoom = vec2(
            1.0 / actual_screen_width * 2.0 * scale_factor,
            1.0 / actual_screen_height * 2.0 * scale_factor,
        );
        set_camera(&self.camera);
        clear_background(Color::from_hex(0x3c9f9c));

        let level = &self.assets.levels[0];
        let min_y = self.camera.target.y + actual_screen_height / scale_factor / 2.0;
        let min_y_tile = (min_y / 8.0).ceil();

        let max_y = self.camera.target.y - actual_screen_height / scale_factor / 2.0;
        let max_y_tile = (max_y / 8.0).floor();
        draw_rectangle(
            level.min_pos.x,
            min_y,
            level.max_pos.x - level.min_pos.x + 16.0 * 8.0,
            max_y - min_y,
            Color::from_hex(0x300f0a),
        );

        for y in max_y_tile as i16..min_y_tile as i16 {
            self.assets
                .tileset
                .draw_tile(level.min_pos.x - 8.0, (y * 8) as f32, 1.0, 3.0, None);
            self.assets.tileset.draw_tile(
                level.max_pos.x + 16.0 * 8.0,
                (y * 8) as f32,
                3.0,
                3.0,
                None,
            );
        }

        let t = &level.camera.render_target.as_ref().unwrap().texture;
        draw_texture(t, level.min_pos.x, level.min_pos.y, WHITE);
        for (pos, ty) in &level.enemies {
            draw_texture_ex(
                ty.animation.animations[0].get_at_time(0),
                pos.x.floor() - 4.0,
                pos.y.floor() - 8.0,
                WHITE,
                DrawTextureParams {
                    flip_x: true,
                    ..Default::default()
                },
            );
        }
        self.player.draw(self.assets);
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
