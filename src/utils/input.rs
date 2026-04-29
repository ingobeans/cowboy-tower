use gamepads::{Button, Gamepads};
use macroquad::prelude::*;

pub fn is_pause_pressed(gamepad_engine: &mut Gamepads) -> bool {
    is_key_pressed(KeyCode::Escape)
        || gamepad_engine.all().any(|f| {
            f.is_just_pressed(Button::LeftCenterCluster)
                || f.is_just_pressed(Button::RightCenterCluster)
        })
}
pub fn is_lasso_pressed(gamepad_engine: &mut Gamepads) -> bool {
    is_mouse_button_pressed(MouseButton::Right)
        || gamepad_engine.all().any(|f| {
            f.is_just_pressed(Button::FrontLeftUpper) || f.is_just_pressed(Button::FrontLeftLower)
        })
}
pub fn is_lasso_down(gamepad_engine: &mut Gamepads) -> bool {
    is_mouse_button_down(MouseButton::Right)
        || gamepad_engine.all().any(|f| {
            f.is_currently_pressed(Button::FrontLeftUpper)
                || f.is_currently_pressed(Button::FrontLeftLower)
        })
}
pub fn is_shoot_down(gamepad_engine: &mut Gamepads) -> bool {
    is_mouse_button_down(MouseButton::Left)
        || gamepad_engine.all().any(|f| {
            f.is_currently_pressed(Button::FrontRightUpper)
                || f.is_currently_pressed(Button::FrontRightLower)
        })
}

pub fn is_jump_pressed(gamepad_engine: &mut Gamepads) -> bool {
    is_key_pressed(KeyCode::Space)
        || gamepad_engine.all().any(|f| {
            f.is_just_pressed(Button::ActionRight) || f.is_just_pressed(Button::ActionDown)
        })
}
pub fn is_jump_down(gamepad_engine: &mut Gamepads) -> bool {
    is_key_down(KeyCode::Space)
        || gamepad_engine.all().any(|f| {
            f.is_currently_pressed(Button::ActionRight)
                || f.is_currently_pressed(Button::ActionDown)
        })
}
pub fn is_interact_pressed(gamepad_engine: &mut Gamepads) -> bool {
    is_key_pressed(KeyCode::E)
        || gamepad_engine.all().any(|f| {
            f.is_just_pressed(Button::ActionRight) || f.is_just_pressed(Button::ActionDown)
        })
}

pub fn get_input_axis(gamepad_engine: &mut Gamepads) -> Vec2 {
    let mut i = Vec2::ZERO;

    for controller in gamepad_engine.all() {
        let axis: Vec2 = controller.left_stick().into();
        if axis.length() <= 0.2 {
            // check d-pad
            let left = controller.is_currently_pressed(Button::DPadLeft);
            let right = controller.is_currently_pressed(Button::DPadRight);
            let up = controller.is_currently_pressed(Button::DPadUp);
            let down = controller.is_currently_pressed(Button::DPadDown);
            if left || right || up || down {
                let horizontal = 0.0 - if left { 1.0 } else { 0.0 } + if right { 1.0 } else { 0.0 };
                let vertical = 0.0 + if down { 1.0 } else { 0.0 } - if up { 1.0 } else { 0.0 };
                return vec2(horizontal, vertical);
            }
        } else {
            // cap axis values to -1 or 1, disallow decimal values
            let horizontal =
                0.0 - if axis.x < 0.0 { 1.0 } else { 0.0 } + if axis.x > 0.0 { 1.0 } else { 0.0 };
            let vertical =
                0.0 + if axis.y < 0.0 { 1.0 } else { 0.0 } - if axis.y > 0.0 { 1.0 } else { 0.0 };
            return vec2(horizontal, vertical);
        }
    }

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
