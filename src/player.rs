use macroquad::prelude::*;

use crate::{
    Projectile,
    assets::{Assets, Level},
    utils::*,
};

fn ceil_g(a: f32) -> f32 {
    if a < 0.0 { a.floor() } else { a.ceil() }
}

pub struct Player {
    pub pos: Vec2,
    pub camera_pos: Vec2,
    pub velocity: Vec2,
    pub on_ground: bool,
    pub facing_left: bool,
    pub moving: bool,
    pub time: f32,
    /// If player isnt actively shooting a projectile, this is 0.
    /// Otherwise it will be the time for the shoot animation.
    pub shooting: f32,
}
impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            camera_pos: pos,
            velocity: Vec2::ZERO,
            on_ground: false,
            facing_left: false,
            moving: false,
            time: 0.0,
            shooting: 0.0,
        }
    }
    pub fn update(&mut self, delta_time: f32, world: &Level, projectiles: &mut Vec<Projectile>) {
        const MOVE_SPEED: f32 = 101.0;
        const MOVE_ACCELERATION: f32 = 22.0;
        const GRAVITY: f32 = 9.8 * 75.0;
        const JUMP_FORCE: f32 = 160.0;

        let input = get_input_axis();
        self.time += delta_time;
        if self.shooting > 0.0 {
            self.shooting += delta_time;
        } else if is_mouse_button_pressed(MouseButton::Left) {
            self.shooting += delta_time;
            projectiles.push(Projectile {
                pos: self.pos
                    + if self.facing_left {
                        vec2(-8.0, 0.0)
                    } else {
                        vec2(8.0, 0.0)
                    },
                direction: vec2(if self.facing_left { -1.0 } else { 1.0 }, 0.0),
                sprite: 0,
                friendly: true,
                dead: false,
            });
        }

        self.velocity.x = self
            .velocity
            .x
            .lerp(input.x * MOVE_SPEED, delta_time * MOVE_ACCELERATION);
        self.velocity.y += GRAVITY * delta_time;

        self.moving = input.x != 0.0;
        if self.moving {
            self.facing_left = input.x.is_sign_negative();
        }

        if self.on_ground && is_key_pressed(KeyCode::Space) {
            self.velocity.y = -JUMP_FORCE;
        }

        (self.pos, self.on_ground) =
            update_physicsbody(self.pos, &mut self.velocity, delta_time, world, true);
        self.camera_pos.x = self.pos.x.max(world.min_pos.x + SCREEN_WIDTH / 2.0 - 64.0);
        let target = self.pos.y - 22.0;
        if self.camera_pos.y < target {
            self.camera_pos.y = target;
        } else {
            let delta = self.camera_pos.y - target;
            let max_delta = 3.5 * 8.0;
            if delta.abs() > max_delta {
                self.camera_pos.y = max_delta * if delta < 0.0 { -1.0 } else { 1.0 } + target;
            }
        }
    }
    pub fn draw(&mut self, assets: &Assets) {
        if self.shooting * 1000.0 >= assets.torso.animations[1].total_length as f32 {
            self.shooting = 0.0;
        }

        // draw legs and torso textures
        let legs = assets.legs.animations[if self.moving { 1 } else { 0 }]
            .get_at_time((self.time * 1000.0) as u32);
        let torso = assets.torso.animations[if self.shooting > 0.0 { 1 } else { 0 }]
            .get_at_time((self.shooting * 1000.0) as u32);
        for texture in [legs, torso] {
            draw_texture_ex(
                texture,
                self.pos.x.floor() - texture.width() / 2.0 + 4.0,
                self.pos.y.floor() - 8.0,
                WHITE,
                DrawTextureParams {
                    flip_x: self.facing_left,
                    ..Default::default()
                },
            );
        }
    }
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
        let tile = world.get_tile((tx) as i16, (ty) as i16)[1];
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
        let tile = world.get_tile((tx) as i16, (ty) as i16)[1];
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
