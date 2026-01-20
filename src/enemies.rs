use crate::{
    assets::{AnimationsGroup, Assets, Level},
    player::{Player, update_physicsbody},
    projectiles::Projectile,
    utils::{DEBUG_FLAGS, draw_cross},
};
use macroquad::prelude::*;
use std::{f32::consts::PI, sync::LazyLock};

pub struct Enemy {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub ty: &'static EnemyType,
    pub path_index: Option<(usize, usize)>,
    pub time: f32,
    /// Used for attack type ShootAfter
    pub has_attacked: bool,
    pub attack_time: f32,
    /// Set to zero when alive. On death, tracks death animation time
    pub death_frames: f32,
    /// Random seed for each enemy, used for random-esque movement and behaviour
    pub wibble_wobble: f32,
    pub waiting_to_spawn: f32,
}
impl Enemy {
    pub fn update(
        &mut self,
        player: &mut Player,
        projectiles: &mut Vec<Projectile>,
        assets: &Assets,
        level: &Level,
        delta_time: f32,
    ) -> bool {
        self.time += delta_time;

        let mut force_moving_animation = false;
        if self.death_frames > 0.0 {
            self.death_frames += delta_time;
            self.time = 0.0;
        } else if self.waiting_to_spawn == f32::INFINITY {
            if self.pos.distance(player.pos) < 128.0 {
                self.waiting_to_spawn =
                    self.ty.animation.animations[self.ty.animation.tag_names["spawning"]]
                        .total_length as f32
                        / 1000.0;
            }
        } else if self.waiting_to_spawn > 0.0 {
            self.waiting_to_spawn -= delta_time;
        } else {
            match self.ty.movement_type {
                MovementType::None => {}
                MovementType::FollowPath => {
                    force_moving_animation = true;
                    let (path_index, path_tile_index) = self.path_index.unwrap();
                    let path = &level.enemy_paths[path_index];
                    let time_per_tile = 1.0 / self.ty.speed;
                    let path_time = path.len() as f32 * time_per_tile;
                    let value = (self.time + path_tile_index as f32 * time_per_tile) % path_time
                        / time_per_tile;
                    let value_index = value.floor();

                    let current = path[value_index as usize];
                    let next = path[(value_index as usize + 1) % path.len()];
                    let amt_between = value - value_index;
                    self.pos = current.lerp(next, amt_between);
                }
                MovementType::Wander => {
                    let value = self.time + self.wibble_wobble;
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
                    self.velocity.x = value * self.ty.speed;
                }
                MovementType::Chase => {
                    let mut direction = self.pos - player.pos;
                    direction.y = 0.0;
                    if direction.x.abs() < 1.0 {
                        direction.x = 0.0;
                    }
                    let move_dir = -direction.normalize_or_zero().x;
                    self.velocity.x = move_dir * self.ty.speed;
                }
            }
            if self.attack_time <= 0.0 {
                if player.death.is_none() {
                    self.attack_time += delta_time;
                    match self.ty.attack_type {
                        AttackType::None => {
                            self.attack_time = 0.0;
                        }
                        AttackType::Melee => {
                            self.attack_time = 0.0;
                            if (player.pos + 4.0).distance(self.pos + 4.0) < 5.0 {
                                player.death = Some((0.0, 0, true))
                            }
                        }
                        AttackType::ShootAfter(_) => {}
                        AttackType::Shoot(sprite) => {
                            let pos = if Projectile::shoot_offset(sprite) {
                                self.pos
                                    + if self.pos.x > player.pos.x {
                                        vec2(-8.0, 0.0)
                                    } else {
                                        vec2(8.0, 0.0)
                                    }
                                    + vec2(4.0, 0.0)
                            } else {
                                self.pos
                            };
                            projectiles.push(Projectile::new(
                                sprite,
                                pos,
                                vec2(if self.pos.x > player.pos.x { -1.0 } else { 1.0 }, 0.0),
                            ));
                        }
                    }
                }
            } else {
                self.attack_time += delta_time;
                let delta = self.attack_time * 1000.0
                    - self.ty.animation.get_by_name("attack").total_length as f32;
                if delta >= 0.0
                    && !self.has_attacked
                    && let AttackType::ShootAfter(sprite) = self.ty.attack_type
                {
                    let pos = if Projectile::shoot_offset(sprite) {
                        self.pos
                            + if self.pos.x > player.pos.x {
                                vec2(-8.0, 0.0)
                            } else {
                                vec2(8.0, 0.0)
                            }
                            + vec2(4.0, 0.0)
                    } else {
                        self.pos
                    };
                    projectiles.push(Projectile::new(
                        sprite,
                        pos,
                        vec2(if self.pos.x > player.pos.x { -1.0 } else { 1.0 }, 0.0),
                    ));
                    self.has_attacked = true;
                }
                if delta >= self.ty.attack_delay * 1000.0 {
                    self.attack_time = 0.0;
                    self.has_attacked = false;
                }
            }
            (self.pos, _, _) =
                update_physicsbody(self.pos, &mut self.velocity, delta_time, level, true, false);
        }
        let rotation = if self.death_frames <= 0.0 {
            0.0
        } else {
            (self.death_frames * 1000.0 * 2.0 / assets.blood.total_length as f32).min(1.0)
                * (PI / 4.0)
                * (if self.pos.x > player.pos.x { 1.0 } else { -1.0 })
        };
        let (animation_id, time) = if self.waiting_to_spawn == f32::INFINITY {
            if !self.ty.animation.tag_names.contains_key("unspawned") {
                return true;
            }
            (self.ty.animation.tag_names["unspawned"], 0.0)
        } else if self.waiting_to_spawn > 0.0 {
            let total = self.ty.animation.animations[self.ty.animation.tag_names["spawning"]]
                .total_length as f32
                / 1000.0;
            (
                self.ty.animation.tag_names["spawning"],
                total - self.waiting_to_spawn,
            )
        } else if self.attack_time > 0.0
            && self.attack_time * 1000.0
                < self.ty.animation.get_by_name("attack").total_length as f32
        {
            (self.ty.animation.tag_names["attack"], self.attack_time)
        } else {
            (
                if force_moving_animation || self.velocity.x.abs() > 5.0 {
                    1
                } else {
                    0
                },
                self.time,
            )
        };
        draw_texture_ex(
            self.ty.animation.animations[animation_id].get_at_time((time * 1000.0) as u32),
            self.pos.x.floor() - 8.0,
            self.pos.y.floor() - 8.0,
            WHITE,
            DrawTextureParams {
                flip_x: self.pos.x > player.pos.x,
                rotation,
                ..Default::default()
            },
        );
        if DEBUG_FLAGS.centres {
            draw_cross(self.pos.x, self.pos.y, RED);
        }
        if self.death_frames <= 0.0 {
            let mut hit_by_projectile = false;
            for projectile in projectiles.iter_mut() {
                if projectile.friendly
                    && projectile.can_kill()
                    && ((projectile.pos.x - 4.0)..(projectile.pos.x + 4.0))
                        .contains(&(self.pos.x + 4.0))
                    && ((projectile.pos.y - 8.0)..(projectile.pos.y + 4.0)).contains(&self.pos.y)
                {
                    projectile.dead |= projectile.should_die_on_kill();
                    hit_by_projectile = true;
                    break;
                }
            }
            if hit_by_projectile {
                self.death_frames += delta_time;
            }
            true
        } else {
            draw_texture_ex(
                assets
                    .blood
                    .get_at_time((self.death_frames * 1000.0) as u32),
                self.pos.x.floor() - 4.0,
                self.pos.y.floor() - 8.0,
                WHITE,
                DrawTextureParams {
                    flip_x: self.pos.x > player.pos.x,
                    ..Default::default()
                },
            );
            self.death_frames * 1000.0 <= assets.blood.total_length as f32
        }
    }
}

