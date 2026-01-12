use crate::assets::Assets;
use macroquad::prelude::*;

pub fn draw_boss_badges(
    assets: &Assets,
    amt: f32,
    mut achieved: u8,
    screen_offset: Vec2,
    active_screen_width: f32,
) {
    // draw boss badge animation
    const BOSS_COUNT: u8 = 3;
    const FADE_IN_TIME: f32 = 0.2;
    const PAUSE_TIME: f32 = 1.0;
    const FLY_OFF_TIME: f32 = 0.2;

    let gap = 6.0;
    let padding = 3.0;
    let width = BOSS_COUNT as f32 * (10.0 + gap) - gap + 2.0 * padding;
    let height = 12.0;

    let animation_time = amt - 1.0;
    let delta = animation_time - (assets.get_badge.total_length - 1) as f32 / 1000.0;
    let mut fly_off = 0.0;
    if delta > PAUSE_TIME {
        fly_off = ((delta - PAUSE_TIME) / FLY_OFF_TIME).min(1.0);
    }

    let x = (screen_offset.x + (active_screen_width - width) / 2.0).floor();
    let y = (screen_offset.y + 8.0).floor() - fly_off * 22.0;

    let alpha = (amt / FADE_IN_TIME).min(1.0);

    // draw base
    draw_rectangle(
        x - 2.0,
        y - 2.0,
        width + 4.0,
        height + 4.0,
        WHITE.with_alpha(alpha),
    );
    draw_rectangle_lines(
        x - 1.0,
        y - 1.0,
        width + 2.0,
        height + 2.0,
        2.0,
        BLACK.with_alpha(alpha),
    );

    // draw current badges
    if amt < 1.0 {
        achieved -= 1;
    }

    // draw badge get animation
    if animation_time > 0.0 && delta < 0.0 {
        draw_texture(
            assets
                .get_badge
                .get_at_time((animation_time * 1000.0) as u32),
            x + padding + (achieved - 1) as f32 * (10.0 + gap) - 5.0,
            y + 1.0 - 5.0,
            WHITE,
        );
    }

    for i in 0..BOSS_COUNT {
        let color = if i < achieved { WHITE } else { BLACK };
        assets.boss_badges.draw_tile(
            x + padding + i as f32 * (10.0 + gap),
            y + 1.0,
            i as f32,
            0.0,
            Some((DrawTextureParams::default(), color.with_alpha(alpha))),
        );
    }
    // draw current badge animation
    if amt < 1.0 {
        let start_pos = screen_offset - vec2(16.0, 16.0);
        let target_pos = vec2(x + padding + achieved as f32 * (10.0 + gap), y + 1.0);
        let x = start_pos.x.lerp(target_pos.x, amt);
        let y = (-(2.0 * amt - 1.0).powi(2) + 1.0) * 32.0 + start_pos.y.lerp(target_pos.y, amt);
        assets
            .boss_badges
            .draw_tile(x, y, achieved as f32, 0.0, None);
    }
}
