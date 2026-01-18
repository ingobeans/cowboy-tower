use macroquad::prelude::*;
use std::{
    fmt::{Debug, Display},
    sync::LazyLock,
};

use crate::assets::Level;
#[derive(Debug)]
pub struct DebugFlags {
    pub show_paths: bool,
    pub show_boss_debug: bool,
}
impl Display for DebugFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug = format!("{:?}", self);
        let split = debug.split(", ");
        for (i, item) in split.enumerate() {
            let (key, value) = item.split_once(":").unwrap();
            let key = key.split(" ").last().unwrap();
            if value.contains("true") {
                if i > 0 {
                    write!(f, "\n")?;
                }
                write!(f, "✓ {key}")?;
            }
        }
        Ok(())
    }
}
pub static DEBUG_FLAGS: LazyLock<DebugFlags> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    {
        use std::env::args;
        let args_owned: Vec<String> = args().collect();
        let args: Vec<&str> = args_owned.iter().map(|f| f.as_str()).collect();
        let flags = DebugFlags {
            show_paths: args.contains(&"paths"),
            show_boss_debug: args.contains(&"boss"),
        };
        println!("{flags}");
        flags
    }
    #[cfg(not(debug_assertions))]
    {
        DebugFlags {
            show_paths: false,
            show_boss_debug: false,
        }
    }
});

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
