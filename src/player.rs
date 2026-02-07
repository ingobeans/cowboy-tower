use std::f32::consts::PI;

use gamepads::Gamepads;
use macroquad::prelude::*;

use crate::{
    assets::{Assets, Horse, Level},
    projectiles::*,
    utils::*,
};
pub use physics::*;

mod physics;
struct ActiveLasso {
    time: f32,
    hook_pos: Vec2,
    speed: f32,
    lasso_length: f32,
    space_activated: bool,
    in_swing: bool,
    /// When further away than [lasso_length], player will lerp towards the nearest point
    /// on the lasso arch. This is the source that is used to find the nearest point.
    ///
    /// The source is only changed if a new source which would yield a higher Y value on the target point is found.
    ///
    /// This behaviour is fine tuned to make the lerping look as un-awkward as possible
    lerp_source: Vec2,
}

pub struct ActiveDialogue {
    pub text: &'static str,
    pub name: &'static str,
    pub portrait_id: usize,
    pub closed: bool,
    pub time: f32,
}

pub enum CinematicBars {
    Extending(f32),
    Retracting(f32),
}

struct ActiveRiding {
    horse_index: usize,
    camera_lerp_time: f32,
    camera_lerp_src: Vec2,
}

const HORSE_MOUNT_LEEWAY: f32 = 0.2;
const JUMP_LAND_LEEWAY: f32 = 0.05;
const COYOTE_TIME: f32 = 0.05;

const MOVE_INABILITY_AFTER_WALL_JUMP: f32 = 0.23;
const COYOTE_TIME_WALL_JUMP: f32 = 0.15;

