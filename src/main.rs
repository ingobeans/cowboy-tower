use std::{env::args, f32::consts::PI};

use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{
    assets::{Assets, AttackType, EnemyType, Horse, MovementType},
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
    fn is_ray(&self) -> bool {
        match self.type_index {
            4 => true,
            _ => false,
        }
    }
    fn shoot_offset(type_index: usize) -> bool {
        match type_index {
            3 | 4 => false,
            _ => true,
        }
    }
    fn base_speed(type_index: usize) -> f32 {
        match type_index {
            1 | 2 => 128.0 * 0.8,
            3 | 4 => 0.0,
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
            3 | 4 => false,
            _ => true,
        }
    }
    fn get_lifetime(&self) -> f32 {
        match &self.type_index {
            2 => 1.0,
            3 => 0.5,
            4 => 0.5,
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
    horses: Vec<Horse>,
    projectiles: Vec<Projectile>,
    level: usize,
    fade_timer: f32,
    level_complete: Option<f32>,
    time: f32,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets, level: usize) -> Self {
        Self {
            assets,
            level,
            player: Player::new(get_player_spawn(assets, level)),
            camera: Camera2D::default(),
            enemies: load_enemies(assets.levels[level].enemies.clone()),
            horses: assets.levels[level].horses.clone(),
            projectiles: Vec::new(),
            fade_timer: 0.0,
            level_complete: None,
            time: 0.0,
        }
    }
    fn load_level(&mut self, level: usize) {
        // Ensure consistent RNG whenever a level is loaded.
        // Otherwise, loading a level directly with command line arguments
        // would yield other RNG than playing through the game until the level
        rand::srand(level as u64);
        self.level = level;
        self.projectiles.clear();
        self.enemies = load_enemies(self.assets.levels[level].enemies.clone());
        self.horses = self.assets.levels[level].horses.clone();
        self.fade_timer = 0.5;
        self.player = Player::new(get_player_spawn(self.assets, level));
        self.player.facing_left = self.level % 2 != 0;
    }
    fn update(&mut self) {
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        self.time += delta_time;
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

        // update horses

        for horse in self.horses.iter_mut() {
            const HORSE_SPEED: f32 = 128.0;
            horse.time += delta_time;
            if horse.running {
                horse.returning_home = false;
                horse.velocity = horse
                    .velocity
                    .lerp(HORSE_SPEED * horse.direction, 1.0 * delta_time);
            } else if horse.returning_home {
                if horse.pos.distance(horse.home_pos) <= 1.0 {
                    horse.pos = horse.home_pos;
                    horse.returning_home = false;
                    horse.velocity = Vec2::ZERO;
                } else if !horse.player_riding {
                    horse.velocity = horse
                        .velocity
                        .lerp(HORSE_SPEED * -horse.direction, 1.0 * delta_time);
                }
            }
            let old_velocity = horse.velocity;
            (horse.pos, _, _) =
                update_physicsbody(horse.pos, &mut horse.velocity, delta_time, level, false);
            // if horse hits walls / stops, make horse.running = false
            if horse.running
                && (old_velocity.length() > horse.velocity.length()
                    || horse.velocity.length() == 0.0)
                && (old_velocity.normalize() - horse.direction.normalize()).length() < 0.1
            {
                // this if also
                // checks that the old velocity was actually in the same direction as the horse should be moving
                // otherwise, this is an edge case where the horse was previously moving in the wrong direction, and
                // is now accelerating in the correct direction, but this makes its total velocity decrease.
                horse.running = false;
            }
            if horse.running {
                if level.get_tile((horse.pos.x / 8.0) as i16, (horse.pos.y / 8.0) as i16)[3]
                    == 418 + 1
                {
                    horse.running = false;
                    horse.velocity = Vec2::ZERO;
                }
            }
            if !horse.returning_home
                && !horse.running
                && !horse.player_riding
                && horse.pos.distance(horse.home_pos) > 1.0
                && level.get_tile(
                    (self.player.pos.x / 8.0) as i16,
                    (self.player.pos.y / 8.0) as i16,
                )[3] != 419 + 1
            {
                horse.returning_home = true;
            }
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
                &mut self.horses,
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
        // draw animated tiles
        for (pos, index) in level.animated_tiles.iter() {
            let time = self.time + pos.x.powi(2) + pos.y.powi(2) * 100.0;
            draw_texture(
                self.assets.animated_tiles[*index].get_at_time((time * 1000.0) as u32),
                pos.x,
                pos.y,
                WHITE,
            );
        }
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
                        // values for this formula found with `find_lowest_drift_factor`
                        let value = value.sin()
                            * (value * 4.627175 + 1.5).sin()
                            * (value * 5.306475 + 8.0).sin().powi(2);
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
                    if self.player.death.is_none() {
                        enemy.attack_time += delta_time;
                        match enemy.ty.attack_type {
                            AttackType::None => {
                                enemy.attack_time = 0.0;
                            }
                            AttackType::ShootAfter(_) => {}
                            AttackType::Shoot(sprite) => {
                                let pos = if Projectile::shoot_offset(sprite) {
                                    enemy.pos
                                        + if enemy.pos.x > self.player.pos.x {
                                            vec2(-8.0, 0.0)
                                        } else {
                                            vec2(8.0, 0.0)
                                        }
                                        + vec2(4.0, 0.0)
                                } else {
                                    enemy.pos
                                };
                                self.projectiles.push(Projectile::new(
                                    sprite,
                                    pos,
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
                        let pos = if Projectile::shoot_offset(sprite) {
                            enemy.pos
                                + if enemy.pos.x > self.player.pos.x {
                                    vec2(-8.0, 0.0)
                                } else {
                                    vec2(8.0, 0.0)
                                }
                                + vec2(4.0, 0.0)
                        } else {
                            enemy.pos
                        };
                        self.projectiles.push(Projectile::new(
                            sprite,
                            pos,
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
        // draw horses
        for horse in self.horses.iter() {
            let flip = horse.is_flipped();
            let actual_flip = if horse.returning_home { !flip } else { flip };
            let normal = horse.get_normal();
            draw_texture_ex(
                self.assets.horse.animations[if horse.running || horse.returning_home {
                    2
                } else {
                    0
                } + if horse.player_riding { 1 } else { 0 }]
                .get_at_time((horse.time * 1000.0) as u32),
                horse.pos.x.floor() - 12.0 + normal.x * 12.0,
                horse.pos.y.floor() - 12.0 + normal.y * 12.0,
                WHITE,
                DrawTextureParams {
                    flip_x: actual_flip,
                    rotation: horse.direction.to_angle() - if flip { PI } else { 0.0 },
                    ..Default::default()
                },
            );

            /* debug: draw horse collision and horse normal
            draw_rectangle(horse.pos.x.floor(), horse.pos.y.floor(), 8.0, 8.0, RED);
            draw_line(
            horse.pos.x,
            horse.pos.y,
            horse.pos.x + normal.x * 16.0,
            horse.pos.y + normal.y * 16.0,
            1.0,
            YELLOW,
            );*/
        }
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
            let ray_direction = vec2(0.0, 1.0);
            let is_ray = projectile.is_ray();
            let section_count = if is_ray {
                let tx = (projectile.pos.x / 8.0) as i16;
                let ty = (projectile.pos.y / 8.0) as i16;
                let mut count = 0;
                loop {
                    if level.get_tile(tx, ty + count * ray_direction.y as i16)[1] != 0 {
                        break;
                    }
                    count += 1;
                }
                count - 1
            } else {
                1
            };
            for i in 0..section_count {
                draw_texture_ex(
                    &self.assets.projectiles.animations[projectile.type_index]
                        .get_at_time((projectile.time * 1000.0) as u32),
                    projectile.pos.x.floor() - 20.0 + ray_direction.x * 8.0 * i as f32,
                    projectile.pos.y.floor() - 20.0 + ray_direction.y * 8.0 * i as f32,
                    WHITE,
                    DrawTextureParams {
                        flip_x: projectile.direction.x < 0.0,
                        rotation,
                        ..Default::default()
                    },
                );
            }
            //draw_rectangle(projectile.pos.x, projectile.pos.y, 2.0,2.0, GREEN);
            if projectile.dead {
                return false;
            }
            if !projectile.friendly && projectile.can_kill() && self.player.death.is_none() {
                let mut player_hit = false;
                if is_ray {
                    if (projectile.pos.x..projectile.pos.x + 8.0)
                        .contains(&(self.player.pos.x + 4.0))
                        && (projectile.pos.y + 8.0
                            ..projectile.pos.y + 8.0 + 8.0 * section_count as f32)
                            .contains(&self.player.pos.y)
                    {
                        player_hit = true;
                    }
                } else if (self.player.pos + vec2(4.0, 4.0)).distance(projectile.pos)
                    < projectile.get_collision_size()
                {
                    player_hit = true;
                }
                if player_hit {
                    self.player.death = Some((0.0, 0));
                    projectile.dead |= projectile.should_die_on_kill();
                }
            }
            projectile.time += delta_time;
            let lifetime = projectile.get_lifetime();
            let died = lifetime != 0.0 && projectile.time >= lifetime;
            if died && let Some(payload) = projectile.get_payload() {
                new_projectiles.push(payload);
            }
            !died
                && if !physics_based && projectile.should_die_on_kill() {
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
        if let Some(death) = &self.player.death {
            let delta = death.0 - self.assets.die.animations[death.1].total_length as f32 / 1000.0;
            if delta > 0.0 {
                if delta > 0.5 {
                    self.load_level(self.level);
                }
                fade_amt = delta * 2.0;
            }
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

#[test]
fn find_lowest_drift_factor() {
    // This isnt exactly a "test" per se, but it was convenient enough to mark it as one so i can run with `cargo test -- --nocapture`

    // The actual movement of enemies with movement type wander, is generated with the following formula:
    //      sin(t)*sin(A * t + 1.5)*sin(B * t + 8.0)^2
    // because its sporatic enough. All values between -0.1 and 0.1 are clamped to 0, and
    // all other values are clamped to either -1 or 1, whichever is closest.
    // I want the integral of this function over one period to be as close to 0 as possible,
    // to ensure enemies dont drift towards one direction over time.
    // This function runs 1000 tests to find the values of A and B that yield the integral closest to 0.

    let step_size = 0.001;

    let tests = 1000;

    let mut best = (f32::MAX, (0.0, 0.0));

    for _ in 0..tests {
        let current_test = (rand::gen_range(1.0, 5.0), rand::gen_range(1.0, 10.0));

        let mut total = 0.0;
        let mut last = 0.0;
        for step in 0..(1.0 / step_size * PI) as u32 {
            let value = step as f32 * step_size;

            let delta_time = value - last;
            last = value;

            let value = value.sin()
                * (value * current_test.0 + 1.5).sin()
                * (value * current_test.1 + 8.0).sin().powi(2);
            let value = if value.abs() < 0.1 {
                0.0
            } else if value.is_sign_positive() {
                1.0
            } else {
                -1.0
            };

            total += value * delta_time;
        }
        // best is overriden if this total is closer to 0.
        // however, a restriction is also imposed that total must be less than 0,
        // since earlier maps were built with the knowledge in mind that enemies drift to the left over time.
        // so changing drift direction would be disruptive
        if total < 0.0 && total.abs() < best.0 {
            best = (total, current_test)
        }
    }
    dbg!(best);
}
