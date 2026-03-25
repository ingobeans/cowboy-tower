use std::f32::consts::PI;

use gamepads::Gamepads;
use macroquad::prelude::*;

use crate::{assets::Assets, data::SaveManager, utils::*};

fn inside(point: Vec2, box_start: Vec2, box_size: Vec2) -> bool {
    (box_start.x..box_start.x + box_size.x).contains(&(point.x + 1.0))
        && (box_start.y..box_start.y + box_size.y).contains(&(point.y + 1.0))
}

pub struct PauseMenu {
    pub selection_index: Option<u8>,
    prev_mouse_pos: Vec2,
    prev_input: Vec2,
}
impl PauseMenu {
    pub fn new() -> Self {
        Self {
            selection_index: None,
            prev_mouse_pos: Vec2::ZERO,
            prev_input: Vec2::ZERO,
        }
    }
    pub fn update(&mut self, gamepad_engine: &mut Gamepads, mouse: Vec2, origin: Vec2) {
        let start = vec2(18.0, 24.0);
        let size = vec2(44.0, 12.0);
        let gap = 3.0;

        let buttons_amt = 4;

        if mouse != self.prev_mouse_pos {
            // handle mouse hovering
            self.selection_index = None;
            for i in 0..buttons_amt {
                let p = start + vec2(0.0, size.y + gap) * i as f32;
                if inside(mouse, p, size) {
                    self.selection_index = Some(i)
                }
            }
            self.prev_mouse_pos = mouse;
        } else {
            // navigate with keyboard/controller
            let input = get_input_axis(gamepad_engine);
            let changed = input != self.prev_input;
            self.prev_input = input;

            if changed {
                if input.y == -1.0 {
                    self.selection_index = Some(
                        self.selection_index
                            .map(|f| f.wrapping_sub(1).min(buttons_amt - 1))
                            .unwrap_or_default(),
                    );
                } else if input.y == 1.0 {
                    self.selection_index = Some(
                        self.selection_index
                            .map(|f| (f + 1) % buttons_amt)
                            .unwrap_or_default(),
                    );
                }
            }
        }

        // draw selected button
        if let Some(selected) = &self.selection_index {
            let p = start + vec2(0.0, size.y + gap) * *selected as f32 + origin;
            draw_rectangle(p.x, p.y, size.x, size.y, Color::from_hex(0x300f0a));
        }
    }
}