pub struct Player {
    pub pos: Vec2,
    pub camera_pos: Vec2,
    pub velocity: Vec2,
    pub on_ground: bool,
    pub facing_left: bool,
    pub moving: bool,
    pub time: f32,
    pub cinematic_bars: Option<CinematicBars>,
    jump_time: f32,
    pub active_dialogue: Option<ActiveDialogue>,
    riding: Option<ActiveRiding>,
    active_lasso: Option<ActiveLasso>,
    lasso_target: Option<Vec2>,
    pub death: Option<(f32, usize, bool)>,
    wall_climbing: Option<(f32, f32)>,
    /// Time since last jump off wall
    jump_of_wall_time: f32,
    /// After falling off wall when wall-climbing, this is set.
    /// - time since fall
    /// - direction
    fall_of_wall: (f32, f32),
    /// Timer since how long it was since player last was on_ground
    last_touched_ground: f32,
    /// Count
    pub defeated_bosses: u8,
    pub time_since_last_boss_defeated: f32,
    /// If player isnt actively shooting a projectile, this is 0.
    /// Otherwise it will be the time for the shoot animation.
    shooting: f32,
    pub in_boss_battle: bool,
    /// If the player isnt playing the level for the first time.
    /// Used to skip bosses' dialogue automatically
    pub has_restarted_level: bool,
    /// When player presses space in the air, and doesn't succesfully mount a horse,
    /// this value is set to [HORSE_MOUNT_LEEWAY]. If the player comes within range of a horse,
    /// before this timer reaches 0.0, the player will mount that horse.
    ///
    /// This gives a bit of leeway when mounting horses mid-air.
    failed_horse_mount_time: f32,
}
impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            camera_pos: pos - vec2(0.0, 100.0),
            active_lasso: None,
            lasso_target: None,
            riding: None,
            active_dialogue: None,
            cinematic_bars: None,
            jump_of_wall_time: 10.0,
            velocity: Vec2::ZERO,
            on_ground: false,
            last_touched_ground: 1.0,
            jump_time: 0.0,
            fall_of_wall: (COYOTE_TIME_WALL_JUMP, 0.0),
            time_since_last_boss_defeated: 10.0,
            defeated_bosses: 0,
            failed_horse_mount_time: 0.0,
            facing_left: false,
            moving: false,
            wall_climbing: None,
            time: 0.0,
            death: None,
            shooting: 0.0,
            in_boss_battle: false,
            has_restarted_level: false,
        }
    }
    pub fn show_cinematic_bars(&mut self) {
        self.cinematic_bars = Some(CinematicBars::Extending(0.0));
    }
    pub fn hide_cinematic_bars(&mut self) {
        self.cinematic_bars = Some(CinematicBars::Retracting(0.0));
    }
    pub fn show_dialogue(&mut self, text: &'static str, name: &'static str, portrait_id: usize) {
        self.active_dialogue = Some(ActiveDialogue {
            text,
            name,
            portrait_id,
            closed: self.has_restarted_level,
            time: if self.active_dialogue.is_some() {
                TEXT_FADE_IN_TIME + DIALOGUE_SLIDE_IN_TIME + 1.0
            } else {
                0.0
            },
        })
    }
    pub fn update(
        &mut self,
        delta_time: f32,
        level: &Level,
        projectiles: &mut Vec<Projectile>,
        horses: &mut [Horse],
        gamepad_engine: &mut Gamepads,
    ) {
        if let Some(death) = &mut self.death {
            death.0 += delta_time;
            if let Some(riding) = self.riding.take() {
                horses[riding.horse_index].player_riding = false;
            }
            if death.2 {
                self.velocity.x = 0.0;
                self.velocity.y += GRAVITY * delta_time;
                (self.pos, self.on_ground, _, _) = update_physicsbody(
                    self.pos,
                    &mut self.velocity,
                    delta_time,
                    level,
                    true,
                    false,
                );
            }
            return;
        }
        if let Some(dialogue) = &mut self.active_dialogue {
            dialogue.time += delta_time;
            if is_interact_pressed(gamepad_engine) {
                dialogue.closed = true;
            }
            return;
        }
        if self.failed_horse_mount_time > 0.0 {
            self.failed_horse_mount_time -= delta_time;
            if self.on_ground {
                if HORSE_MOUNT_LEEWAY - self.failed_horse_mount_time < JUMP_LAND_LEEWAY {
                    self.jump_time = delta_time;
                    self.velocity.y = -JUMP_FORCE;
                    self.on_ground = false;
                }
                self.failed_horse_mount_time = 0.0;
            } else if let Some(horse) = self.find_mountable_horse(horses) {
                self.riding = Some(ActiveRiding {
                    horse_index: horse.0,
                    camera_lerp_time: delta_time,
                    camera_lerp_src: self.camera_pos,
                });
                horse.1.running = true;
                horse.1.player_riding = true;
            }
        }
        if self.jump_of_wall_time < MOVE_INABILITY_AFTER_WALL_JUMP {
            self.jump_of_wall_time += delta_time;
        }
        if self.fall_of_wall.0 < COYOTE_TIME_WALL_JUMP {
            self.fall_of_wall.0 += delta_time;
            if self.on_ground {
                self.fall_of_wall.0 = COYOTE_TIME_WALL_JUMP;
            }
        }
        const MOVE_SPEED: f32 = 101.0;
        const MOVE_ACCELERATION: f32 = 22.0;
        const JUMP_FORCE: f32 = 160.0;
        self.time += delta_time;
        let input = get_input_axis(gamepad_engine);

        if self.on_ground {
            self.jump_time = 0.0;
        } else if self.jump_time > 0.0 {
            self.jump_time += delta_time
        }

        if self.shooting > 0.0 {
            self.shooting += delta_time;
        } else if self.active_lasso.as_ref().is_none_or(|f| f.time == 0.0)
            && is_shoot_pressed(gamepad_engine)
            && self.riding.is_none()
            && self.wall_climbing.is_none()
        {
            self.shooting += delta_time;
            projectiles.push(Projectile::new(
                0,
                self.pos
                    + if self.facing_left {
                        vec2(-8.0, 0.0)
                    } else {
                        vec2(8.0, 0.0)
                    }
                    + vec2(4.0, 0.0),
                vec2(if self.facing_left { -1.0 } else { 1.0 }, 0.0),
            ));
        }

        if let Some(lasso) = &mut self.active_lasso {
            self.lasso_target = None;
            lasso.lasso_length = lasso.lasso_length.min(32.0);
            if lasso.time > 0.0 {
                lasso.time += delta_time;
            }
            if !lasso.in_swing && self.pos.distance(lasso.hook_pos) - 2.0 <= lasso.lasso_length {
                lasso.in_swing = true;
                lasso.speed = f32::NAN;
            }
            if lasso.in_swing {
                self.moving = false;
                // without drag the player over time builds more and more speed.
                // the drag factor is also relative to the lasso length since a shorter lasso length
                // yields faster acceleration
                let drag_factor: f32 = 0.8 * lasso.lasso_length / 32.0;

                let down = vec2(0.0, lasso.lasso_length);
                let delta = self.pos - lasso.hook_pos;

                let angle = delta.to_angle();
                let right_half_circle = angle < PI / 2.0 && angle > -PI / 2.0;
                if lasso.speed.is_nan() {
                    lasso.speed = (-self.velocity.x).clamp(-GRAVITY, GRAVITY);
                }

                lasso.speed *= 1.0.lerp(drag_factor, delta_time);

                let new_angle = angle + lasso.speed * delta_time / lasso.lasso_length;
                let new_delta_normalized = Vec2::from_angle(new_angle);
                let new_delta = new_delta_normalized * lasso.lasso_length;
                let move_amt = new_delta - delta;
                self.velocity = move_amt / delta_time;

                let down_delta_delta = down - delta;
                lasso.speed += down_delta_delta.y * delta_time * GRAVITY / lasso.lasso_length
                    * if right_half_circle { 1.0 } else { -1.0 };
            } else {
                const MOVE_SPEED: f32 = 128.0;
                let mut target_pos = (lasso.lerp_source - lasso.hook_pos).normalize()
                    * lasso.lasso_length
                    + lasso.hook_pos;
                let new_target_pos =
                    (self.pos - lasso.hook_pos).normalize() * lasso.lasso_length + lasso.hook_pos;
                if new_target_pos.y > target_pos.y {
                    lasso.lerp_source = self.pos;
                    target_pos = new_target_pos;
                }
                if self.jump_time <= 0.0 {
                    self.jump_time = delta_time;
                }

                let delta = target_pos - self.pos;
                let normalized = delta.normalize();
                self.velocity = self
                    .velocity
                    .lerp(normalized * MOVE_SPEED, delta_time * 5.0);
                self.velocity = self.velocity.lerp(self.velocity * 1.2, delta_time * 5.0);
            }
            if lasso.space_activated {
                if !is_jump_down(gamepad_engine) && !is_lasso_down(gamepad_engine) {
                    self.active_lasso = None;
                }
            } else if !is_lasso_down(gamepad_engine) {
                self.active_lasso = None;
            }
        } else {
            // find nearest lasso target in direction player is facing
            let mut targets: Vec<&Vec2> = level
                .lasso_targets
                .iter()
                .filter(|f| {
                    (if self.facing_left {
                        f.x < self.pos.x
                    } else {
                        f.x > self.pos.x
                    }) && f.distance(self.pos) <= MAX_LASSO_DISTANCE
                        && raycast(**f, self.pos, level).is_none()
                })
                .collect();
            self.lasso_target = None;
            if !targets.is_empty() {
                targets.sort_by(|a, b| {
                    (a.x.powi(2) + a.y.powi(2))
                        .sqrt()
                        .total_cmp(&(b.x.powi(2) + b.y.powi(2)).sqrt())
                });
                let closest = *targets[0];
                self.lasso_target = Some(closest);
            }
            if is_lasso_pressed(gamepad_engine)
                && let Some(target) = &self.lasso_target
            {
                self.active_lasso = Some(ActiveLasso {
                    time: delta_time,
                    hook_pos: *target,
                    speed: f32::NAN,
                    lasso_length: target.distance(self.pos),
                    in_swing: false,
                    lerp_source: self.pos,
                    space_activated: false,
                });
            }

            if self.jump_of_wall_time >= MOVE_INABILITY_AFTER_WALL_JUMP {
                self.velocity.x = self
                    .velocity
                    .x
                    .lerp(input.x * MOVE_SPEED, delta_time * MOVE_ACCELERATION);
            }

            if self.wall_climbing.is_none() || self.velocity.y < 0.0 {
                self.velocity.y += GRAVITY * delta_time;
            } else if let Some((time, _)) = self.wall_climbing {
                // handle gliding down wall when wall climbing
                const STOP_TIME: f32 = 0.4;
                const MAX_SPEED: f32 = 48.0;
                const ACCELERATE_TIME: f32 = 1.5;

                // the speed of which you glide should follow a curve that looks something like:
                //
                //              _______________________
                //             /
                //           /
                //  _______/
                //         ^
                // STOP_TIME
                //
                //         <---->  ACCELERATE_TIME

                let amt = if time < STOP_TIME {
                    0.0
                } else if time < STOP_TIME + ACCELERATE_TIME {
                    MAX_SPEED * (time - STOP_TIME) / ACCELERATE_TIME
                } else {
                    MAX_SPEED
                };
                self.velocity.y = amt;
            }

            self.moving = input.x != 0.0;
            if self.moving {
                self.facing_left = input.x.is_sign_negative();
            }

            let wall_jump_state = if let Some((_, direction)) = self.wall_climbing {
                Some(direction)
            } else if self.fall_of_wall.0 < COYOTE_TIME_WALL_JUMP {
                Some(self.fall_of_wall.1)
            } else {
                None
            };

            if is_jump_pressed(gamepad_engine) {
                if let Some(direction) = wall_jump_state {
                    self.jump_time = delta_time;
                    self.velocity.y = -JUMP_FORCE * 1.2;
                    self.on_ground = false;
                    self.velocity.x = 0.65 * JUMP_FORCE * -direction;
                    self.jump_of_wall_time = 0.0;
                    // prevent extra jumps with coyote time
                    self.fall_of_wall.0 = COYOTE_TIME_WALL_JUMP;
                } else if let Some(riding) = self.riding.take() {
                    self.jump_time = delta_time;
                    self.velocity = horses[riding.horse_index].velocity;

                    let normal = horses[riding.horse_index].get_normal();
                    self.velocity += normal * JUMP_FORCE;
                    horses[riding.horse_index].player_riding = false;
                } else {
                    // check if by horse
                    if let Some(horse) = self.find_mountable_horse(horses) {
                        self.riding = Some(ActiveRiding {
                            horse_index: horse.0,
                            camera_lerp_time: delta_time,
                            camera_lerp_src: self.camera_pos,
                        });
                        horse.1.running = true;
                        horse.1.player_riding = true;
                    } else if self.on_ground
                        || (self.last_touched_ground < COYOTE_TIME && self.jump_time <= 0.0)
                    {
                        self.jump_time = delta_time;
                        self.velocity.y = -JUMP_FORCE;
                        self.on_ground = false;
                    } else {
                        // check if we can lasso
                        if self.active_lasso.is_none()
                            && let Some(target) = &self.lasso_target
                        {
                            self.active_lasso = Some(ActiveLasso {
                                time: delta_time,
                                hook_pos: *target,
                                speed: f32::NAN,
                                lasso_length: target.distance(self.pos),
                                in_swing: false,
                                lerp_source: self.pos,
                                space_activated: true,
                            });
                        } else {
                            // failed to mount horse or jump.
                            self.failed_horse_mount_time = HORSE_MOUNT_LEEWAY;
                        }
                    }
                }
            }
        }

        if !self.on_ground {
            self.last_touched_ground += delta_time;
        }

        let old_velocity = self.velocity;
        let touched_death_tile;
        let colliding_with_wall_climb_target;

        if let Some(riding) = &self.riding {
            self.pos =
                horses[riding.horse_index].pos + horses[riding.horse_index].get_normal() * 16.0;
        } else {
            (
                self.pos,
                self.on_ground,
                touched_death_tile,
                colliding_with_wall_climb_target,
            ) = update_physicsbody(
                self.pos,
                &mut self.velocity,
                delta_time,
                level,
                true,
                self.in_boss_battle,
            );
            if self.on_ground {
                self.last_touched_ground = 0.0;
            }
            if let Some(direction) = colliding_with_wall_climb_target {
                if let Some((time, _)) = &mut self.wall_climbing {
                    *time += delta_time;
                } else {
                    self.wall_climbing = Some((delta_time, direction));
                }
            } else {
                if let Some((_, direction)) = self.wall_climbing
                    && self.jump_of_wall_time > MOVE_INABILITY_AFTER_WALL_JUMP
                {
                    // wall climbing was canceled
                    self.fall_of_wall = (delta_time, direction);
                }
                self.wall_climbing = None;
            }
            if let Some(tile) = touched_death_tile
                && self.death.is_none()
            {
                let death_tile_index = DEATH_TILES.iter().position(|f| *f == tile).unwrap();
                self.death = Some((0.0, death_tile_index, false));
            }
        }

        if old_velocity.length() > self.velocity.length()
            && let Some(lasso) = &mut self.active_lasso
        {
            lasso.speed = 0.0;
        }

        let min_x = if level.name != "0-0.tmx" {
            level.min_pos.x
        } else {
            level.find_marker(3).x
        };
        let max_x = if level.name != "0-0.tmx" {
            level.max_pos.x
        } else {
            level.find_marker(1).x - 16.0 * 8.0 + 8.0
        };
        let mut target_camera_pos = self.camera_pos;
        target_camera_pos.x = self
            .pos
            .x
            .max(min_x + SCREEN_WIDTH / 2.0 - 64.0)
            .min(max_x + 16.0 * 8.0 - (SCREEN_WIDTH / 2.0 - 64.0));
        let target = self.pos.y - 22.0;
        if target_camera_pos.y < target {
            target_camera_pos.y = target;
        } else {
            let delta = target_camera_pos.y - target;
            let max_delta = 3.5 * 8.0;
            if delta.abs() > max_delta {
                target_camera_pos.y = max_delta * if delta < 0.0 { -1.0 } else { 1.0 } + target;
            }
        }
        if let Some(riding) = &mut self.riding
            && riding.camera_lerp_time > 0.0
        {
            const LERP_TIME: f32 = 0.25;
            self.camera_pos = riding
                .camera_lerp_src
                .lerp(target_camera_pos, riding.camera_lerp_time / LERP_TIME);
            riding.camera_lerp_time += delta_time;
            if riding.camera_lerp_time >= LERP_TIME {
                riding.camera_lerp_time = 0.0;
            }
        } else {
            self.camera_pos = target_camera_pos;
        }
        if level.name == "0-0.tmx" {
            self.camera_pos.y = self.camera_pos.y.min(-22.0);
        }
    }
    fn find_mountable_horse<'a>(&self, horses: &'a mut [Horse]) -> Option<(usize, &'a mut Horse)> {
        let mut horses: Vec<(usize, &'a mut Horse)> = horses
            .iter_mut()
            .enumerate()
            .filter(|f| {
                // special case: if horse is upside down, move point player distance is checked from down,
                // to make mounting easier (you dont need to tap space twice).
                if f.1.flip && f.1.direction.x.abs() > 0.5 {
                    (f.1.pos + vec2(0.0, 8.0)).distance(self.pos) < 16.0
                } else {
                    f.1.pos.distance(self.pos) < 16.0
                }
            })
            .collect();
        if !horses.is_empty() {
            horses.sort_by(|a, b| {
                a.1.pos
                    .distance(self.pos)
                    .total_cmp(&b.1.pos.distance(self.pos))
            });
            let best = horses.remove(0);
            Some(best)
        } else {
            None
        }
    }
    pub fn draw(&mut self, assets: &Assets) {
        if self.riding.is_some() {
            return;
        }
        if let Some(death) = self.death {
            let time =
                ((death.0 * 1000.0) as u32).min(assets.die.animations[death.1].total_length - 1);
            let texture = assets.die.animations[death.1].get_at_time(time);
            draw_texture_ex(
                texture,
                self.pos.x.floor() - 11.0,
                self.pos.y.floor() - 8.0,
                WHITE,
                DrawTextureParams {
                    flip_x: self.facing_left,
                    ..Default::default()
                },
            );
            return;
        }

        if let Some(target) = &self.lasso_target {
            draw_texture_ex(
                assets.target.get_at_time((self.time * 1000.0) as u32),
                target.x - 8.0,
                target.y - 8.0,
                WHITE,
                DrawTextureParams {
                    flip_x: self.facing_left,
                    ..Default::default()
                },
            );
        }
        if self.shooting * 1000.0 >= assets.torso.animations[1].total_length as f32 {
            self.shooting = 0.0;
        }

        let mut torso = if self.wall_climbing.is_some() {
            &assets.torso.animations[3].frames[0].0
        } else {
            assets.torso.animations[if self.shooting > 0.0 { 1 } else { 0 }]
                .get_at_time((self.shooting * 1000.0) as u32)
        };
        if let Some(lasso) = &mut self.active_lasso {
            const LASSO_EXTEND_TIME: f32 = 0.2;
            const LASSO_EARLY_START: f32 = 0.1;
            let delta = lasso.time - assets.torso.animations[2].total_length as f32 / 1000.0;
            if lasso.time > 0.0 {
                let mut active_time = lasso.time;
                if delta > 0.0 {
                    active_time = (assets.torso.animations[2].total_length - 1) as f32;
                    if delta + LASSO_EARLY_START > LASSO_EXTEND_TIME {
                        lasso.time = 0.0;
                    }
                }
                torso = assets.torso.animations[2].get_at_time((active_time * 1000.0) as u32);
            }
            if delta + LASSO_EARLY_START > 0.0 || lasso.time == 0.0 {
                let amt = if lasso.time == 0.0 {
                    1.0
                } else {
                    (delta + LASSO_EARLY_START) / LASSO_EXTEND_TIME
                };
                let target_delta_pos = lasso.hook_pos - self.pos;
                let normalized = target_delta_pos.normalize();
                let scaled = normalized * target_delta_pos.length() * amt;
                let moved = scaled + self.pos;
                draw_line(
                    self.pos.x + if self.facing_left { 8.0 } else { 0.0 },
                    self.pos.y,
                    moved.x,
                    moved.y + 3.0,
                    1.0,
                    Color::from_hex(0x773421),
                );
                if amt >= 1.0 {
                    assets.tileset.draw_tile(
                        lasso.hook_pos.x - 4.0,
                        lasso.hook_pos.y - 4.0,
                        1.0,
                        4.0,
                        None,
                    );
                }
            }
        }

        // draw legs and torso textures

        let legs = if self.wall_climbing.is_some() {
            &assets.legs.animations[3].frames[0].0
        } else if self.jump_time > 0.0 {
            let anim = &assets.legs.animations[2];
            if self.jump_time * 1000.0 >= anim.total_length as f32 {
                self.jump_time = 0.0;
            }
            anim.get_at_time((self.jump_time * 1000.0) as u32)
        } else {
            assets.legs.animations[if self.moving { 1 } else { 0 }]
                .get_at_time((self.time * 1000.0) as u32)
        };
        let draw_pos = vec2(self.pos.x.floor() - 8.0, self.pos.y.floor() - 8.0);
        for texture in [legs, torso] {
            draw_texture_ex(
                texture,
                draw_pos.x,
                draw_pos.y,
                WHITE,
                DrawTextureParams {
                    flip_x: self.facing_left,
                    ..Default::default()
                },
            );
        }

        if DEBUG_FLAGS.centres {
            draw_cross(self.pos.x, self.pos.y, BLUE);
        }
    }
}
