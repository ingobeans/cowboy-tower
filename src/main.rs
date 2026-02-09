use std::{env::args, f32::consts::PI};

use gamepads::Gamepads;
use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{
    assets::{Assets, Horse, Level},
    bosses::{Boss, new_boss},
    enemies::*,
    player::{CinematicBars, Player, update_physicsbody},
    projectiles::*,
    tower::*,
    ui::draw_boss_badges,
    utils::*,
};

mod assets;
mod bosses;
mod enemies;
mod player;
mod projectiles;
mod tower;
mod ui;
mod utils;

fn load_enemies(input: Vec<LevelEnemyData>) -> Vec<Enemy> {
    input
        .into_iter()
        .map(|f| Enemy {
            pos: f.pos,
            velocity: Vec2::ZERO,
            ty: f.ty,
            time: 0.0,
            path_index: f.path_index,
            has_attacked: false,
            spawner: f.spawner,
            death_frames: 0.0,
            attack_time: -f.attack_delay,
            wibble_wobble: rand::gen_range(0.0, PI * 2.0),
        })
        .collect()
}

fn get_elevator_pos(assets: &Assets, level_index: usize) -> Vec2 {
    let level = &assets.levels[level_index];
    let elevator_texture = assets.elevator.animations[0].get_at_time(0);
    if let Some(pos) = level.forced_level_end {
        return vec2(
            if !level_index.is_multiple_of(2) {
                pos.x
            } else {
                pos.x - elevator_texture.width() + 8.0
            },
            pos.y - elevator_texture.height() + 8.0,
        );
    }
    vec2(
        if !level_index.is_multiple_of(2) {
            level.player_spawn.x
        } else {
            level.max_pos.x + 16.0 * 8.0 - elevator_texture.width()
        },
        level.player_spawn.y - elevator_texture.height() + 8.0,
    )
}

fn get_player_spawn(assets: &Assets, level_index: usize) -> Vec2 {
    let level = &assets.levels[level_index];
    if let Some(pos) = level.forced_player_spawn {
        return pos;
    }
    let left_level_end = !level_index.is_multiple_of(2);

    if left_level_end {
        vec2(
            level.max_pos.x + 16.0 * 8.0 - 3.0 * 8.0,
            level.player_spawn.y,
        )
    } else {
        level.player_spawn + vec2(16.0, 0.0)
    }
}

fn load_boss(level: &Level) -> Option<Box<dyn Boss>> {
    level.boss.map(|(i, p)| new_boss(i, p))
}

fn load_fog_points(level: &Level) -> Vec<FogPoint> {
    let density = 0.01;
    let points_amt = level.fog_points.len();
    let amt = (points_amt as f32 * 8.0 * 8.0 * density) as u32;
    let mut points = Vec::new();
    for _ in 0..amt {
        let index = rand::gen_range(0, points_amt);
        let localized_offset = vec2(rand::gen_range(0.0, 8.0), rand::gen_range(0.0, 8.0));
        let pos = level.fog_points[index] + localized_offset;
        points.push(FogPoint { pos });
    }
    points
}

struct FogPoint {
    pos: Vec2,
}