#[derive(Clone, Copy)]
pub struct LevelEnemyData {
    pub pos: Vec2,
    pub ty: &'static EnemyType,
    pub attack_delay: f32,
    pub path_index: Option<(usize, usize)>,
    pub spawner: f32,
}

#[allow(dead_code)]
pub enum MovementType {
    None,
    Wander,
    FollowPath,
    Chase,
}

#[allow(dead_code)]
pub enum AttackType {
    None,
    Shoot(usize),
    /// Like shoot, but projectile is fired after animation is completed
    ShootAfter(usize),
    Melee,
}

pub struct EnemyType {
    pub animation: AnimationsGroup,
    pub movement_type: MovementType,
    pub attack_type: AttackType,
    pub attack_delay: f32,
    pub speed: f32,
}
pub static ENEMIES: LazyLock<Vec<EnemyType>> = LazyLock::new(|| {
    vec![
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/bandit.ase")),
            movement_type: MovementType::Wander,
            speed: 16.0,
            attack_type: AttackType::Shoot(1),
            attack_delay: 1.5,
        },
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/bandit2.ase")),
            movement_type: MovementType::None,
            speed: 0.0,
            attack_type: AttackType::Shoot(1),
            attack_delay: 2.0,
        },
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/demo_bandit.ase")),
            movement_type: MovementType::Wander,
            speed: 16.0,
            attack_type: AttackType::ShootAfter(2),
            attack_delay: 2.0,
        },
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/laser.ase")),
            movement_type: MovementType::None,
            attack_type: AttackType::ShootAfter(4),
            speed: 0.0,
            attack_delay: 2.0,
        },
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/bat.ase")),
            movement_type: MovementType::FollowPath,
            attack_type: AttackType::Melee,
            speed: 5.0,
            attack_delay: 0.0,
        },
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/skeleton.ase")),
            movement_type: MovementType::Chase,
            attack_type: AttackType::Melee,
            speed: 32.0,
            attack_delay: 0.0,
        },
    ]
});
