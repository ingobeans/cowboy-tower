use macroquad::prelude::*;

mod fireking;
mod henry;

use crate::{
    assets::{Assets, Level},
    bosses::fireking::Fireking,
    bosses::henry::Henry,
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
        0 => Box::new(Henry::new(pos)),
        1 => Box::new(Fireking::new(pos)),
        _ => panic!(),
    }
}
