use std::env::args;

use gamepads::Gamepads;
use macroquad::{miniquad::conf::Platform, prelude::*};

use crate::{assets::Assets, game::Game, menu::MainMenu, utils::*};

mod assets;
mod bosses;
mod enemies;
mod game;
mod menu;
mod player;
mod projectiles;
mod tower;
mod ui;
mod utils;

struct GameManager<'a> {
    assets: &'a Assets,
    game: Option<Game<'a>>,
    menu: MainMenu,
    gamepad_engine: Gamepads,
}

impl<'a> GameManager<'a> {
    fn new(assets: &'a Assets, level: Option<usize>) -> Self {
        Self {
            assets,
            gamepad_engine: Gamepads::new(),
            game: level.map(|i| Game::new(assets, i)),
            menu: MainMenu::new(),
        }
    }
    fn update(&mut self) {
        self.gamepad_engine.poll();
        if let Some(game) = &mut self.game {
            game.update(&mut self.gamepad_engine);
        } else {
            let result = self.menu.update(self.assets, &mut self.gamepad_engine);
            if result {
                self.game = Some(Game::new(self.assets, 0));
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "cowboy tower".to_string(),

        platform: Platform {
            // disable vsync if uncapped flag
            swap_interval: if DEBUG_FLAGS.uncapped { Some(0) } else { None },
            ..Default::default()
        },

        // set window width and height to be equal to target width and height,
        // so no scaling, or scale_factor == 1.0.
        //
        // useful for pixel perfect screenshots, like for making UI mockups.
        window_width: if DEBUG_FLAGS.unscaled {
            SCREEN_WIDTH as i32
        } else {
            800
        },
        window_height: if DEBUG_FLAGS.unscaled {
            SCREEN_HEIGHT as i32
        } else {
            600
        },
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    info!("cowboy tower v{}", env!("CARGO_PKG_VERSION"));
    let assets = Assets::load();
    let mut level = None;

    // load level from command line argument
    'outer: for arg in args().skip(1) {
        // check for direct match
        for (i, l) in assets.levels.iter().enumerate() {
            if l.name == arg {
                level = Some(i);
                break 'outer;
            }
        }
        // check for start of name match
        for (i, l) in assets.levels.iter().enumerate() {
            if l.name.starts_with(&arg) {
                level = Some(i);
                break 'outer;
            }
        }
    }
    let mut game_manager = GameManager::new(&assets, level);

    loop {
        game_manager.update();
        next_frame().await;
    }
}
