use macroquad::prelude::*;
#[cfg(test)]
use std::f32::consts::PI;
mod debug;
pub use debug::*;

pub const SCREEN_WIDTH: f32 = 256.0;
pub const SCREEN_HEIGHT: f32 = 144.0;
pub const MAX_LASSO_DISTANCE: f32 = 64.0;
pub const GRAVITY: f32 = 9.8 * 75.0;
pub const LEVEL_TRANSITION_LENGTH: f32 = 0.5;
pub const DEATH_TILES: &[u16] = &[128, 352, 288];

pub const FLOOR_PADDING: f32 = 16.0;

pub const DIALOGUE_SLIDE_IN_TIME: f32 = 0.5;
pub const TEXT_FADE_IN_TIME: f32 = 0.2;
pub const CINEMATIC_BAR_FADE_TIME: f32 = 1.0;

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

#[test]
fn find_lowest_drift_factor() {
    // This isnt exactly a "test" per se, but it was convenient enough to mark it as one so i can run with `cargo test -- --nocapture`

    // The actual movement of enemies with movement type wander, is generated with the following formula:
    //      sin(t)*sin(A * t + 1.5)*sin(B * t + 8.0)^2
    // because its sporatic enough. All values between -0.1 and 0.1 are clamped to 0, and
    // all other values are clamped to either -1 or 1, whichever is closest.
    // I want the integral of this function over one period to be as close to 0 as possible,
    // to ensure enemies dont drift towards one direction over time.
    // This function runs 1000 tests to find the values of A and B that yield the integral closest to 0.

    let step_size = 0.001;

    let tests = 1000;

    let mut best = (f32::MAX, (0.0, 0.0));

    for _ in 0..tests {
        let current_test = (rand::gen_range(1.0, 5.0), rand::gen_range(1.0, 10.0));

        let mut total = 0.0;
        let mut last = 0.0;
        for step in 0..(1.0 / step_size * PI) as u32 {
            let value = step as f32 * step_size;

            let delta_time = value - last;
            last = value;

            let value = value.sin()
                * (value * current_test.0 + 1.5).sin()
                * (value * current_test.1 + 8.0).sin().powi(2);
            let value = if value.abs() < 0.1 {
                0.0
            } else if value.is_sign_positive() {
                1.0
            } else {
                -1.0
            };

            total += value * delta_time;
        }
        // best is overriden if this total is closer to 0.
        // however, a restriction is also imposed that total must be less than 0,
        // since earlier maps were built with the knowledge in mind that enemies drift to the left over time.
        // so changing drift direction would be disruptive
        if total < 0.0 && total.abs() < best.0 {
            best = (total, current_test)
        }
    }
    dbg!(best);
}
