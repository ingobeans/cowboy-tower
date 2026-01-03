use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::{
    Projectile,
    assets::{Assets, Horse, Level},
    utils::*,
};

fn ceil_g(a: f32) -> f32 {
    if a < 0.0 { a.floor() } else { a.ceil() }
}

struct ActiveLasso {
    time: f32,
    hook_pos: Vec2,
    speed: f32,
    lasso_length: f32,
    in_swing: bool,
    /// When further away than [lasso_length], player will lerp towards the nearest point
    /// on the lasso arch. This is the source that is used to find the nearest point.
    ///
    /// The source is only changed if a new source which would yield a higher Y value on the target point is found.
    ///
    /// This behaviour is fine tuned to make the lerping look as un-awkward as possible
    lerp_source: Vec2,
}

pub struct Player {
    pub pos: Vec2,
    pub camera_pos: Vec2,
    pub velocity: Vec2,
    pub on_ground: bool,
    pub facing_left: bool,
    pub moving: bool,
    pub time: f32,
    pub jump_time: f32,
    /// Index of horse being ridden
    pub riding: Option<usize>,
    active_lasso: Option<ActiveLasso>,
    pub lasso_target: Option<Vec2>,
    pub death: Option<(f32, usize)>,
    /// If player isnt actively shooting a projectile, this is 0.
    /// Otherwise it will be the time for the shoot animation.
    pub shooting: f32,
}
impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            camera_pos: pos - vec2(0.0, 100.0),
            active_lasso: None,
            lasso_target: None,
            riding: None,
            velocity: Vec2::ZERO,
            on_ground: false,
            jump_time: 0.0,
            facing_left: false,
            moving: false,
            time: 0.0,
            death: None,
            shooting: 0.0,
        }
    }
    pub fn update(
        &mut self,
        delta_time: f32,
        world: &Level,
        projectiles: &mut Vec<Projectile>,
        horses: &mut Vec<Horse>,
    ) {
        if let Some(death) = &mut self.death {
            death.0 += delta_time;
            return;
        }
        const MOVE_SPEED: f32 = 101.0;
        const MOVE_ACCELERATION: f32 = 22.0;
        const JUMP_FORCE: f32 = 160.0;
        self.time += delta_time;
        let input = get_input_axis();

        if self.on_ground {
            self.jump_time = 0.0;
        } else if self.jump_time > 0.0 {
            self.jump_time += delta_time
        }

        if self.shooting > 0.0 {
            self.shooting += delta_time;
        } else if self.active_lasso.as_ref().is_none_or(|f| f.time == 0.0)
            && is_mouse_button_pressed(MouseButton::Left)
            && self.riding.is_none()
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

                let delta = target_pos - self.pos;
                let normalized = delta.normalize();
                self.velocity = self
                    .velocity
                    .lerp(normalized * MOVE_SPEED, delta_time * 5.0);
                self.velocity = self.velocity.lerp(self.velocity * 1.2, delta_time * 5.0);
            }
            if !is_mouse_button_down(MouseButton::Right) {
                self.active_lasso = None;
            }
        } else {
            // find nearest lasso target in direction player is facing
            let mut targets: Vec<&Vec2> = world
                .lasso_targets
                .iter()
                .filter(|f| {
                    f.distance(self.pos) <= MAX_LASSO_DISTANCE
                        && if self.facing_left {
                            f.x < self.pos.x
                        } else {
                            f.x > self.pos.x
                        }
                })
                .collect();
            self.lasso_target = None;
            if !targets.is_empty() {
                targets.sort_by(|a, b| {
                    (a.x.powi(2) + a.y.powi(2))
                        .sqrt()
                        .total_cmp(&(b.x.powi(2) + b.y.powi(2)).sqrt())
                });
                let closest = targets[0].clone();
                self.lasso_target = Some(closest);
            }
            if is_mouse_button_pressed(MouseButton::Right)
                && let Some(target) = &self.lasso_target
            {
                self.active_lasso = Some(ActiveLasso {
                    time: delta_time,
                    hook_pos: *target,
                    speed: f32::NAN,
                    lasso_length: target.distance(self.pos),
                    in_swing: false,
                    lerp_source: self.pos,
                });
            }

            self.velocity.x = self
                .velocity
                .x
                .lerp(input.x * MOVE_SPEED, delta_time * MOVE_ACCELERATION);
            self.velocity.y += GRAVITY * delta_time;

            self.moving = input.x != 0.0;
            if self.moving {
                self.facing_left = input.x.is_sign_negative();
            }

            if is_key_pressed(KeyCode::Space) {
                if let Some(horse) = self.riding {
                    self.riding = None;
                    self.jump_time = delta_time;
                    self.velocity = horses[horse].velocity;

                    let normal = horses[horse].get_normal();
                    self.velocity += normal * JUMP_FORCE;
                    horses[horse].player_riding = false;
                } else {
                    // check if by horse
                    let mut horses: Vec<(usize, &mut Horse)> = horses
                        .iter_mut()
                        .enumerate()
                        .filter(|f| f.1.pos.distance(self.pos) < 16.0)
                        .collect();
                    if !horses.is_empty() {
                        horses.sort_by(|a, b| {
                            a.1.pos
                                .distance(self.pos)
                                .total_cmp(&b.1.pos.distance(self.pos))
                        });
                        self.riding = Some(horses[0].0);
                        horses[0].1.running = true;
                        horses[0].1.player_riding = true;
                    } else if self.on_ground {
                        self.jump_time = delta_time;
                        self.velocity.y = -JUMP_FORCE;
                    }
                }
            }
        }
        let old_velocity = self.velocity;
        let touched_death_tile;

        if let Some(horse) = &self.riding {
            self.pos = horses[*horse].pos + horses[*horse].get_normal() * 16.0;
        } else {
            (self.pos, self.on_ground, touched_death_tile) =
                update_physicsbody(self.pos, &mut self.velocity, delta_time, world, true);
            if let Some(tile) = touched_death_tile
                && self.death.is_none()
            {
                self.death = Some((0.0, DEATH_TILES.iter().position(|f| *f == tile).unwrap()));
            }
        }

        if old_velocity.length() > self.velocity.length()
            && let Some(lasso) = &mut self.active_lasso
        {
            lasso.speed = 0.0;
        }
        self.camera_pos.x = self
            .pos
            .x
            .max(world.min_pos.x + SCREEN_WIDTH / 2.0 - 64.0)
            .min(world.max_pos.x + 16.0 * 8.0 - (SCREEN_WIDTH / 2.0 - 64.0));
        let target = self.pos.y - 22.0;
        if self.camera_pos.y < target {
            self.camera_pos.y = target;
        } else {
            let delta = self.camera_pos.y - target;
            let max_delta = 3.5 * 8.0;
            if delta.abs() > max_delta {
                self.camera_pos.y = max_delta * if delta < 0.0 { -1.0 } else { 1.0 } + target;
            }
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
                &assets.target.get_at_time((self.time * 1000.0) as u32),
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

        let mut torso = assets.torso.animations[if self.shooting > 0.0 { 1 } else { 0 }]
            .get_at_time((self.shooting * 1000.0) as u32);
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
                    moved.y,
                    1.0,
                    Color::from_hex(0x773421),
                );
            }
        }

        // draw legs and torso textures

        let legs = if self.jump_time > 0.0 {
            let anim = &assets.legs.animations[2];
            if self.jump_time * 1000.0 >= anim.total_length as f32 {
                self.jump_time = 0.0;
            }
            anim.get_at_time((self.jump_time * 1000.0) as u32)
        } else {
            assets.legs.animations[if self.moving { 1 } else { 0 }]
                .get_at_time((self.time * 1000.0) as u32)
        };
        for texture in [legs, torso] {
            draw_texture_ex(
                texture,
                self.pos.x.floor() - texture.width() / 2.0 + 4.0,
                self.pos.y.floor() - 8.0,
                WHITE,
                DrawTextureParams {
                    flip_x: self.facing_left,
                    ..Default::default()
                },
            );
        }
    }
}

