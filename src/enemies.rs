use crate::assets::AnimationsGroup;
use macroquad::prelude::*;
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct LevelEnemyData {
    pub pos: Vec2,
    pub ty: &'static EnemyType,
    pub attack_delay: f32,
    pub path_index: Option<(usize, usize)>,
}

#[allow(dead_code)]
pub enum MovementType {
    None,
    Wander,
    FollowPath,
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
}
pub static ENEMIES: LazyLock<Vec<EnemyType>> = LazyLock::new(|| {
    vec![
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/bandit.ase")),
            movement_type: MovementType::Wander,
            attack_type: AttackType::Shoot(1),
            attack_delay: 1.5,
        },
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/bandit2.ase")),
            movement_type: MovementType::None,
            attack_type: AttackType::Shoot(1),
            attack_delay: 2.0,
        },
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/demo_bandit.ase")),
            movement_type: MovementType::Wander,
            attack_type: AttackType::ShootAfter(2),
            attack_delay: 2.0,
        },
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/laser.ase")),
            movement_type: MovementType::None,
            attack_type: AttackType::ShootAfter(4),
            attack_delay: 2.0,
        },
        EnemyType {
            animation: AnimationsGroup::from_file(include_bytes!("../assets/bat.ase")),
            movement_type: MovementType::FollowPath,
            attack_type: AttackType::Melee,
            attack_delay: 0.0,
        },
    ]
});
