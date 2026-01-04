use macroquad::prelude::*;

pub const SCREEN_WIDTH: f32 = 256.0;
pub const SCREEN_HEIGHT: f32 = 144.0;
pub const MAX_LASSO_DISTANCE: f32 = 64.0;
pub const GRAVITY: f32 = 9.8 * 75.0;
pub const LEVEL_TRANSITION_LENGTH: f32 = 0.5;
pub const DEATH_TILES: &[u16] = &[128, 352];

pub fn create_camera(w: f32, h: f32) -> Camera2D {
    let rt = render_target(w as u32, h as u32);
    rt.texture.set_filter(FilterMode::Nearest);

    Camera2D {
        render_target: Some(rt),
        zoom: Vec2::new(1.0 / w * 2.0, 1.0 / h * 2.0),
        ..Default::default()
    }
}

pub fn get_input_axis() -> Vec2 {
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
    i
}
