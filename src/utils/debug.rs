use macroquad::prelude::*;
use std::{
    fmt::{Debug, Display},
    sync::LazyLock,
};

use crate::assets::Level;
#[derive(Debug, Default)]
pub struct DebugFlags {
    pub paths: bool,
    pub boss: bool,
    pub centres: bool,
    pub horses: bool,
    pub special: bool,
    pub bloom: bool,
    pub uncapped: bool,
    pub fps: bool,
    pub unscaled: bool,
}
pub static DEBUG_FLAGS: LazyLock<DebugFlags> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    {
        use std::env::args;
        let args_owned: Vec<String> = args().collect();
        let args: Vec<&str> = args_owned.iter().map(|f| f.as_str()).collect();
        let flags = DebugFlags {
            paths: args.contains(&"paths"),
            boss: args.contains(&"boss"),
            special: args.contains(&"special"),
            horses: args.contains(&"horses"),
            bloom: args.contains(&"bloom"),
            uncapped: args.contains(&"uncapped"),
            fps: args.contains(&"fps"),
            unscaled: args.contains(&"unscaled"),
            centres: args.contains(&"centre") || args.contains(&"center"),
        };
        print!("{flags}");
        flags
    }
    #[cfg(not(debug_assertions))]
    {
        DebugFlags::default()
    }
});
impl Display for DebugFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug = format!("{:?}", self);
        let split = debug.split(", ");
        let mut first = true;
        for item in split {
            let (key, value) = item.split_once(":").unwrap();
            let key = key.split(" ").last().unwrap();
            if value.contains("true") {
                if first {
                    first = false;
                } else {
                    writeln!(f)?;
                }
                write!(f, "✓ {key}")?;
            }
        }
        if first { Ok(()) } else { writeln!(f) }
    }
}

pub fn draw_cross(x: f32, y: f32, color: Color) {
    const LENGTH: f32 = 3.0;
    draw_rectangle(
        x.floor() - (LENGTH - 1.0) / 2.0 - 0.0,
        y.floor(),
        LENGTH,
        1.0,
        color,
    );
    draw_rectangle(
        x.floor(),
        y.floor() - (LENGTH - 1.0) / 2.0 - 0.0,
        1.0,
        LENGTH,
        color,
    );
}

pub fn debug_paths(level: &Level) {
    for (i, path) in level.enemy_paths.iter().enumerate() {
        for (j, pos) in path.iter().enumerate() {
            draw_rectangle_lines(
                pos.x,
                pos.y,
                8.0,
                8.0,
                2.0,
                [RED, GREEN, BLUE, WHITE, BROWN][i],
            );
            draw_rectangle(
                pos.x,
                pos.y,
                8.0,
                8.0,
                [RED, GREEN, BLUE, WHITE, BROWN][i].with_alpha(1.0 - j as f32 / path.len() as f32),
            );
        }
    }
}
