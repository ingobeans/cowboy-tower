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
    fn is_just_pressed(&mut self, buttons: &[Button]) -> bool {
        self.inner
            .all()
            .any(|f| buttons.iter().any(|b| f.is_just_pressed(*b)))
    }
    fn is_currently_pressed(&mut self, buttons: &[Button]) -> bool {
        self.inner
            .all()
            .any(|f| buttons.iter().any(|b| f.is_currently_pressed(*b)))
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

    pub fn is_action_pressed(&mut self, action: Action) -> bool {
        match action {
            Action::Keybased(keycode, buttons) => {
                let key_pressed = is_key_pressed(keycode);
                if key_pressed {
                    self.last_input_type_controller = false;
                    true
                } else if self.is_just_pressed(buttons) {
                    self.last_input_type_controller = true;
                    true
                } else {
                    false
                }
            }
            Action::Mousebased(mousebutton, buttons) => {
                let mouse_pressed = is_mouse_button_pressed(mousebutton);
                if mouse_pressed {
                    self.last_input_type_controller = false;
                    true
                } else if self.is_just_pressed(buttons) {
                    self.last_input_type_controller = true;
                    true
                } else {
                    false
                }
            }
        }
    }
    pub fn is_action_down(&mut self, action: Action) -> bool {
        match action {
            Action::Keybased(keycode, buttons) => {
                let key_pressed = is_key_down(keycode);
                if key_pressed {
                    self.last_input_type_controller = false;
                    true
                } else if self.is_currently_pressed(buttons) {
                    self.last_input_type_controller = true;
                    true
                } else {
                    false
                }
            }
            Action::Mousebased(mousebutton, buttons) => {
                let mouse_pressed = is_mouse_button_down(mousebutton);
                if mouse_pressed {
                    self.last_input_type_controller = false;
                    true
                } else if self.is_currently_pressed(buttons) {
                    self.last_input_type_controller = true;
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum Action {
    Keybased(KeyCode, &'static [Button]),
    Mousebased(MouseButton, &'static [Button]),
}

// define actions
pub static LASSO: Action = Action::Mousebased(
    MouseButton::Right,
    &[Button::FrontLeftLower, Button::FrontLeftUpper],
);
pub static SHOOT: Action = Action::Mousebased(
    MouseButton::Left,
    &[Button::FrontRightLower, Button::FrontRightUpper],
);
pub static PAUSE: Action = Action::Keybased(
    KeyCode::Escape,
    &[Button::LeftCenterCluster, Button::RightCenterCluster],
);
pub static JUMP: Action =
    Action::Keybased(KeyCode::Space, &[Button::ActionDown, Button::ActionRight]);
pub static INTERACT: Action =
    Action::Keybased(KeyCode::E, &[Button::ActionDown, Button::ActionRight]);

pub fn get_input_axis(gamepad_engine: &mut Gamepads) -> Vec2 {
    let mut i = Vec2::ZERO;

    if let Some(axis) = gamepad_engine.get_axis() {
        gamepad_engine.last_input_type_controller = true;
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

    if i != Vec2::ZERO {
        gamepad_engine.last_input_type_controller = false;
    }

    i
}
