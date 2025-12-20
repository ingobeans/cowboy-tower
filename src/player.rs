use macroquad::prelude::*;

use crate::assets::Level;

fn ceil_g(a: f32) -> f32 {
    if a < 0.0 { a.floor() } else { a.ceil() }
}

pub fn update_physicsbody(
    pos: Vec2,
    velocity: &mut Vec2,
    delta_time: f32,
    world: &Level,
    tall: bool,
) -> (Vec2, bool) {
    let mut grounded = false;
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
        tiles_y.push((tile_x.trunc(), (new.y / 8.0).trunc() - 1.0));
        tiles_y.push((ceil_g(tile_x), (new.y / 8.0).trunc() - 1.0));
    }

    for (tx, ty) in tiles_y {
        let tile = world.get_tile((tx / 2.0) as usize, (ty / 2.0) as usize)[0];
        if tile != 0 {
            let c = if velocity.y < 0.0 {
                tile_y.floor() * 8.0
            } else {
                grounded = true;
                tile_y.ceil() * 8.0
            };
            new.y = c;
            velocity.y = 0.0;
            break;
        }
    }
    let mut tiles_x = vec![
        ((new.x / 8.0).trunc(), ceil_g(new.y / 8.0)),
        (ceil_g(new.x / 8.0), ceil_g(new.y / 8.0)),
        (ceil_g(new.x / 8.0), (new.y / 8.0).trunc()),
        ((new.x / 8.0).trunc(), (new.y / 8.0).trunc()),
    ];
    if tall {
        tiles_x.push(((new.x / 8.0).trunc(), (new.y / 8.0).trunc() - 1.0));
        tiles_x.push((ceil_g(new.x / 8.0), (new.y / 8.0).trunc() - 1.0));
    }

    for (tx, ty) in tiles_x {
        let tile = world.get_tile((tx / 2.0) as usize, (ty / 2.0) as usize)[0];
        if tile != 0 {
            let c = if velocity.x < 0.0 {
                tile_x.floor() * 8.0
            } else {
                tile_x.ceil() * 8.0
            };
            new.x = c;
            velocity.x = 0.0;
            break;
        }
    }
    (new, grounded)
}
