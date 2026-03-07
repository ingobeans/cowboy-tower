use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::{assets::Assets, utils::DEBUG_FLAGS};

pub struct MainMenu {
    camera: Camera3D,
    time: f32,
}

const RENDER_WIDTH: u32 = 256 * 4;
const RENDER_HEIGHT: u32 = 144 * 4;

impl MainMenu {
    pub fn new() -> Self {
        let rt = render_target_ex(
            RENDER_WIDTH,
            RENDER_HEIGHT,
            RenderTargetParams {
                sample_count: 1,
                depth: true,
            },
        );
        rt.texture.set_filter(FilterMode::Nearest);

        Self {
            time: 0.0,
            camera: Camera3D {
                position: vec3(-4.156386, -2.2503686, 8.904824),
                target: vec3(-4.042063, -2.0609324, 7.911381),
                up: vec3(0., 1.0, 0.),
                render_target: Some(rt),
                ..Default::default()
            },
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
    fn movement(&mut self) {
        let mut i = Vec2::ZERO;

        if is_key_down(KeyCode::A) {
            i.x -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            i.x += 1.0;
        }
        if is_key_down(KeyCode::W) {
            i.y -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            i.y += 1.0;
        }
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
    pub fn update(&mut self, assets: &Assets) -> bool {
        self.time += get_frame_time();

        if DEBUG_FLAGS.menufly {
            self.movement();
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

        set_camera(&self.camera);
        clear_background(Color::from_hex(0x1cb7ff));
        draw_mesh(&assets.tower_mesh);

        set_default_camera();
        clear_background(Color::from_hex(0x1cb7ff));

        let actual_screen_width = screen_width();
        let actual_screen_height = screen_height();
        let screen_height = actual_screen_width / RENDER_WIDTH as f32 * RENDER_HEIGHT as f32;

        draw_texture_ex(
            &self.camera.render_target.as_ref().unwrap().texture,
            0.0,
            actual_screen_height - screen_height,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(actual_screen_width, screen_height)),
                flip_y: true,
                ..Default::default()
            },
        );
        draw_text("press space to start", 64.0, 64.0, 64.0, WHITE);
        is_key_pressed(KeyCode::Space)
    }
}