struct Game<'a> {
    assets: &'a Assets,
    camera: Camera2D,
    player: Player,
    enemies: Vec<Enemy>,
    horses: Vec<Horse>,
    projectiles: Vec<Projectile>,
    boss: Option<Box<dyn Boss>>,
    level: usize,
    fade_timer: f32,
    level_complete: Option<f32>,
    time: f32,
    level_transition_time: f32,
    height: f32,
    world_manager: WorldManager,
    gamepad_engine: Gamepads,
    fog_points: Vec<FogPoint>,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets, level: usize) -> Self {
        let mut y = 0.0;
        for l in &assets.levels[..level] {
            y += l.get_height() + FLOOR_PADDING + 16.0;
        }
        let world_manager = WorldManager::new(assets);
        let tower_height = world_manager.world_heights.last().unwrap().1;
        SKY_MATERIAL.set_uniform("maxTowerHeight", tower_height);

        Self {
            assets,
            level,
            height: y,
            world_manager,
            player: Player::new(get_player_spawn(assets, level)),
            camera: Camera2D::default(),
            enemies: load_enemies(assets.levels[level].enemies.clone()),
            boss: load_boss(&assets.levels[level]),
            fog_points: load_fog_points(&assets.levels[level]),
            horses: assets.levels[level].horses.clone(),
            gamepad_engine: Gamepads::new(),
            projectiles: Vec::new(),
            fade_timer: 0.0,
            level_complete: None,
            time: 0.0,
            level_transition_time: 0.0,
        }
    }
    fn load_level(&mut self, level: usize) {
        // Ensure consistent RNG whenever a level is loaded.
        // Otherwise, loading a level directly with command line arguments
        // would yield other RNG than playing through the game until the level
        rand::srand(level as u64);
        self.level = level;
        self.fog_points = load_fog_points(&self.assets.levels[level]);
        self.projectiles.clear();
        self.enemies = load_enemies(self.assets.levels[level].enemies.clone());
        self.boss = load_boss(&self.assets.levels[level]);
        self.horses = self.assets.levels[level].horses.clone();
        self.player = Player::new(get_player_spawn(self.assets, level));
        self.player.facing_left = !self.level.is_multiple_of(2);
    }
    fn update(&mut self) {
        self.gamepad_engine.poll();
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        self.time += delta_time;
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor = (actual_screen_width / SCREEN_WIDTH)
            .min(actual_screen_height / SCREEN_HEIGHT)
            .floor();

        let elevator_doors_animation = &self.assets.doors.animations[0];

        #[cfg(debug_assertions)]
        {
            if is_key_pressed(KeyCode::G) {
                self.level_complete = Some(delta_time);
            }
        }

        if let Some(time) = self.level_complete
            && time * 1000.0 >= elevator_doors_animation.total_length as f32
        {
            self.level_complete = None;
            self.height += self.assets.levels[self.level].get_height() + FLOOR_PADDING + 16.0;
            self.load_level(self.level + 1);
            self.level_transition_time = LEVEL_TRANSITION_LENGTH;
            self.player.update(
                delta_time,
                &self.assets.levels[self.level],
                &mut self.projectiles,
                &mut self.horses,
                &mut self.gamepad_engine,
            );
        }
        let level = &self.assets.levels[self.level];

        let left_level_end = !self.level.is_multiple_of(2);

        let elevator_texture =
            &self.assets.elevator.animations[level.get_world_index() as usize].frames[0].0;
        let elevator_pos = get_elevator_pos(self.assets, self.level);
        let player_spawn = get_player_spawn(self.assets, self.level);

        if self.level_complete.is_none()
            && (self.player.pos.x - (elevator_pos.x + elevator_texture.width() / 2.0)).abs() <= 6.0
        {
            self.level_complete = Some(0.0);
        }

        let player_tile = level.get_tile(
            (self.player.pos.x / 8.0).floor() as i16,
            (self.player.pos.y / 8.0).floor() as i16,
        )[3];

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
            (horse.pos, _, _, _) = update_physicsbody(
                horse.pos,
                &mut horse.velocity,
                delta_time,
                level,
                false,
                false,
            );
            // if horse hits walls / stops, make horse.running = false
            if horse.running
                && (old_velocity.length() > horse.velocity.length()
                    || horse.velocity.length() == 0.0)
                && (old_velocity.normalize() - horse.direction.normalize()).length() < 0.1
            {
                // this if also
                // checks that the old velocity was actually in the same direction as the horse should be moving
                // otherwise, there would be an edge case where the horse was previously moving in the wrong direction, and
                // is now accelerating in the correct direction, but this makes its total velocity decrease.

                horse.running = false;
            }
            if horse.running
                && level.get_tile((horse.pos.x / 8.0) as i16, (horse.pos.y / 8.0) as i16)[3]
                    == 418 + 1
            {
                horse.running = false;
                horse.velocity = Vec2::ZERO;
            }
            if !horse.returning_home
                && !horse.running
                && !horse.player_riding
                && horse.pos.distance(horse.home_pos) > 1.0
                && player_tile != 419 + 1
            {
                horse.returning_home = true;
            }
        }

        if player_tile == 705 + 1 {
            let pos = (self.player.pos / 8.0).floor() * 8.0;
            let camera_offset = level.camera_offsets.iter().find(|f| f.0 == pos).unwrap();
            self.player.camera_offset.set_target(camera_offset.1);
        } else if player_tile == 704 + 1 {
            self.player.camera_offset.set_target(0.0);
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
            }
        } else {
            self.player.update(
                delta_time,
                &self.assets.levels[self.level],
                &mut self.projectiles,
                &mut self.horses,
                &mut self.gamepad_engine,
            );
        }

        if self.level_transition_time > 0.0 {
            let old = &self.assets.levels[self.level - 1];
            let elevator_pos = get_elevator_pos(self.assets, self.level - 1);
            let y_diff = (old.min_pos.y - (level.max_pos.y + FLOOR_PADDING)).abs();
            self.level_transition_time -= delta_time;
            self.camera.target = self.player.camera_pos.floor();
            let x =
                (LEVEL_TRANSITION_LENGTH - self.level_transition_time) / LEVEL_TRANSITION_LENGTH;
            let amt = (1.0 - (x - 1.0).powi(2)).sqrt();
            self.camera.target.y = (elevator_pos.y + y_diff - 22.0 + elevator_texture.height()
                - 8.0)
                .lerp(self.player.camera_pos.y.floor(), amt);
        } else {
            self.camera.target = self.player.camera_pos.floor();
            self.camera.target.y -= self.player.camera_offset.current_offset * 8.0;
        }
        self.camera.zoom = vec2(
            1.0 / actual_screen_width * 2.0 * scale_factor,
            1.0 / actual_screen_height * 2.0 * scale_factor,
        );
        set_camera(&self.camera);
        let camera_offset = self.camera.target.y - (player_spawn.y - 22.0);

        SKY_MATERIAL.set_uniform(
            "y",
            -self.height + camera_offset + actual_screen_height / scale_factor / 2.0,
        );
        SKY_MATERIAL.set_uniform("height", actual_screen_height / scale_factor + 4.0);

        const SKY_COLOR: Color = Color::from_hex(0x1CB7FF);
        clear_background(SKY_COLOR);

        gl_use_material(&SKY_MATERIAL);
        let screen_offset = vec2(
            self.camera.target.x - actual_screen_width / scale_factor / 2.0,
            self.camera.target.y - actual_screen_height / scale_factor / 2.0,
        );

        draw_rectangle(
            screen_offset.x - 2.0,
            screen_offset.y - 2.0,
            actual_screen_width / scale_factor + 4.0,
            actual_screen_height / scale_factor + 4.0,
            SKY_COLOR,
        );
        gl_use_default_material();

        let ground_y = self.assets.levels[0].find_marker(2).y;
        draw_rectangle(
            screen_offset.x,
            ground_y - 1.0 + self.height,
            actual_screen_width / scale_factor,
            actual_screen_height / scale_factor,
            Color::from_hex(0xefb775),
        );
        self.world_manager
            .draw_tower(self.height, self.assets, self.level);

        if self.level > 0 {
            SKY_MATERIAL.set_uniform(
                "y",
                -self.height + camera_offset - level.floor_height + 56.0,
            );
            SKY_MATERIAL.set_uniform("height", level.get_height());
            gl_use_material(&SKY_MATERIAL);

            draw_rectangle(
                level.min_pos.x + 2.0,
                level.roof_height + 2.0,
                (level.min_pos.x - level.max_pos.x).abs() + 16.0 * 8.0 - 4.0,
                level.get_height() - 4.0,
                SKY_COLOR,
            );
            gl_use_default_material();
        }

        // draw level
        let t = &level.camera.render_target.as_ref().unwrap().texture;
        draw_texture(t, level.min_pos.x, level.min_pos.y, WHITE);

        if self.level_transition_time > 0.0 {
            // draw previous level if during transition
            let old = &self.assets.levels[self.level - 1];
            let t = &old.camera.render_target.as_ref().unwrap().texture;
            let x = if left_level_end {
                level.max_pos.x - (old.max_pos.x - old.min_pos.x).abs()
            } else {
                level.min_pos.x
            };
            draw_texture(t, x, level.max_pos.y + 16.0, WHITE);
            let elevator_pos = get_elevator_pos(self.assets, self.level - 1)
                + vec2(
                    x - old.min_pos.x,
                    (old.min_pos.y - (level.max_pos.y + 16.0)).abs(),
                );
            let elevator_shaft_height = old
                .forced_level_elevator_shaft_height
                .unwrap_or(old.roof_height);
            draw_rectangle(
                elevator_pos.x,
                elevator_shaft_height + (old.min_pos.y - (level.max_pos.y + 16.0)).abs(),
                elevator_texture.width(),
                old.forced_level_end.unwrap_or(old.player_spawn).y - elevator_texture.height()
                    + 8.0
                    - elevator_shaft_height,
                self.world_manager.world_colors[old.get_world_index() as usize].2,
            );
            draw_texture(elevator_texture, elevator_pos.x, elevator_pos.y, WHITE);
            draw_texture(
                &elevator_doors_animation.frames.last().unwrap().0,
                elevator_pos.x,
                elevator_pos.y,
                WHITE,
            );
        }

        let elevator_shaft_height = level
            .forced_level_elevator_shaft_height
            .unwrap_or(level.roof_height);
        draw_rectangle(
            elevator_pos.x,
            elevator_shaft_height,
            elevator_texture.width(),
            elevator_pos.y - elevator_shaft_height,
            self.world_manager.world_colors[level.get_world_index() as usize].2,
        );
        // draw animated tiles
        for (pos, index) in level.animated_tiles.iter() {
            let time = self.time + pos.x.powi(2) + pos.y.powi(2) * 4.2;
            draw_texture(
                self.assets.animated_tiles[*index].get_at_time((time * 1000.0) as u32),
                pos.x,
                pos.y,
                WHITE,
            );
        }
        if DEBUG_FLAGS.paths {
            debug_paths(level);
        }
        self.enemies.retain_mut(|f| {
            f.update(
                &mut self.player,
                &mut self.projectiles,
                self.assets,
                level,
                delta_time,
            )
        });

        // draw level beginning elevator
        if self.level > 0 {
            let time =
                (LEVEL_TRANSITION_LENGTH - self.level_transition_time) / LEVEL_TRANSITION_LENGTH;
            let pos = get_player_spawn(self.assets, self.level);
            let pos = vec2(
                pos.x
                    + if left_level_end {
                        3.0 * 8.0 - elevator_texture.width()
                    } else {
                        -2.0 * 8.0
                    },
                pos.y - elevator_texture.height() + 8.0,
            );
            draw_texture(
                &self.assets.elevator.animations[level.get_world_index() as usize].frames[1].0,
                pos.x,
                pos.y,
                WHITE,
            );
            draw_texture(
                self.assets.doors.animations[1].get_at_time(
                    ((time) * 1000.0).min((self.assets.doors.animations[1].total_length - 1) as f32)
                        as u32,
                ),
                pos.x,
                pos.y,
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

            if DEBUG_FLAGS.horses {
                draw_rectangle(horse.pos.x.floor(), horse.pos.y.floor(), 8.0, 8.0, RED);
                draw_line(
                    horse.pos.x,
                    horse.pos.y,
                    horse.pos.x + normal.x * 16.0,
                    horse.pos.y + normal.y * 16.0,
                    1.0,
                    YELLOW,
                );
            }
        }

        if let Some(boss) = &mut self.boss {
            boss.update(
                self.assets,
                delta_time,
                level,
                &mut self.projectiles,
                &mut self.player,
            );
        }
        self.player.draw(self.assets);
        if let Some(time) = &self.level_complete {
            // draw level end elevator door animation if level complete
            let texture = elevator_doors_animation.get_at_time((*time * 1000.0) as u32);
            draw_texture(texture, elevator_pos.x, elevator_pos.y, WHITE);
        }
        let mut new_projectiles = Vec::new();
        self.projectiles.retain_mut(|projectile| {
            let physics = projectile.get_physics();

            if let Some(friction) = physics {
                const OFFSET: Vec2 = vec2(4.0, 4.0);
                projectile.direction.y += GRAVITY * delta_time;

                let (new_pos, on_ground, _, _) = update_physicsbody(
                    projectile.pos - OFFSET,
                    &mut projectile.direction,
                    delta_time,
                    level,
                    false,
                    false,
                );
                projectile.pos = new_pos + OFFSET;
                if on_ground {
                    projectile.direction.x =
                        projectile.direction.x.lerp(0.0, delta_time * friction);
                }
            } else {
                projectile.pos += projectile.direction * delta_time;
            }
            let rotation = if physics.is_none() {
                0.0
            } else {
                projectile.time
                    * 10.0
                    * if projectile.direction.x < 0.0 {
                        -1.0
                    } else {
                        1.0
                    }
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
                    self.assets.projectiles.animations[projectile.type_index]
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
            if DEBUG_FLAGS.centres {
                draw_cross(projectile.pos.x, projectile.pos.y, WHITE);
            }
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
                    self.player.death = Some((0.0, projectile.player_death_animation(), true));
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
                && if physics.is_none() && projectile.should_die_on_kill() {
                    // check two points for tile collision,
                    // only kill projectile if both are colliding.

                    let mut didnt_hit_wall = false;
                    let tx = (projectile.pos.x / 8.0).floor() as i16;
                    for offset in [0.0, -2.0] {
                        let ty = ((projectile.pos.y + offset) / 8.0).floor() as i16;
                        let tile = level.get_tile(tx, ty);
                        // make projectile hit wall if there is a collision tile or special tile id 672 (projectile barrier)
                        let hit_wall = tile[1] > 0 || tile[3] == 672 + 1;
                        if !hit_wall {
                            didnt_hit_wall = true;
                        }
                    }

                    didnt_hit_wall
                } else {
                    true
                }
        });
        self.projectiles.append(&mut new_projectiles);

        // draw fog
        for point in self.fog_points.iter() {
            let t = &self.assets.clouds.frames[0].0;
            draw_texture(
                t,
                point.pos.x - 32.0,
                point.pos.y - 32.0,
                WHITE.with_alpha(0.02),
            );
        }

        // SCREEN EFFECTS

        if DEBUG_FLAGS.bloom {
            BLOOM_MATERIAL.set_uniform("scale", scale_factor);
            gl_use_material(&BLOOM_MATERIAL);
            draw_rectangle(
                screen_offset.x,
                screen_offset.y,
                actual_screen_width / scale_factor,
                actual_screen_height / scale_factor,
                WHITE,
            );
            gl_use_default_material();
        }

        // DRAW UI

        // draw cinematic bars
        if let Some(bars) = &mut self.player.cinematic_bars {
            let amt = match bars {
                CinematicBars::Extending(time) => {
                    *time += delta_time;
                    *time
                }
                CinematicBars::Retracting(time) => {
                    *time += delta_time;
                    CINEMATIC_BAR_FADE_TIME - *time
                }
            }
            .clamp(0.0, 1.0);
            const CINEMATIC_BAR_HEIGHT: f32 = 12.0;
            let screen_zero_coordinate = vec2(
                self.camera.target.x - actual_screen_width / scale_factor / 2.0,
                self.camera.target.y - actual_screen_height / scale_factor / 2.0,
            );
            draw_rectangle(
                screen_zero_coordinate.x,
                screen_zero_coordinate.y,
                actual_screen_width,
                CINEMATIC_BAR_HEIGHT * amt,
                BLACK,
            );
            draw_rectangle(
                screen_zero_coordinate.x,
                screen_zero_coordinate.y + actual_screen_height / scale_factor
                    - CINEMATIC_BAR_HEIGHT * amt,
                actual_screen_width,
                CINEMATIC_BAR_HEIGHT * amt,
                BLACK,
            );
            if let CinematicBars::Retracting(time) = bars
                && *time >= 1.0
            {
                self.player.cinematic_bars = None;
            }
        }

        // draw dialogue
        if !self.player.has_restarted_level
            && let Some(dialogue) = &self.player.active_dialogue
        {
            let t = &self.assets.dialogue;
            let max_slide_offset = t.width() + 4.0;
            let slide_amt = if dialogue.time < DIALOGUE_SLIDE_IN_TIME {
                dialogue.time / DIALOGUE_SLIDE_IN_TIME
            } else {
                1.0
            };
            let slide_amt = slide_amt.powi(2);
            let pos = vec2(
                self.camera.target.x - actual_screen_width / scale_factor / 2.0 + 2.0
                    - max_slide_offset
                    + max_slide_offset * slide_amt,
                self.camera.target.y - actual_screen_height / scale_factor / 2.0
                    + actual_screen_height / scale_factor
                    - t.height()
                    - 2.0,
            )
            .floor();
            draw_texture(t, pos.x, pos.y, WHITE);
            self.assets.portraits.draw_tile(
                pos.x + 3.0,
                pos.y + 3.0,
                dialogue.portrait_id as f32,
                0.0,
                None,
            );

            let fade_amt = if dialogue.time > DIALOGUE_SLIDE_IN_TIME + TEXT_FADE_IN_TIME {
                1.0
            } else {
                (dialogue.time - DIALOGUE_SLIDE_IN_TIME).max(0.0) / TEXT_FADE_IN_TIME
            };
            let fade_amt = fade_amt.powi(3);

            let font_size = 48;
            let font_scale = 0.25 * 0.5;
            draw_multiline_text_ex(
                dialogue.text,
                pos.x + 28.0,
                pos.y + font_size as f32 * font_scale + 4.0,
                None,
                TextParams {
                    font: Some(&self.assets.font),
                    font_size,
                    font_scale,
                    color: BLACK.with_alpha(fade_amt),
                    ..Default::default()
                },
            );
            let font_scale = 0.25 * 0.5 * 0.5;
            draw_text_ex(
                dialogue.name,
                pos.x + 28.0,
                pos.y + font_size as f32 * font_scale + 1.0,
                TextParams {
                    font: Some(&self.assets.font),
                    font_size,
                    font_scale,
                    color: BLACK.with_alpha(fade_amt),
                    ..Default::default()
                },
            );
        }

        // handle fading out
        if self.fade_timer > 0.0 {
            self.fade_timer -= delta_time;
        }
        let mut fade_amt = self.fade_timer * 2.0;
        if let Some(death) = &self.player.death {
            let delta = death.0 - self.assets.die.animations[death.1].total_length as f32 / 1000.0;
            if delta > 0.0 {
                if delta > 0.5 {
                    let restarted = self.player.in_boss_battle || self.player.has_restarted_level;
                    self.load_level(self.level);
                    self.fade_timer = 0.5;
                    self.player.has_restarted_level = restarted;
                }
                fade_amt = delta * 2.0;
            }
        }
        if fade_amt > 0.0 {
            draw_rectangle(
                screen_offset.x,
                screen_offset.y,
                actual_screen_width,
                actual_screen_height,
                BLACK.with_alpha(fade_amt),
            );
        }
        self.player.time_since_last_boss_defeated += delta_time;

        draw_boss_badges(
            self.assets,
            self.player.time_since_last_boss_defeated,
            self.player.defeated_bosses,
            screen_offset,
            actual_screen_width / scale_factor,
        );
        if is_key_pressed(KeyCode::H) {
            self.player.time_since_last_boss_defeated = 0.0;
            self.player.defeated_bosses = 1;
        }
        if is_key_pressed(KeyCode::L) {
            self.player.cinematic_bars = Some(CinematicBars::Extending(0.0));
        }
    }
}

#[macroquad::main("cowboy tower")]
async fn main() {
    info!("cowboy tower v{}", env!("CARGO_PKG_VERSION"));
    //miniquad::window::set_window_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    let assets = Assets::load();
    let mut level = 0;

    // load level from command line argument
    'outer: for arg in args().skip(1) {
        // check for direct match
        for (i, l) in assets.levels.iter().enumerate() {
            if l.name == arg {
                level = i;
                break 'outer;
            }
        }
        // check for start of name match
        for (i, l) in assets.levels.iter().enumerate() {
            if l.name.starts_with(&arg) {
                level = i;
                break 'outer;
            }
        }
    }
    let mut game = Game::new(&assets, level);

    loop {
        game.update();
        next_frame().await;
    }
}
