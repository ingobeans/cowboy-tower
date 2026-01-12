use macroquad::prelude::*;

use crate::{
    assets::{Assets, Level},
    bosses::Boss,
    player::Player,
    projectiles::Projectile,
};

fn populate_fireball_positions(
    positions: &mut Vec<f32>,
    player_pos: Vec2,
    left_target: Vec2,
    right_target: Vec2,
) {
    loop {
        for item in positions.iter_mut() {
            *item = rand::gen_range(left_target.x, right_target.x);
        }
        // force last position to be directly on player
        *positions.last_mut().unwrap() = player_pos.x;

        // check that there is at least one space between two fireballs greater than 64 pixels
        positions.sort_by(|a, b| a.total_cmp(&b));
        let mut last = positions[0];
        for pos in positions.iter().skip(1) {
            let delta = *pos - last;
            if delta >= 64.0 {
                return;
            }
            last = *pos;
        }
    }
}

enum State {
    /// - Wait time
    Idle(f32),
    /// - Count of waves
    /// - Current target positions
    Fireballs(u8, Vec<f32>),
    /// - Jump count
    /// - Jump animation phase
    /// - Jump src
    /// - Jump target
    Jump(u8, usize, Vec2, Vec2),
    LandOnPipe,
    /// - Pipe position on death
    Death(f32),
}
pub struct Fireking {
    pos: Vec2,
    spawn: Vec2,
    health: u8,
    state: State,
    time: f32,
    activated: f32,
    blood_effects: Vec<(Vec2, f32, bool)>,
    dialogue_id: usize,
}
impl Fireking {
    pub fn new(pos: Vec2) -> Self {
        Fireking {
            pos,
            spawn: pos,
            health: 10,
            state: State::Idle(1.0),
            time: 0.0,
            activated: 0.0,
            blood_effects: Vec::new(),
            dialogue_id: 0,
        }
    }
}
impl Boss for Fireking {
    fn update(
        &mut self,
        assets: &Assets,
        delta_time: f32,
        level: &Level,
        projectiles: &mut Vec<Projectile>,
        player: &mut Player,
    ) {
        let dialogue_messages = &["I see you have defeated Henry.", "But now you shall burn."];
        const FIREBALL_FALL_TIME: f32 = 1.0;
        const FIREBALL_AMT: usize = 10;
        const FIREBALL_WAVE_AMT: u8 = 3;
        const PIPE_MOVE_TIME: f32 = 1.0;

        let mut pipe_pos = self.pos.y;

        let loop_animation;
        let mut flipped = false;
        let mut animation;

        let dead = matches!(self.state, State::Death(_));

        let left_target = level.find_marker(0);
        let right_target = level.find_marker(1);

        if player.pos.x <= right_target.x {
            player.camera_pos.y = player.camera_pos.y.min(-80.0);
        }

        self.time += delta_time;
        if self.activated <= 0.0 {
            loop_animation = true;
            animation = 0;

            if let Some(dialogue) = &mut player.active_dialogue {
                if dialogue.closed {
                    if self.dialogue_id + 1 >= dialogue_messages.len() {
                        player.show_cinematic_bars();
                        self.activated = delta_time;
                        projectiles.clear();
                        self.time = 0.0;
                        player.active_dialogue = None;
                        player.in_boss_battle = true;
                    } else {
                        self.dialogue_id += 1;
                        player.show_dialogue(dialogue_messages[self.dialogue_id], "Fireking", 1);
                    }
                }
            } else if player.pos.x < level.find_marker(4).x && player.on_ground {
                player.show_dialogue(dialogue_messages[0], "Fireking", 1);
            }
        } else {
            if player.death.is_none() && player.pos.x > level.find_marker(3).x + 8.0 {
                player.death = Some((0.0, 1, false))
            }

            match &mut self.state {
                State::Death(pos) => {
                    animation = 5;
                    loop_animation = false;
                    pipe_pos = pos.lerp(self.spawn.y, (self.time / 0.5).min(1.0));
                    if pipe_pos + 1.0 >= self.spawn.y {
                        player.in_boss_battle = false;
                    }
                }
                State::Idle(wait) => {
                    animation = 0;
                    loop_animation = true;
                    if self.time >= *wait {
                        let mut positions = vec![0.0; FIREBALL_AMT];
                        populate_fireball_positions(
                            &mut positions,
                            player.pos,
                            left_target,
                            right_target,
                        );
                        self.state = State::Fireballs(0, positions);
                        self.time = 0.0;
                    }
                }
                State::LandOnPipe => {
                    animation = 0;
                    loop_animation = true;
                    const PIPE_MOVE_DOWN_TIME: f32 = 0.25;
                    self.pos.y = (self.spawn.y - 4.0 * 8.0)
                        .lerp(self.spawn.y, (self.time / PIPE_MOVE_DOWN_TIME).min(1.0));
                    if self.time / PIPE_MOVE_DOWN_TIME >= 1.0 {
                        self.pos.y = self.spawn.y;
                        self.state = State::Idle(2.0);
                        self.time = 0.0;
                    }
                    pipe_pos = self.pos.y;
                }
                State::Jump(amt, phase, src, target) => {
                    const JUMP_AIR_TIME: f32 = 1.0;
                    const JUMP_HEIGHT: f32 = 68.0;
                    const JUMP_AMT: u8 = 5;
                    loop_animation = false;
                    pipe_pos = self.spawn.y - 4.0 * 8.0;
                    animation = 2 + *phase;
                    flipped = src.x > target.x;

                    let delta = self.time
                        - assets.fireking.animations[animation].total_length as f32 / 1000.0;
                    let animation_finished = delta >= 0.0;

                    if animation_finished {
                        if *phase == 0 {
                            *phase += 1;
                            self.time = 0.0;
                        } else if *phase == 1 {
                            let jump = delta / JUMP_AIR_TIME;
                            let y_amt = -4.0 * jump.powi(2) + 4.0 * jump;
                            let y = src.y - y_amt * JUMP_HEIGHT + (target.y - src.y) * jump;

                            let x_amt = (jump.powi(2) + jump) * 0.5;
                            let x = src.x.lerp(target.x, x_amt);

                            self.pos = vec2(x, y);
                            if jump >= 1.0 {
                                if *amt < JUMP_AMT - 1 {
                                    let directions = [vec2(1.0, 0.0), vec2(-1.0, 0.0)];
                                    for direction in directions {
                                        projectiles.push(Projectile::new(6, self.pos, direction));
                                    }
                                }
                                if *amt >= JUMP_AMT - 1 {
                                    *amt += 1;
                                }
                                *phase += 1;
                                self.time = 0.0;
                            }
                        } else {
                            self.time = 0.0;
                            *phase = 1;
                            *amt += 1;
                            *src = vec2(self.pos.x, self.spawn.y);
                            *target = vec2(player.pos.x, self.spawn.y);
                            if *amt >= JUMP_AMT - 1 {
                                *target = vec2(self.spawn.x, pipe_pos);
                            }
                        }
                    }

                    if *amt < JUMP_AMT - 1 {
                        let pos = *target - vec2(26.0, 0.0);
                        draw_texture(&assets.henry_target, pos.x, pos.y, WHITE);
                    }

                    animation = 2 + *phase;
                    if *amt >= JUMP_AMT {
                        self.time = 0.0;
                        self.state = State::LandOnPipe;
                    }
                }
                State::Fireballs(amt, positions) => {
                    animation = 1;
                    loop_animation = false;

                    const WAIT_TIME: f32 = 1.0;

                    if self.time <= PIPE_MOVE_TIME {
                        let amt = self.time;
                        self.pos.y = self.spawn.y.lerp(self.spawn.y - 4.0 * 8.0, amt);
                    }
                    let mut fireball_time = self.time;
                    let mut fireball_animation = 0;
                    let mut fall_amt =
                        (self.time - (PIPE_MOVE_TIME + WAIT_TIME)) / FIREBALL_FALL_TIME;
                    if self.time > PIPE_MOVE_TIME + WAIT_TIME + FIREBALL_FALL_TIME {
                        if player.death.is_none() {
                            for position in positions.iter() {
                                if (player.pos.x - *position).abs() < 16.0 {
                                    player.death = Some((0.0, 3, true));
                                    break;
                                }
                            }
                        }
                        fireball_animation = 1;
                        fireball_time = self.time - PIPE_MOVE_TIME + WAIT_TIME + FIREBALL_FALL_TIME;
                        fall_amt = 1.0;
                        let fireball_finish_time =
                            (assets.fireball.animations[1].total_length - 1) as f32 / 1000.0;
                        if self.time
                            >= PIPE_MOVE_TIME
                                + WAIT_TIME
                                + FIREBALL_FALL_TIME
                                + fireball_finish_time
                        {
                            fireball_time = fireball_finish_time;
                            self.time = PIPE_MOVE_TIME;
                            *amt += 1;
                            populate_fireball_positions(
                                positions,
                                player.pos,
                                left_target,
                                right_target,
                            );
                        }
                    }

                    let fireball_pos = level.roof_height.lerp(self.spawn.y, fall_amt);
                    let texture = assets.fireball.animations[fireball_animation]
                        .get_at_time((fireball_time * 1000.0) as u32);
                    for position in positions.iter() {
                        draw_texture(
                            assets
                                .fireking_target
                                .get_at_time((self.time * 1000.0) as u32),
                            *position - 12.0,
                            self.spawn.y - 2.0,
                            WHITE,
                        );
                        draw_texture(texture, *position - 26.0, fireball_pos - 38.0, WHITE);
                    }
                    if *amt >= FIREBALL_WAVE_AMT {
                        self.state = State::Jump(
                            0,
                            0,
                            vec2(self.pos.x, self.spawn.y - 4.0 * 8.0),
                            vec2(player.pos.x, self.spawn.y),
                        );
                        animation = 2;
                        self.time = 0.0;
                    }
                }
            }
        }

        let draw_pos = self.pos - vec2(30.0, 52.0);
        if !dead && self.activated > 0.0 {
            for projectile in projectiles {
                if projectile.friendly
                    && (draw_pos.y + 23.0..draw_pos.y + 60.0).contains(&projectile.pos.y)
                    && (self.pos.x - 8.0..self.pos.x + 8.0).contains(&projectile.pos.x)
                {
                    projectile.dead = true;
                    self.health = self.health.saturating_sub(1);
                    self.blood_effects
                        .push((projectile.pos, 0.0, projectile.direction.x > 0.0));
                }
            }
        }

        // draw pipe
        draw_texture_ex(
            &assets.fireking_pipe,
            self.spawn.x - 18.0,
            pipe_pos + 6.0,
            WHITE,
            DrawTextureParams {
                flip_x: false,
                ..Default::default()
            },
        );

        // draw fireking (both body and crown)

        let animation_time = if loop_animation {
            self.time
        } else {
            self.time
                .min((assets.fireking.animations[animation].total_length - 1) as f32 / 1000.0)
        };
        let mut textures = vec![
            assets.fireking.animations[animation].get_at_time((animation_time * 1000.0) as u32),
        ];
        if !dead {
            textures.push(assets.fire_crown.get_at_time((self.time * 1000.0) as u32));
        }
        for t in textures {
            draw_texture_ex(
                t,
                draw_pos.x,
                draw_pos.y,
                WHITE,
                DrawTextureParams {
                    flip_x: flipped,
                    ..Default::default()
                },
            );
        }
        if self.health <= 0 && !dead && self.pos.y + 1.0 >= self.spawn.y {
            self.state = State::Death(pipe_pos);
            self.time = 0.0;
        }

        if self.activated > 0.0 {
            let lavafall_pos = level.find_marker(3);
            draw_texture(
                assets.lavafall.get_at_time((self.time * 1000.0) as u32),
                lavafall_pos.x + 2.0,
                lavafall_pos.y - 3.0,
                WHITE,
            );
        }
        self.blood_effects.retain_mut(|(pos, time, facing_right)| {
            let anim = &assets.blood;
            *time += delta_time;
            draw_texture_ex(
                anim.get_at_time((*time * 1000.0) as u32),
                pos.x - 8.0,
                pos.y - 8.0,
                WHITE,
                DrawTextureParams {
                    flip_x: *facing_right,
                    ..Default::default()
                },
            );
            *time * 1000.0 < anim.total_length as f32
        });
        //draw_rectangle(self.pos.x, self.pos.y, -32.0, 2.0, GREEN);
        //draw_rectangle(self.pos.x, draw_pos.y+23.0, 2.0, draw_pos.y+60.0-(draw_pos.y+23.0), BLUE);
    }
}