pub fn update_physicsbody(
    pos: Vec2,
    velocity: &mut Vec2,
    delta_time: f32,
    world: &Level,
    tall: bool,
) -> (Vec2, bool, Option<u16>) {
    let mut grounded = false;
    let mut touched_death_tile = None;
    let mut new = pos + *velocity * delta_time;

    let tile_x = pos.x / 8.0;
    let tile_y = pos.y / 8.0;

    let mut tiles_y = vec![
        (tile_x.trunc(), ceil_g(new.y / 8.0)),
        (ceil_g(tile_x), ceil_g(new.y / 8.0)),
        (tile_x.trunc(), (new.y / 8.0).trunc()),
        (ceil_g(tile_x), (new.y / 8.0).trunc()),
    ];
    if tall {
        tiles_y.push((tile_x.trunc(), (new.y / 8.0).floor() - 1.0));
        tiles_y.push((ceil_g(tile_x), (new.y / 8.0).floor() - 1.0));
    }

    for (tx, ty) in tiles_y {
        let tile = world.get_tile((tx) as i16, (ty) as i16)[1];
        if !grounded && tile > 0 && DEATH_TILES.contains(&(tile - 1)) {
            touched_death_tile = Some(tile - 1);
            continue;
        }
        if tile != 0 {
            let c = if velocity.y < 0.0 {
                tile_y.floor() * 8.0
            } else {
                grounded = true;
                touched_death_tile = None;
                tile_y.ceil() * 8.0
            };
            new.y = c;
            velocity.y = 0.0;
            break;
        }
    }
    let mut tiles_x = vec![
        ((new.x / 8.0).trunc(), ceil_g(new.y / 8.0)),
        (ceil_g(new.x / 8.0), ceil_g(new.y / 8.0)),
        (ceil_g(new.x / 8.0), (new.y / 8.0).trunc()),
        ((new.x / 8.0).trunc(), (new.y / 8.0).trunc()),
    ];
    if tall {
        tiles_x.push(((new.x / 8.0).trunc(), (new.y / 8.0).floor() - 1.0));
        tiles_x.push((ceil_g(new.x / 8.0), (new.y / 8.0).floor() - 1.0));
    }

    for (tx, ty) in tiles_x {
        let tile = world.get_tile((tx) as i16, (ty) as i16)[1];
        if tile > 0 && DEATH_TILES.contains(&(tile - 1)) {
            continue;
        }
        if tile != 0 {
            let c = if velocity.x < 0.0 {
                tile_x.floor() * 8.0
            } else {
                tile_x.ceil() * 8.0
            };
            new.x = c;
            velocity.x = 0.0;
            break;
        }
    }
    (new, grounded, touched_death_tile)
}
