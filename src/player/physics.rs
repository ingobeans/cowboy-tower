use macroquad::prelude::*;

use crate::{assets::Level, utils::*};

fn ceil_g(a: f32) -> f32 {
    if a < 0.0 { a.floor() } else { a.ceil() }
}

pub fn raycast(from: Vec2, to: Vec2, world: &Level) -> Option<Vec2> {
    const STEP_SIZE: f32 = 0.5;
    let mut pos = (from / 8.0).floor();
    let to = (to / 8.0).floor();
    let delta = (to - pos).normalize_or_zero();

    while pos.distance(to) > STEP_SIZE {
        pos += delta * STEP_SIZE;
        let tx = pos.x.floor();
        let ty = pos.y.floor();
        if world.get_tile(tx as _, ty as _)[1] > 0 {
            return Some(pos);
        }
    }
    None
}

pub fn update_physicsbody(
    pos: Vec2,
    velocity: &mut Vec2,
    delta_time: f32,
    level: &Level,
    tall: bool,
    enable_special_collisions: bool,
) -> (Vec2, bool, Option<u16>, Option<f32>) {
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

    fn handle_stair_walking(
        tile: u16,
        velocity: &mut Vec2,
        new: &mut Vec2,
        grounded: &mut bool,
        pos: Vec2,
        tx: f32,
        ty: f32,
    ) {
        if velocity.y < 0.0 {
            return;
        }

        // width is actually only 7.0 to leave 1px room on the apex of the stair,
        // so that theres a tiny bit of margin til the next tile and you dont have weird collisions

        let w = 7.0;
        let h = 8.0;

        if tile == 833 + 1 || tile == 832 + 1 {
            let stair_pos = vec2(tx, ty) * 8.0
                + if tile == 833 + 1 {
                    vec2(1.0, 0.0)
                } else {
                    Vec2::ZERO
                };

            let pos = if tile == 833 + 1 {
                pos
            } else {
                pos + vec2(8.0, 0.0)
            };

            // check if player is in stair x bound
            let in_bound = if tile == 833 + 1 {
                (stair_pos.x - 1.0..stair_pos.x + w).contains(&pos.x)
            } else {
                (stair_pos.x..stair_pos.x + w + 2.0).contains(&pos.x)
            };

            if in_bound {
                // player x relative to stair origin
                let player_stair_x = if tile == 833 + 1 {
                    stair_pos.x - pos.x
                } else {
                    -(stair_pos.x - pos.x) - w
                };
                let target_height = player_stair_x / w * h;
                new.y = stair_pos.y - (h + target_height.max(-8.0)).min(8.0);
                velocity.y = 0.0;
                *grounded = true;
            }
        }
    }

    for (tx, ty) in tiles_y {
        let tile_data = level.get_tile((tx) as i16, (ty) as i16);
        let mut tile = tile_data[1];
        if !grounded && tile > 0 && DEATH_TILES.contains(&(tile - 1)) {
            if ty * 8.0 > pos.y {
                touched_death_tile = Some(tile - 1);
            }
            continue;
        }
        if enable_special_collisions
            && tile == 0
            && level.get_tile(tx as i16, ty as i16)[3] == 864 + 1
        {
            tile = 1;
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
        } else {
            handle_stair_walking(tile_data[3], velocity, &mut new, &mut grounded, pos, tx, ty);
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

    let mut colliding_with_wall_climb_target = None;
    for (tx, ty) in tiles_x {
        let tile_data = level.get_tile((tx) as i16, (ty) as i16);
        let mut tile = tile_data[1];
        if tile > 0 && DEATH_TILES.contains(&(tile - 1)) {
            continue;
        }
        if enable_special_collisions
            && tile == 0
            && level.get_tile(tx as i16, ty as i16)[3] == 864 + 1
        {
            tile = 1;
        }
        if tile != 0 {
            if tile_data[3] == 512 + 1 || tile_data[3] == 513 + 1 {
                if velocity.y < 0.0 {
                    velocity.y = (velocity.y + 125.0 * delta_time).min(0.0);
                }
                colliding_with_wall_climb_target = Some(if tx * 8.0 < pos.x { -1.0 } else { 1.0 });
            }
            let c = if velocity.x < 0.0 {
                tile_x.floor() * 8.0
            } else {
                tile_x.ceil() * 8.0
            };
            new.x = c;
            velocity.x = 0.0;
            break;
        } else {
            handle_stair_walking(tile_data[3], velocity, &mut new, &mut grounded, pos, tx, ty);
        }
    }
    (
        new,
        grounded,
        touched_death_tile,
        colliding_with_wall_climb_target,
    )
}
