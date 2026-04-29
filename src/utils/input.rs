use gamepads::Button;
use macroquad::prelude::*;

pub struct Gamepads {
    inner: gamepads::Gamepads,
    pub last_input_type_controller: bool,
}
impl Gamepads {
    pub fn new() -> Self {
        Self {
            inner: gamepads::Gamepads::new(),
            last_input_type_controller: false,
        }
    }
    pub fn poll(&mut self) {
        self.inner.poll();
    }
    pub fn is_just_pressed(&mut self, buttons: &[Button]) -> bool {
        let result = self
            .inner
            .all()
            .any(|f| buttons.iter().any(|b| f.is_just_pressed(*b)));
        if result {
            self.last_input_type_controller = true;
        }
        result
    }
    pub fn is_currently_pressed(&mut self, buttons: &[Button]) -> bool {
        let result = self
            .inner
            .all()
            .any(|f| buttons.iter().any(|b| f.is_currently_pressed(*b)));
        if result {
            self.last_input_type_controller = true;
        }
        result
    }
    pub fn get_axis(&mut self) -> Option<Vec2> {
        for controller in self.inner.all() {
            let axis: Vec2 = controller.left_stick().into();
            if axis.length() <= 0.2 {
                // check d-pad
                let left = controller.is_currently_pressed(Button::DPadLeft);
                let right = controller.is_currently_pressed(Button::DPadRight);
                let up = controller.is_currently_pressed(Button::DPadUp);
                let down = controller.is_currently_pressed(Button::DPadDown);
                if left || right || up || down {
                    let horizontal =
                        0.0 - if left { 1.0 } else { 0.0 } + if right { 1.0 } else { 0.0 };
                    let vertical = 0.0 + if down { 1.0 } else { 0.0 } - if up { 1.0 } else { 0.0 };

                    self.last_input_type_controller = true;
                    return Some(vec2(horizontal, vertical));
                }
            } else {
                // cap axis values to -1 or 1, disallow decimal values
                let horizontal = 0.0 - if axis.x < 0.0 { 1.0 } else { 0.0 }
                    + if axis.x > 0.0 { 1.0 } else { 0.0 };
                let vertical = 0.0 + if axis.y < 0.0 { 1.0 } else { 0.0 }
                    - if axis.y > 0.0 { 1.0 } else { 0.0 };

                self.last_input_type_controller = true;
                return Some(vec2(horizontal, vertical));
            }
        }
        None
    }
}

pub fn is_pause_pressed(gamepad_engine: &mut Gamepads) -> bool {
    is_key_pressed(KeyCode::Escape)
        || gamepad_engine.is_just_pressed(&[Button::LeftCenterCluster, Button::RightCenterCluster])
}
pub fn is_lasso_pressed(gamepad_engine: &mut Gamepads) -> bool {
    is_mouse_button_pressed(MouseButton::Right)
        || gamepad_engine.is_just_pressed(&[Button::FrontLeftLower, Button::FrontLeftUpper])
}
pub fn is_lasso_down(gamepad_engine: &mut Gamepads) -> bool {
    is_mouse_button_down(MouseButton::Right)
        || gamepad_engine.is_currently_pressed(&[Button::FrontLeftLower, Button::FrontLeftUpper])
}
pub fn is_shoot_down(gamepad_engine: &mut Gamepads) -> bool {
    is_mouse_button_down(MouseButton::Left)
        || gamepad_engine.is_currently_pressed(&[Button::FrontRightLower, Button::FrontRightUpper])
}

pub fn is_jump_pressed(gamepad_engine: &mut Gamepads) -> bool {
    is_key_pressed(KeyCode::Space)
        || gamepad_engine.is_just_pressed(&[Button::ActionDown, Button::ActionRight])
}
pub fn is_jump_down(gamepad_engine: &mut Gamepads) -> bool {
    is_key_down(KeyCode::Space)
        || gamepad_engine.is_currently_pressed(&[Button::ActionDown, Button::ActionRight])
}
pub fn is_interact_pressed(gamepad_engine: &mut Gamepads) -> bool {
    is_key_pressed(KeyCode::E)
        || gamepad_engine.is_just_pressed(&[Button::ActionDown, Button::ActionRight])
}

pub fn get_input_axis(gamepad_engine: &mut Gamepads) -> Vec2 {
    let mut i = Vec2::ZERO;

    if let Some(axis) = gamepad_engine.get_axis() {
        return axis;
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
