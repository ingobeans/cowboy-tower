use macroquad::prelude::*;

use crate::{
    assets::{Assets, Level},
    player::Player,
    projectiles::Projectile,
};

pub trait Boss {
    #[allow(unused_variables)]
    fn update(
        &mut self,
        assets: &Assets,
        delta_time: f32,
        level: &Level,
        projectiles: &mut Vec<Projectile>,
        player: &mut Player,
    ) {
    }
}

pub fn new_boss(index: usize, pos: Vec2) -> Box<dyn Boss> {
    match index {
        0 => Box::new(Henry {
            pos,
            spawn: pos,
            health: 12,
            state: HenryState::Idle,
            time: 0.0,
            activated: 0.0,
            blood_effects: Vec::new(),
        }),
        _ => panic!(),
    }
}

enum HenryState {
    Idle,
    /// - amount of jumps completed,
    /// - jump source
    /// - jump land target
    Jumping(u8, Vec2, Vec2),
    /// - amount of barrels thrown,
    /// - whether he is on left or right side of arena (true = left).
    ThrowingBarrels(u8, bool),
    Death,
}
pub struct Henry {
    pos: Vec2,
    spawn: Vec2,
    health: u8,
    state: HenryState,
    time: f32,
    blood_effects: Vec<(Vec2, f32, bool)>,
    activated: f32,
}
impl Boss for Henry {
    fn update(
        &mut self,
        assets: &Assets,
        delta_time: f32,
        level: &Level,
        projectiles: &mut Vec<Projectile>,
        player: &mut Player,
    ) {
        let mut pole_anim_time = None;
        let pole_anim = &assets.pole;
        let mut activate = false;
        if self.activated > 0.0 {
            if (self.activated + delta_time) * 1000.0 < pole_anim.total_length as f32 {
                self.activated += delta_time;
            }
            pole_anim_time = Some(self.activated);
        } else if let Some(dialogue) = &mut player.active_dialoge {
            if dialogue.2 {
                player.active_dialoge = None;
                activate = true;
            }
        } else if player.pos.distance(level.find_marker(4)) <= 8.0 {
            if player.has_restarted_level {
                activate = true;
            } else {
                player.active_dialoge =
                    Some(("Hm. A puny little cowboy.\nYou will be crushed.", 0, false))
            }
        }
        if activate {
            self.activated = delta_time;
            player.in_boss_battle = true;
        }
        let dead = matches!(self.state, HenryState::Death);

        if dead {
            if self.time > 1.5 {
                let time = self.time - 1.5;
                let max = (pole_anim.total_length - 1) as f32 / 1000.0;
                if time >= max {
                    player.in_boss_battle = false;
                    pole_anim_time = None;
                } else {
                    pole_anim_time = Some(max - time);
                }
            }
        }

        if let Some(time) = pole_anim_time {
            for pos in [level.find_marker(2), level.find_marker(3)] {
                let t = pole_anim.get_at_time((time * 1000.0) as u32);
                draw_texture(t, pos.x, pos.y - t.height() + 4.0, WHITE);
            }
        }

        self.time += delta_time;

        // get general state info
        let animation = match &self.state {
            HenryState::Idle => 0,
            HenryState::Jumping(..) => 1,
            HenryState::ThrowingBarrels(_, _) => 2,
            HenryState::Death => 3,
        };
        let flipped = match &self.state {
            HenryState::Idle => self.pos.x < player.pos.x,
            HenryState::Jumping(.., src, dest) => dest.x < src.x,
            HenryState::ThrowingBarrels(_, side) => *side,
            HenryState::Death => false,
        };
        let loop_animation = matches!(self.state, HenryState::Idle);

        // update states
        match &mut self.state {
            HenryState::Death => {}
            HenryState::Idle => {
                if self.time >= 2.0 {
                    if self.activated > 0.0 {
                        self.state =
                            HenryState::Jumping(0, self.pos, vec2(player.pos.x, self.spawn.y));
                    }
                    self.time = 0.0;
                }
            }
            HenryState::Jumping(amt, src, dest) => {
                const JUMP_COMPLETE_SPEED: f32 = 1.25;
                const JUMP_HEIGHT: f32 = 78.0;
                const JUMP_AMT: u8 = 5;
                let time_delta = self.time - 0.220;
                let in_jump = time_delta.is_sign_positive();

                if time_delta > JUMP_COMPLETE_SPEED {
                    self.time = 0.0;
                    *amt += 1;
                    self.pos.y = self.spawn.y;
                    src.x = self.pos.x;
                    dest.x = player.pos.x;
                    if *amt >= JUMP_AMT - 1 {
                        let left_marker = level.find_marker(0);
                        let right_marker = level.find_marker(1);
                        if *amt == JUMP_AMT - 1 {
                            let left_side = (player.pos.x - left_marker.x).abs()
                                > (player.pos.x - right_marker.x).abs();
                            if left_side {
                                dest.x = left_marker.x;
                            } else {
                                dest.x = right_marker.x;
                            }
                        } else {
                            let left_side = (self.pos.x - left_marker.x).abs()
                                < (self.pos.x - right_marker.x).abs();
                            self.state = HenryState::ThrowingBarrels(0, left_side);
                        }
                    }
                } else if in_jump {
                    let jump = time_delta / JUMP_COMPLETE_SPEED;
                    let y_amt = -4.0 * jump.powi(2) + 4.0 * jump;
                    let y = src.y - y_amt * JUMP_HEIGHT;

                    let x_amt = (jump.powi(2) + jump) * 0.5;
                    let x = src.x.lerp(dest.x, x_amt);

                    self.pos = vec2(x, y);

                    // draw target indicator
                    if *amt < JUMP_AMT - 1 {
                        let pos = *dest - vec2(26.0, 0.0);
                        draw_texture(&assets.henry_target, pos.x, pos.y, WHITE);
                    }
                }
            }
            HenryState::ThrowingBarrels(amt, left_side) => {
                const BARRELS_TO_THROW: u8 = 3;

                let time_delta =
                    self.time - assets.henry.animations[animation].total_length as f32 / 1000.0;
                if time_delta >= -0.300 {
                    self.time = 0.0;
                    let pos = self.pos + vec2(16.0, 0.0) * if *left_side { 1.0 } else { -1.0 };
                    let dir = if *left_side {
                        vec2(1.0, 0.0)
                    } else {
                        vec2(-1.0, 0.0)
                    };
                    let projectile = Projectile::new(5, pos, dir);
                    projectiles.push(projectile);
                    *amt += 1;
                    if *amt >= BARRELS_TO_THROW {
                        self.state = HenryState::Idle;
                        self.time = 0.0;
                    }
                }
            }
        }

        if !dead && player.death.is_none() && (player.pos + 4.0).distance(self.pos) <= 16.0 {
            player.death = Some((0.0, 3, true));
        }

        if !dead && self.activated > 0.0 {
            for projectile in projectiles {
                if projectile.friendly && self.pos.distance(projectile.pos) <= 16.0 {
                    self.health = self.health.saturating_sub(1);
                    projectile.dead = true;
                    self.blood_effects.push((
                        projectile.pos.move_towards(self.pos, 4.0),
                        0.0,
                        projectile.direction.x > 0.0,
                    ));
                    break;
                }
            }
        }
        if !dead && self.health == 0 && (self.pos.y - self.spawn.y).abs() < 0.1 {
            self.time = 0.0;
            self.state = HenryState::Death;
        }

        let animation_time = if loop_animation {
            self.time
        } else {
            self.time
                .min((assets.henry.animations[animation].total_length - 1) as f32 / 1000.0)
        };

        let draw_pos = self.pos - vec2(30.0, 52.0);
        draw_texture_ex(
            assets.henry.animations[animation].get_at_time((animation_time * 1000.0) as u32),
            draw_pos.x,
            draw_pos.y,
            WHITE,
            DrawTextureParams {
                flip_x: flipped,
                ..Default::default()
            },
        );

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
        //draw_rectangle_lines(self.pos.x.floor(), self.pos.y.floor(), 8.0, 8.0, 1.0, GREEN);
    }
}
