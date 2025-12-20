use macroquad::prelude::*;

pub const SCREEN_WIDTH: f32 = 256.0;
pub const SCREEN_HEIGHT: f32 = 144.0;

pub fn create_camera(w: f32, h: f32) -> Camera2D {
    let rt = render_target(w as u32, h as u32);
    rt.texture.set_filter(FilterMode::Nearest);

    Camera2D {
        render_target: Some(rt),
        zoom: Vec2::new(1.0 / w * 2.0, 1.0 / h * 2.0),
        ..Default::default()
    }
}
