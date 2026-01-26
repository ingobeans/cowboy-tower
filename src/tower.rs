use macroquad::prelude::*;

use crate::assets::Assets;
use crate::utils::*;

fn calculate_world_heights(assets: &Assets) -> Vec<(f32, f32)> {
    let mut total = -3.0 * 8.0;
    let mut worlds = vec![(0.0, total)];
    let mut last_world = 0;
    for level in assets.levels.iter() {
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

pub struct WorldManager {
    pub world_heights: Vec<(f32, f32)>,
    pub world_colors: Vec<(Color, Color, Color)>,
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
        }
    }
    pub fn draw_tower(&self, y: f32, assets: &Assets, level_index: usize) {
        let level = &assets.levels[level_index];

        for world_index in 0..=2 {
            let (wall_color, border_color, _) = self.world_colors[world_index];
            let min_x = if level_index > 0 {
                level.min_pos.x
            } else {
                level.find_marker(0).x
            };
            let max_x = if level_index > 0 {
                level.max_pos.x
            } else {
                level.find_marker(1).x - 16.0 * 8.0 + 8.0
            };

            let offset = if level_index == 0 {
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
