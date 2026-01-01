use std::f32::consts::PI;

use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{
    assets::{Assets, AttackType, EnemyType, MovementType},
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
    attack_time: f32,
    /// Set to zero when alive. On death, tracks death animation time
    death_frames: f32,
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
            death_frames: 0.0,
            attack_time: 0.0,
            wibble_wobble: rand::gen_range(0.0, PI * 2.0),
        })
        .collect()
}

struct Projectile {
    pos: Vec2,
    direction: Vec2,
    sprite: usize,
    /// Is projectile fired by the player?
    friendly: bool,
    /// True when projectile hits an enemy, marker to show that it should be destroyed.
    dead: bool,
}

struct Game<'a> {
    assets: &'a Assets,
    camera: Camera2D,
    player: Player,
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    level: usize,
    fade_timer: f32,
    level_complete: Option<f32>,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        Self {
            assets,
            player: Player::new(assets.levels[0].player_spawn),
            camera: Camera2D::default(),
            enemies: load_enemies(assets.levels[0].enemies.clone()),
            projectiles: Vec::new(),
            level: 0,
            fade_timer: 0.0,
            level_complete: None,
        }
    }
    fn load_level(&mut self, level: usize) {
        self.level = level;
        self.projectiles.clear();
        self.enemies = load_enemies(self.assets.levels[level].enemies.clone());
        self.player = Player::new(self.assets.levels[level].player_spawn);
        self.fade_timer = 0.5;
    }
    fn update(&mut self) {
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);

        let level = &self.assets.levels[self.level];
        let elevator_texture = self.assets.elevator.animations[0].get_at_time(0);
        let elevator_doors_animation = &self.assets.elevator.animations[1];
        let elevator_pos = vec2(
            level.max_pos.x + 16.0 * 8.0 - elevator_texture.width(),
            level.player_spawn.y - elevator_texture.height() + 8.0,
        );

        if self.player.pos.x > elevator_pos.x + 12.0 && self.level_complete.is_none() {
            self.level_complete = Some(0.0);
        }
        if let Some(time) = &mut self.level_complete {
            if *time == 0.0 {
                self.player.time += delta_time;
                self.player.moving = true;
                const ELEVATOR_OFFSET: Vec2 = vec2(14.0, 48.0);
                const MOVE_SPEED: f32 = 16.0;
                self.player.pos = self
                    .player
                    .pos
                    .move_towards(elevator_pos + ELEVATOR_OFFSET, delta_time * MOVE_SPEED);
                if self.player.pos.distance(elevator_pos + ELEVATOR_OFFSET) <= 1.0 {
                    *time = delta_time;
                }
            } else {
                *time += delta_time;
                if *time * 1000.0 > elevator_doors_animation.total_length as f32 {
                    self.level_complete = None;
                    self.load_level(self.level + 1);
                }
            }
        } else {
            self.player.update(
                delta_time,
                &self.assets.levels[self.level],
                &mut self.projectiles,
            );
        }

        self.camera.target = self.player.camera_pos.floor();
        self.camera.zoom = vec2(
            1.0 / actual_screen_width * 2.0 * scale_factor,
            1.0 / actual_screen_height * 2.0 * scale_factor,
        );
        set_camera(&self.camera);
        clear_background(Color::from_hex(0x1CB7FF));

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

        draw_rectangle(
            elevator_pos.x,
            max_y,
            elevator_texture.width(),
            elevator_pos.y - max_y,
            Color::from_hex(0x3e2004),
        );
        self.enemies.retain_mut(|enemy| {
            enemy.time += delta_time;
            if enemy.death_frames > 0.0 {
                enemy.death_frames += delta_time;
                enemy.time = 0.0;
            } else {
                match enemy.ty.movement_type {
                    MovementType::None => {}
                    MovementType::Wander => {
                        let value = enemy.time + enemy.wibble_wobble;
                        let value = value.sin()
                            * (value * 3.0 + 1.5).sin()
                            * (value * 4.0 + 8.0).sin().powi(2);
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
                if enemy.attack_time <= 0.0 {
                    if self.player.death_frames <= 0.0 {
                        enemy.attack_time += delta_time;
                        match enemy.ty.attack_time {
                            AttackType::None => {
                                enemy.attack_time = 0.0;
                            }
                            AttackType::Shoot(sprite) => {
                                self.projectiles.push(Projectile {
                                    pos: enemy.pos
                                        + if enemy.pos.x > self.player.pos.x {
                                            vec2(-8.0, 0.0)
                                        } else {
                                            vec2(8.0, 0.0)
                                        }
                                        + vec2(4.0, 0.0),
                                    direction: vec2(
                                        if enemy.pos.x > self.player.pos.x {
                                            -0.8
                                        } else {
                                            0.8
                                        },
                                        0.0,
                                    ),
                                    sprite,
                                    friendly: false,
                                    dead: false,
                                });
                            }
                        }
                    }
                } else {
                    enemy.attack_time += delta_time;
                    if enemy.attack_time * 1000.0
                        > enemy.ty.animation.get_by_name("attack").total_length as f32
                            + enemy.ty.attack_delay * 1000.0
                    {
                        enemy.attack_time = 0.0;
                    }
                }
                (enemy.pos, _, _) =
                    update_physicsbody(enemy.pos, &mut enemy.velocity, delta_time, level, true);
            }
            let rotation = if enemy.death_frames <= 0.0 {
                0.0
            } else {
                (enemy.death_frames * 1000.0 * 2.0 / self.assets.blood.total_length as f32).min(1.0)
                    * (PI / 4.0)
                    * (if enemy.pos.x > self.player.pos.x {
                        1.0
                    } else {
                        -1.0
                    })
            };
            let (animation_id, time) = if enemy.attack_time > 0.0
                && enemy.attack_time * 1000.0
                    < enemy.ty.animation.get_by_name("attack").total_length as f32
            {
                (enemy.ty.animation.tag_names["attack"], enemy.attack_time)
            } else {
                (if enemy.velocity.x.abs() > 5.0 { 1 } else { 0 }, enemy.time)
            };
            draw_texture_ex(
                enemy.ty.animation.animations[animation_id].get_at_time((time * 1000.0) as u32),
                enemy.pos.x.floor() - 8.0,
                enemy.pos.y.floor() - 8.0,
                WHITE,
                DrawTextureParams {
                    flip_x: enemy.pos.x > self.player.pos.x,
                    rotation,
                    ..Default::default()
                },
            );
            if enemy.death_frames <= 0.0 {
                let mut hit_by_projectile = false;
                for projectile in self.projectiles.iter_mut() {
                    if projectile.friendly
                        && ((projectile.pos.x - 4.0)..(projectile.pos.x + 4.0))
                            .contains(&(enemy.pos.x + 4.0))
                        && ((projectile.pos.y - 4.0)..(projectile.pos.y + 4.0))
                            .contains(&enemy.pos.y)
                    {
                        projectile.dead = true;
                        hit_by_projectile = true;
                        break;
                    }
                }
                if hit_by_projectile {
                    enemy.death_frames += delta_time;
                }
                true
            } else {
                draw_texture_ex(
                    self.assets
                        .blood
                        .get_at_time((enemy.death_frames * 1000.0) as u32),
                    enemy.pos.x.floor() - 4.0,
                    enemy.pos.y.floor() - 8.0,
                    WHITE,
                    DrawTextureParams {
                        flip_x: enemy.pos.x > self.player.pos.x,
                        ..Default::default()
                    },
                );
                enemy.death_frames * 1000.0 <= self.assets.blood.total_length as f32
            }
        });
        draw_texture(elevator_texture, elevator_pos.x, elevator_pos.y, WHITE);
        self.player.draw(self.assets);
        if let Some(time) = &self.level_complete {
            let texture = elevator_doors_animation.get_at_time((*time * 1000.0) as u32);
            draw_texture(texture, elevator_pos.x, elevator_pos.y, WHITE);
        }
        self.projectiles.retain_mut(|projectile| {
            projectile.pos += projectile.direction * delta_time * 128.0;
            draw_texture_ex(
                &self.assets.projectiles.frames[projectile.sprite].0,
                projectile.pos.x.floor() - 4.0,
                projectile.pos.y.floor() - 4.0,
                WHITE,
                DrawTextureParams {
                    flip_x: projectile.direction.x < 0.0,
                    ..Default::default()
                },
            );
            if projectile.dead {
                return false;
            }
            if !projectile.friendly
                && self.player.death_frames <= 0.0
                && ((projectile.pos.x - 4.0)..(projectile.pos.x + 4.0))
                    .contains(&(self.player.pos.x + 4.0))
                && ((projectile.pos.y - 4.0)..(projectile.pos.y + 4.0)).contains(&self.player.pos.y)
            {
                self.player.death_frames += delta_time;
                projectile.dead = true;
            }
            let tx = (projectile.pos.x / 8.0) as i16;
            let ty = (projectile.pos.y / 8.0) as i16;
            let hit_wall = level.get_tile(tx, ty)[1] != 0;

            !hit_wall
        });
        if self.fade_timer > 0.0 {
            self.fade_timer -= delta_time;
        }
        let mut fade_amt = self.fade_timer * 2.0;
        let delta = self.player.death_frames - self.assets.die.total_length as f32 / 1000.0;
        if delta > 0.0 {
            if delta > 0.5 {
                self.load_level(self.level);
            }
            fade_amt = delta * 2.0;
        }
        if fade_amt > 0.0 {
            draw_rectangle(
                self.camera.target.x - actual_screen_width / scale_factor / 2.0,
                self.camera.target.y - actual_screen_height / scale_factor / 2.0,
                actual_screen_width,
                actual_screen_height,
                BLACK.with_alpha(fade_amt),
            );
        }
    }
}

#[macroquad::main("cowboy tower")]
async fn main() {
    //miniquad::window::set_window_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    let assets = Assets::load();
    let mut game = Game::new(&assets);

    loop {
        game.update();
        next_frame().await;
    }
}
