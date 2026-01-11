use macroquad::prelude::*;

use crate::{
    assets::{Assets, Level},
    bosses::Boss,
    player::Player,
    projectiles::Projectile,
};

enum State {
    Idle,
    /// - Count of waves
    /// - Current target positions
    Fireballs(u8, Vec<f32>),
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
            health: 12,
            state: State::Idle,
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
        let dialogue_messages = &["I see you have defeated\nHenry.", "But now you shall burn."];
        const FIREBALL_FALL_TIME: f32 = 1.0;
        const FIREBALL_AMT: u8 = 10;

        let loop_animation;
        let animation;

        let left_target = level.find_marker(0);
        let right_target = level.find_marker(1);

        self.time += delta_time;
        if self.activated <= 0.0 {
            loop_animation = true;
            animation = 0;

            if let Some(dialogue) = &mut player.active_dialogue {
                if dialogue.closed {
                    if self.dialogue_id + 1 >= dialogue_messages.len() {
                        player.show_cinematic_bars();
                        self.activated = delta_time;
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
                State::Idle => {
                    animation = 0;
                    loop_animation = true;
                    if self.time >= 1.0 {
                        let mut positions = Vec::new();
                        for _ in 0..FIREBALL_AMT {
                            positions.push(rand::gen_range(left_target.x, right_target.x));
                        }
                        *positions.last_mut().unwrap() = player.pos.x;
                        self.state = State::Fireballs(0, positions);
                        self.time = 0.0;
                    }
                }
                State::Fireballs(amt, positions) => {
                    animation = 1;
                    loop_animation = false;

                    const WAIT_TIME: f32 = 1.0;
                    const PIPE_MOVE_TIME: f32 = 1.0;

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
                            for item in positions.iter_mut() {
                                *item = rand::gen_range(left_target.x, right_target.x);
                            }
                            *positions.last_mut().unwrap() = player.pos.x;
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
                }
            }
        }

        let draw_pos = self.pos - vec2(30.0, 52.0);
        for projectile in projectiles {
            if (draw_pos.y + 23.0..draw_pos.y + 60.0).contains(&projectile.pos.y)
                && projectile.pos.x < self.pos.x + 8.0
            {
                projectile.dead = true;
                self.health = self.health.saturating_sub(1);
                self.blood_effects
                    .push((projectile.pos, 0.0, projectile.direction.x > 0.0));
            }
        }

        // draw pipe
        draw_texture_ex(
            &assets.fireking_pipe,
            self.spawn.x - 18.0,
            self.pos.y + 6.0,
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
        let textures = [
            assets.fireking.animations[animation].get_at_time((animation_time * 1000.0) as u32),
            assets.fire_crown.get_at_time((self.time * 1000.0) as u32),
        ];
        for t in textures {
            draw_texture_ex(
                t,
                draw_pos.x,
                draw_pos.y,
                WHITE,
                DrawTextureParams {
                    flip_x: false,
                    ..Default::default()
                },
            );
        }

        if self.activated > 0.0 {
            let lavafall_pos = level.find_marker(3);
            draw_texture_ex(
                assets.lavafall.get_at_time((self.time * 1000.0) as u32),
                lavafall_pos.x + 2.0,
                lavafall_pos.y - 3.0,
                WHITE,
                DrawTextureParams {
                    flip_x: false,
                    ..Default::default()
                },
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
