use std::{env::args, f32::consts::PI};

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
    /// Used for attack type ShootAfter
    has_attacked: bool,
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
            has_attacked: false,
            death_frames: 0.0,
            attack_time: 0.0,
            wibble_wobble: rand::gen_range(0.0, PI * 2.0),
        })
        .collect()
}

struct Projectile {
    pos: Vec2,
    direction: Vec2,
    type_index: usize,
    time: f32,
    /// Is projectile fired by the player?
    friendly: bool,
    /// True when projectile hits an enemy, marker to show that it should be destroyed.
    dead: bool,
}
impl Projectile {
    fn new(type_index: usize, pos: Vec2, direction: Vec2) -> Self {
        Self {
            pos,
            direction: direction * Self::base_speed(type_index),
            type_index,
            time: 0.0,
            friendly: type_index == 0,
            dead: false,
        }
    }
    fn base_speed(type_index: usize) -> f32 {
        match type_index {
            1 | 2 => 128.0 * 0.8,
            3 => 0.0,
            _ => 128.0,
        }
    }
    fn is_physics_based(&self) -> bool {
        match &self.type_index {
            2 => true,
            _ => false,
        }
    }
    fn get_payload(&self) -> Option<Projectile> {
        match &self.type_index {
            2 => Some(Projectile::new(3, self.pos, Vec2::ZERO)),
            _ => None,
        }
    }
    fn get_collision_size(&self) -> f32 {
        match &self.type_index {
            3 => 17.0,
            _ => 8.0,
        }
    }
    fn can_kill(&self) -> bool {
        match &self.type_index {
            2 => false,
            _ => true,
        }
    }
    fn should_die_on_kill(&self) -> bool {
        match &self.type_index {
            3 => false,
            _ => true,
        }
    }
    fn get_lifetime(&self) -> f32 {
        match &self.type_index {
            2 => 1.0,
            3 => 0.5,
            _ => 0.0,
        }
    }
}

