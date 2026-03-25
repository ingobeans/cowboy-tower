use macroquad::prelude::*;

use crate::assets::{Assets, Level};
use crate::game::{get_elevator_pos, get_player_spawn};
use crate::utils::*;

fn calculate_world_heights(assets: &Assets) -> Vec<(f32, f32)> {
    let mut total = -3.0 * 8.0;
    let mut worlds = vec![(0.0, total)];
    let mut last_world = 0;
    for level in assets.levels.iter().skip(1) {
        let world = level.get_world_index();
        if world != last_world {
            last_world = world;
            worlds.push((2.0 * FLOOR_PADDING, total - 2.0 * FLOOR_PADDING));
        }
        let height = level.get_height();
        total += height + FLOOR_PADDING + 16.0;
        worlds.last_mut().unwrap().0 += height + FLOOR_PADDING + 8.0;
    }
    worlds.last_mut().unwrap().0 -= 2.0 * FLOOR_PADDING + 8.0;
    worlds
}

struct Cloud {
    pos: Vec2,
}

pub struct WorldManager {
    pub world_heights: Vec<(f32, f32)>,
    pub world_colors: Vec<(Color, Color, Color)>,
    clouds: Vec<Cloud>,
}
impl WorldManager {
    pub fn new(assets: &Assets) -> Self {
        Self {
            world_heights: calculate_world_heights(assets),
            world_colors: vec![
                (
                    Color::from_hex(0x300f0a),
                    Color::from_hex(0x5c320b),
                    Color::from_hex(0x3e2004),
                ),
                (
                    Color::from_hex(0x16100b),
                    Color::from_hex(0x927e6a),
                    Color::from_hex(0x392a1c),
                ),
                (BLACK, Color::from_hex(0x392a1c), Color::from_hex(0x36170c)),
            ],
            clouds: Vec::new(),
        }
    }
    pub fn create_clouds(&mut self, assets: &Assets, level_index: usize) {
        let level = &assets.levels[level_index];
        let w = (level.max_pos.x - level.min_pos.x).abs() + 16.0 * 8.0 + SCREEN_WIDTH * 2.0;
        let mut h = (level.roof_height - level.floor_height).abs();
        if level_index <= 1 {
            h -= 64.0;
        }
        let world = level.get_world_index();
        const WORLD_DENSITIES: &[f32] = &[0.0005, 0.0001, 0.0];
        let amt = (WORLD_DENSITIES[world as usize] * w * h) as u16;

        if !self.clouds.is_empty() {
            let x_offset = get_elevator_pos(assets, level_index - 1).x
                - get_player_spawn(assets, level_index).x
                + 13.0;
            for cloud in &mut self.clouds {
                cloud.pos.y += assets.levels[level_index].get_height() + FLOOR_PADDING + 16.0;
                cloud.pos.x -= x_offset;
            }
        }

        for _ in 0..amt {
            let x = rand::gen_range(0.0, w);
            let y = rand::gen_range(0.0, h);
            let pos = vec2(level.min_pos.x - SCREEN_WIDTH + x, level.roof_height + y);
            let cloud = Cloud { pos };
            self.clouds.push(cloud);
        }
    }
    pub fn draw_clouds(&mut self, assets: &Assets, level: &Level, delta_time: f32, paused: bool) {
        let offset = assets.clouds.frames[0].0.size() / 2.0;
        self.clouds.retain_mut(|cloud| {
            draw_texture(
                &assets.clouds.frames[0].0,
                cloud.pos.x - offset.x,
                cloud.pos.y - offset.y,
                WHITE,
            );
            if !paused {
                cloud.pos.x -= delta_time * 6.0;
                if cloud.pos.x <= level.min_pos.x - SCREEN_WIDTH {
                    cloud.pos.x = level.max_pos.x + SCREEN_WIDTH + 16.0 * 8.0;
                }
            }
            cloud.pos.y < level.floor_height + SCREEN_HEIGHT * 2.0
        });
    }
    pub fn draw_tower(&self, y: f32, assets: &Assets, level_index: usize) {
        let level = &assets.levels[level_index];

        for world_index in 0..=2 {
            let (wall_color, border_color, _) = self.world_colors[world_index];
            let min_x = if level_index > 1 {
                level.min_pos.x
            } else {
                level.find_marker(0).x
            };
            let max_x = if level_index > 1 {
                level.max_pos.x
            } else {
                level.find_marker(1).x - 16.0 * 8.0 + 8.0
            };

            let offset = if level_index <= 1 {
                level.find_marker(0).y - 3.0 * 8.0
            } else {
                0.0
            };

            draw_rectangle(
                min_x - 2.0,
                -self.world_heights[world_index].1 + y + offset,
                max_x - min_x + 16.0 * 8.0 + 4.0,
                -(self.world_heights[world_index].0 + FLOOR_PADDING) - offset,
                border_color,
            );
            draw_rectangle(
                min_x,
                -self.world_heights[world_index].1 + y + offset,
                max_x - min_x + 16.0 * 8.0,
                -(self.world_heights[world_index].0 + FLOOR_PADDING) - offset,
                wall_color,
            );
        }
    }
}
