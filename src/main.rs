use std::f32::consts::PI;

use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{
    assets::{Assets, EnemyType, MovementType},
    player::{Player, update_physicsbody},
    utils::*,
};

mod assets;
mod player;
mod utils;

struct Enemy {
    pos: Vec2,
    velocity: Vec2,
    ty: &'static EnemyType,
    time: f32,
    /// Random seed for each enemy, used for random-esque movement and behaviour
    wibble_wobble: f32,
}

fn load_enemies(input: Vec<(Vec2, &'static EnemyType)>) -> Vec<Enemy> {
    input
        .into_iter()
        .map(|f| Enemy {
            pos: f.0,
            velocity: Vec2::ZERO,
            ty: f.1,
            time: 0.0,
            wibble_wobble: rand::gen_range(0.0, PI * 2.0),
        })
        .collect()
}

struct Game<'a> {
    assets: &'a Assets,
    camera: Camera2D,
    player: Player,
    enemies: Vec<Enemy>,
    projectiles: Vec<(Vec2, usize)>,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        Self {
            assets,
            player: Player::new(vec2(0.0, -10.0 * 8.0)),
            camera: Camera2D::default(),
            enemies: load_enemies(assets.levels[0].enemies.clone()),
            projectiles: Vec::new(),
        }
    }
    fn update(&mut self) {
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        self.player.update(delta_time, &self.assets.levels[0]);
        self.camera.target = self.player.camera_pos.floor();
        self.camera.zoom = vec2(
            1.0 / actual_screen_width * 2.0 * scale_factor,
            1.0 / actual_screen_height * 2.0 * scale_factor,
        );
        set_camera(&self.camera);
        clear_background(Color::from_hex(0x1CB7FF));

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
        self.enemies.retain_mut(|enemy| {
            enemy.time += delta_time;
            match enemy.ty.movement_type {
                MovementType::None => {}
                MovementType::Wander => {
                    let value = enemy.time + enemy.wibble_wobble;
                    let value =
                        value.sin() * (value * 3.0 + 1.5).sin() * (value * 4.0 + 8.0).sin().powi(2);
                    let value = if value.abs() < 0.1 {
                        0.0
                    } else if value.is_sign_positive() {
                        1.0
                    } else {
                        -1.0
                    };
                    enemy.velocity.x = value * 16.0;
                }
            }
            (enemy.pos, _) =
                update_physicsbody(enemy.pos, &mut enemy.velocity, delta_time, &level, true);
            draw_texture_ex(
                enemy.ty.animation.animations[0].get_at_time(0),
                enemy.pos.x.floor() - 4.0,
                enemy.pos.y.floor() - 8.0,
                WHITE,
                DrawTextureParams {
                    flip_x: enemy.pos.x > self.player.pos.x,
                    ..Default::default()
                },
            );
            true
        });
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