fn get_player_spawn(assets: &Assets, level: usize) -> Vec2 {
    let left_level_end = level % 2 != 0;
    let player_spawn = if left_level_end {
        vec2(
            assets.levels[level].max_pos.x + 16.0 * 8.0 - 3.0 * 8.0,
            assets.levels[level].player_spawn.y,
        )
    } else {
        assets.levels[level].player_spawn + vec2(16.0, 0.0)
    };
    player_spawn
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
    fn new(assets: &'a Assets, level: usize) -> Self {
        Self {
            assets,
            level,
            player: Player::new(get_player_spawn(assets, level)),
            camera: Camera2D::default(),
            enemies: load_enemies(assets.levels[level].enemies.clone()),
            projectiles: Vec::new(),
            fade_timer: 0.0,
            level_complete: None,
        }
    }
    fn load_level(&mut self, level: usize) {
        self.level = level;
        self.projectiles.clear();
        self.enemies = load_enemies(self.assets.levels[level].enemies.clone());
        self.fade_timer = 0.5;
        self.player = Player::new(get_player_spawn(self.assets, level));
        self.player.facing_left = self.level % 2 != 0;
    }
    fn update(&mut self) {
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);

        let level = &self.assets.levels[self.level];

        let left_level_end = self.level % 2 != 0;

        let elevator_texture = self.assets.elevator.animations[0].get_at_time(0);
        let elevator_doors_animation = &self.assets.elevator.animations[1];
        let elevator_pos = vec2(
            if left_level_end {
                level.player_spawn.x
            } else {
                level.max_pos.x + 16.0 * 8.0 - elevator_texture.width()
            },
            level.player_spawn.y - elevator_texture.height() + 8.0,
        );

        if self.level_complete.is_none()
            && (self.player.pos.x - (elevator_pos.x + elevator_texture.width() / 2.0)).abs() <= 6.0
        {
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
                if *time * 1000.0 >= elevator_doors_animation.total_length as f32 {
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
                        match enemy.ty.attack_type {
                            AttackType::None => {
                                enemy.attack_time = 0.0;
                            }
                            AttackType::ShootAfter(_) => {}
                            AttackType::Shoot(sprite) => {
                                self.projectiles.push(Projectile::new(
                                    sprite,
                                    enemy.pos
                                        + if enemy.pos.x > self.player.pos.x {
                                            vec2(-8.0, 0.0)
                                        } else {
                                            vec2(8.0, 0.0)
                                        }
                                        + vec2(4.0, 0.0),
                                    vec2(
                                        if enemy.pos.x > self.player.pos.x {
                                            -1.0
                                        } else {
                                            1.0
                                        },
                                        0.0,
                                    ),
                                ));
                            }
                        }
                    }
                } else {
                    enemy.attack_time += delta_time;
                    let delta = enemy.attack_time * 1000.0
                        - enemy.ty.animation.get_by_name("attack").total_length as f32;
                    if delta >= 0.0
                        && !enemy.has_attacked
                        && let AttackType::ShootAfter(sprite) = enemy.ty.attack_type
                    {
                        self.projectiles.push(Projectile::new(
                            sprite,
                            enemy.pos
                                + if enemy.pos.x > self.player.pos.x {
                                    vec2(-8.0, 0.0)
                                } else {
                                    vec2(8.0, 0.0)
                                }
                                + vec2(4.0, 0.0),
                            vec2(
                                if enemy.pos.x > self.player.pos.x {
                                    -1.0
                                } else {
                                    1.0
                                },
                                0.0,
                            ),
                        ));
                        enemy.has_attacked = true;
                    }
                    if delta >= enemy.ty.attack_delay * 1000.0 {
                        enemy.attack_time = 0.0;
                        enemy.has_attacked = false;
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
                        && projectile.can_kill()
                        && ((projectile.pos.x - 4.0)..(projectile.pos.x + 4.0))
                            .contains(&(enemy.pos.x + 4.0))
                        && ((projectile.pos.y - 4.0)..(projectile.pos.y + 4.0))
                            .contains(&enemy.pos.y)
                    {
                        projectile.dead |= projectile.should_die_on_kill();
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
        // draw level beginning elevator
        if self.level > 0 {
            draw_texture(
                self.assets.elevator.animations[2]
                    .get_at_time(((0.5 - self.fade_timer) * 1000.0) as u32),
                get_player_spawn(self.assets, self.level).x
                    + if left_level_end {
                        3.0 * 8.0 - elevator_texture.width()
                    } else {
                        -2.0 * 8.0
                    },
                elevator_pos.y,
                WHITE,
            );
        }
        // draw level end elevator
        draw_texture(elevator_texture, elevator_pos.x, elevator_pos.y, WHITE);
        self.player.draw(self.assets);
        if let Some(time) = &self.level_complete {
            // draw level end elevator door animation if level complete
            let texture = elevator_doors_animation.get_at_time((*time * 1000.0) as u32);
            draw_texture(texture, elevator_pos.x, elevator_pos.y, WHITE);
        }
        let mut new_projectiles = Vec::new();
        self.projectiles.retain_mut(|projectile| {
            let physics_based = projectile.is_physics_based();

            if physics_based {
                const OFFSET: Vec2 = vec2(4.0, 4.0);
                projectile.direction.y += GRAVITY * delta_time;

                let (new_pos, on_ground, _) = update_physicsbody(
                    projectile.pos - OFFSET,
                    &mut projectile.direction,
                    delta_time,
                    level,
                    false,
                );
                projectile.pos = new_pos + OFFSET;
                if on_ground {
                    projectile.direction.x = projectile.direction.x.lerp(0.0, delta_time * 2.0);
                }
            } else {
                projectile.pos += projectile.direction * delta_time;
            }
            let rotation = if !physics_based {
                0.0
            } else {
                projectile.time * 10.0
            };
            draw_texture_ex(
                &self.assets.projectiles.animations[projectile.type_index]
                    .get_at_time((projectile.time * 1000.0) as u32),
                projectile.pos.x.floor() - 20.0,
                projectile.pos.y.floor() - 20.0,
                WHITE,
                DrawTextureParams {
                    flip_x: projectile.direction.x < 0.0,
                    rotation,
                    ..Default::default()
                },
            );
            if projectile.dead {
                return false;
            }
            if !projectile.friendly
                && projectile.can_kill()
                && self.player.death_frames <= 0.0
                && (self.player.pos + vec2(4.0, 4.0)).distance(projectile.pos)
                    < projectile.get_collision_size()
            {
                self.player.death_frames += delta_time;
                projectile.dead |= projectile.should_die_on_kill();
            }
            projectile.time += delta_time;
            let lifetime = projectile.get_lifetime();
            let died = lifetime != 0.0 && projectile.time >= lifetime;
            if died && let Some(payload) = projectile.get_payload() {
                new_projectiles.push(payload);
            }
            !died
                && if !physics_based {
                    let tx = (projectile.pos.x / 8.0) as i16;
                    let ty = (projectile.pos.y / 8.0) as i16;
                    let hit_wall = level.get_tile(tx, ty)[1] != 0;

                    !hit_wall
                } else {
                    true
                }
        });
        self.projectiles.append(&mut new_projectiles);
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
    let mut level = 0;
    for arg in args() {
        if let Ok(index) = arg.parse::<usize>() {
            level = index;
        }
    }
    let mut game = Game::new(&assets, level);

    loop {
        game.update();
        next_frame().await;
    }
}
