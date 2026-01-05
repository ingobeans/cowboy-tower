use macroquad::prelude::*;

pub struct Projectile {
    pub pos: Vec2,
    pub direction: Vec2,
    pub type_index: usize,
    pub time: f32,
    /// Is projectile fired by the player?
    pub friendly: bool,
    /// True when projectile hits an enemy, marker to show that it should be destroyed.
    pub dead: bool,
}
impl Projectile {
    pub fn new(type_index: usize, pos: Vec2, direction: Vec2) -> Self {
        Self {
            pos,
            direction: direction * Self::base_speed(type_index),
            type_index,
            time: 0.0,
            friendly: type_index == 0,
            dead: false,
        }
    }
    pub fn is_ray(&self) -> bool {
        match self.type_index {
            4 => true,
            _ => false,
        }
    }
    pub fn shoot_offset(type_index: usize) -> bool {
        match type_index {
            3 | 4 => false,
            _ => true,
        }
    }
    pub fn base_speed(type_index: usize) -> f32 {
        match type_index {
            1 | 2 => 128.0 * 0.8,
            3 | 4 => 0.0,
            _ => 128.0,
        }
    }
    pub fn is_physics_based(&self) -> bool {
        match &self.type_index {
            2 => true,
            _ => false,
        }
    }
    pub fn get_payload(&self) -> Option<Projectile> {
        match &self.type_index {
            2 => Some(Projectile::new(3, self.pos, Vec2::ZERO)),
            _ => None,
        }
    }
    pub fn get_collision_size(&self) -> f32 {
        match &self.type_index {
            3 => 17.0,
            _ => 8.0,
        }
    }
    pub fn can_kill(&self) -> bool {
        match &self.type_index {
            2 => false,
            _ => true,
        }
    }
    pub fn should_die_on_kill(&self) -> bool {
        match &self.type_index {
            3 | 4 => false,
            _ => true,
        }
    }
    pub fn player_death_animation(&self) -> usize {
        match &self.type_index {
            4 => 2,
            _ => 0,
        }
    }
    pub fn get_lifetime(&self) -> f32 {
        match &self.type_index {
            2 => 1.0,
            3 => 0.5,
            4 => 1.0,
            _ => 0.0,
        }
    }
}