pub struct MainMenu {
    camera: Camera3D,
    time: f32,
    button_index: Option<usize>,
    last_input: Vec2,
    fade_out: f32,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            camera: Camera3D {
                position: vec3(-4.156386, -2.2503686, 8.904824),
                target: vec3(-4.042063, -2.0609324, 7.911381),
                up: vec3(0., 1.0, 0.),
                ..Default::default()
            },
            button_index: None,
            last_input: Vec2::ZERO,
            fade_out: 0.0,
        }
    }
    fn get_look_angle(&self) -> Vec2 {
        let target = self.camera.target - self.camera.position;
        let y_angle = target.z.atan2(target.x);
        let x_angle = target.y.atan();

        Vec2 {
            x: x_angle,
            y: y_angle,
        }
    }
    fn look(&mut self, mut mouse: Vec2) {
        mouse.x = -mouse.x;
        mouse *= 0.6;

        let old_look_angle = self.get_look_angle();

        let move_y_angle = mouse.x.atan();
        let new_y_angle = move_y_angle + old_look_angle.y;

        let move_x_angle = mouse.y.atan();
        let new_x_angle = move_x_angle + old_look_angle.x;

        let x = new_y_angle.cos();
        let y = new_x_angle.tan();
        let z = new_y_angle.sin();

        self.camera.target = vec3(x, y, z) + self.camera.position;
    }
    fn forward(&self, include_vertical: bool) -> Vec3 {
        let mut forward = self.camera.target - self.camera.position;
        if !include_vertical {
            forward.y = 0.;
        }
        forward.normalize()
    }
    fn right(&self, include_vertical: bool) -> Vec3 {
        let mut target = self.camera.target - self.camera.position;
        target.x *= -1.;
        (target.x, target.z) = (target.z, target.x);
        if !include_vertical {
            target.y = 0.;
        }

        target.normalize()
    }
    fn movement(&mut self, gamepad_engine: &mut Gamepads) {
        let i = get_input_axis(gamepad_engine);
        let mut moved: Vec3 = Vec3::ZERO;

        moved += self.forward(true) * -i.y * get_frame_time() * 10.0;
        moved += self.right(false) * -i.x * get_frame_time() * 10.0;
        self.camera.position += moved;
        self.camera.target += moved;
        if is_mouse_button_down(MouseButton::Right) {
            self.look(mouse_delta_position());
        } else {
            self.look(Vec2::ZERO);
        }
        if is_key_pressed(KeyCode::G) {
            dbg!(self.camera.position);
            dbg!(self.camera.target);
        }
    }
    pub fn update(
        &mut self,
        assets: &Assets,
        gamepad_engine: &mut Gamepads,
        save_manager: &mut SaveManager,
    ) -> bool {
        self.time += get_frame_time();
        if self.fade_out > 0.0 {
            self.fade_out += get_frame_time();
        }

        if DEBUG_FLAGS.menufly {
            self.movement(gamepad_engine);
        } else {
            const ORBIT_RADIUS: f32 = 8.7;
            const ORBIT_CENTER: Vec3 = Vec3::ZERO;
            const ORBIT_SPEED: f32 = 0.1;
            self.camera.target = ORBIT_CENTER
                + vec3(0.0, 0.5, 0.0)
                + vec3(
                    (self.time * ORBIT_SPEED + PI / 2.0).cos(),
                    0.0,
                    (self.time * ORBIT_SPEED + PI / 2.0).sin(),
                ) * 3.0;
            self.camera.position.x =
                (self.time * ORBIT_SPEED).cos() * ORBIT_RADIUS + ORBIT_CENTER.x;
            self.camera.position.z =
                (self.time * ORBIT_SPEED).sin() * ORBIT_RADIUS + ORBIT_CENTER.z;
        }
        const BUTTON_AMT: usize = 2;

        let input = get_input_axis(gamepad_engine);
        if input != self.last_input {
            if input.y != 0.0 {
                if input.y < 0.0 {
                    self.button_index = Some(
                        self.button_index
                            .map(|f| if f > 0 { f - 1 } else { BUTTON_AMT - 1 })
                            .unwrap_or(0),
                    );
                } else {
                    self.button_index = Some(
                        self.button_index
                            .map(|f| if f + 1 < BUTTON_AMT { f + 1 } else { 0 })
                            .unwrap_or(0),
                    );
                }
            }
            self.last_input = input;
        }

        set_camera(&self.camera);
        clear_background(Color::from_hex(0x1cb7ff));
        // draw scene
        draw_mesh(&assets.tower_mesh);

        // return to default camera to draw UI
        set_default_camera();

        let actual_screen_width = screen_width();
        let actual_screen_height = screen_height();

        let scale_factor = actual_screen_height / SCREEN_HEIGHT;

        let margin: Vec2 = vec2(11.0, 6.0) * scale_factor;
        draw_texture_ex(
            &assets.logo,
            margin.x,
            margin.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(assets.logo.size() * scale_factor),
                ..Default::default()
            },
        );

        fn draw_button(
            x: f32,
            y: f32,
            index: usize,
            assets: &Assets,
            scale_factor: f32,
            selected: bool,
        ) -> bool {
            let size = assets.menu_button.frames[0].0.size() * scale_factor;
            let mouse = mouse_position();
            let hovered = (x..x + size.x).contains(&mouse.0) && (y..y + size.y).contains(&mouse.1);

            if selected || hovered {
                gl_use_material(&BUTTON_HOVER_MATERIAL);
            }
            draw_texture_ex(
                &assets.menu_button.frames[index].0,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(size),
                    ..Default::default()
                },
            );
            if selected || hovered {
                gl_use_default_material();
            }
            hovered
        }

        let play_hovered = draw_button(
            margin.x,
            margin.y + 57.0 * scale_factor,
            if save_manager.level == 0 { 0 } else { 1 },
            assets,
            scale_factor,
            self.button_index.is_some_and(|f| f == 0),
        );
        draw_button(
            margin.x,
            margin.y + (57.0 + 15.0) * scale_factor,
            2,
            assets,
            scale_factor,
            self.button_index.is_some_and(|f| f == 1),
        );

        let level = &assets.levels[save_manager.level];
        let (world, level) = level
            .name
            .split_once('-')
            .map(|f| {
                (
                    f.0.parse::<usize>().unwrap(),
                    (&f.1[..1]).parse::<u8>().unwrap_or_else(|_| {
                        assets.levels[save_manager.level - 1].name[2..3]
                            .parse::<u8>()
                            .unwrap()
                            + 1
                    }),
                )
            })
            .unwrap();
        // count how many levels are in the current world
        let mut count = 0;
        for level in &assets.levels {
            if level.get_world_index() == world as _ {
                count += 1;
            }
        }

        let world_tower_heights = [37.0, 37.0, 33.0];
        let mut prev_worlds = 0.0;
        for i in 0..world {
            prev_worlds += world_tower_heights[i];
        }
        let bubble_height =
            level as f32 / (count - 1) as f32 * world_tower_heights[world] + prev_worlds;
        let draw_height = actual_screen_height
            - 16.0 * scale_factor
            - assets.player_bubble.frames[0].0.height() * scale_factor
            - bubble_height * scale_factor
            + if bubble_height > 0.0 {
                19.0 * scale_factor
            } else {
                0.0
            };

        let x = actual_screen_width
            - (71.96 * actual_screen_width / actual_screen_height - 88.0) * scale_factor
            - assets.player_bubble.frames[0].0.width() * scale_factor * 2.1;
        draw_texture_ex(
            &assets.player_bubble.frames[if bubble_height == 0.0 { 0 } else { 1 }].0,
            x,
            draw_height,
            WHITE,
            DrawTextureParams {
                dest_size: Some(assets.player_bubble.frames[0].0.size() * scale_factor),
                ..Default::default()
            },
        );
        const FADE_OUT_TIME: f32 = 0.5;
        if self.fade_out > 0.0 {
            draw_rectangle(
                -1.0,
                -1.0,
                actual_screen_width + 2.0,
                actual_screen_height + 2.0,
                BLACK.with_alpha(self.fade_out / FADE_OUT_TIME),
            );
        } else if (play_hovered && is_mouse_button_pressed(MouseButton::Left))
            || (self.button_index.is_some_and(|f| f == 0) && is_jump_pressed(gamepad_engine))
        {
            self.fade_out = get_frame_time();
        }
        self.fade_out >= FADE_OUT_TIME
    }
}
