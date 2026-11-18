mod entity;
mod fruit;
mod game;
mod global;
mod segment;
mod snake;
mod ui;

use game::Game;
use macroquad::prelude::*;
use macroquad::window::miniquad::conf::Icon;

use crate::global::{clampf, theme, SCR_HEIGHT, SCR_WIDTH, TITLE};

/// Ikon jendela digambar secara prosedural (sama seperti setup_icon() di C++).
fn make_icon(size: usize) -> Vec<u8> {
    let mut icon = vec![0u8; size * size * 4];
    let scale = size as f32 / 32.0;

    fn put(icon: &mut [u8], size: usize, x: usize, y: usize, c: [u8; 4]) {
        let i = (y * size + x) * 4;
        icon[i..i + 4].copy_from_slice(&c);
    }

    fn color(c: macroquad::prelude::Color, alpha: u8) -> [u8; 4] {
        [
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
            alpha,
        ]
    }

    // Panel gelap membulat.
    let radius = 7.0 * scale;
    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let cx = clampf(px, radius, size as f32 - radius);
            let cy = clampf(py, radius, size as f32 - radius);
            let dx = px - cx;
            let dy = py - cy;
            if dx * dx + dy * dy <= radius * radius {
                put(&mut icon, size, x, y, [13, 19, 36, 255]);
            }
        }
    }

    // Apel.
    let apple_x = 16.0 * scale;
    let apple_y = 19.5 * scale;
    let apple_r = 9.4 * scale;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - apple_x;
            let dy = y as f32 + 0.5 - apple_y;
            if dx * dx + dy * dy <= apple_r * apple_r {
                put(&mut icon, size, x, y, color(theme::FRUIT, 255));
            }
        }
    }

    // Daun.
    let leaf_x = 19.0 * scale;
    let leaf_y = 10.5 * scale;
    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 + 0.5 - leaf_x) / (4.2 * scale);
            let dy = (y as f32 + 0.5 - leaf_y) / (2.0 * scale);
            if dx * dx + dy * dy <= 1.0 {
                put(&mut icon, size, x, y, color(theme::LEAF, 255));
            }
        }
    }

    // Kilau putih.
    for y in (14.0 * scale) as usize..(18.0 * scale) as usize {
        for x in (12.0 * scale) as usize..(15.0 * scale) as usize {
            if x < size && y < size {
                put(&mut icon, size, x, y, [255, 255, 255, 190]);
            }
        }
    }

    icon
}

fn window_conf() -> Conf {
    let small_data = make_icon(16);
    let medium_data = make_icon(32);
    let big_data = make_icon(64);

    let mut small = [0u8; 1024];
    let mut medium = [0u8; 4096];
    let mut big = [0u8; 16384];
    small.copy_from_slice(&small_data);
    medium.copy_from_slice(&medium_data);
    big.copy_from_slice(&big_data);

    Conf {
        window_title: TITLE.to_string(),
        window_width: SCR_WIDTH as i32,
        window_height: SCR_HEIGHT as i32,
        window_resizable: false,
        high_dpi: false,
        icon: Some(Icon { small, medium, big }),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    loop {
        for key in get_keys_pressed() {
            if game.handle_key(key) {
                game.save_best();
                return;
            }
        }

        if is_quit_requested() {
            game.save_best();
            return;
        }

        let dt = clampf(get_frame_time(), 0.0, 0.1);
        game.update(dt);
        game.render();
        next_frame().await;
    }
}
