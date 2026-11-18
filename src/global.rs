//! Konstanta layout, palet warna, dan utilitas bersama.
//! Mirror dari `global.h` / `global.cpp` di proyek SnakeD (C++/SFML).

use macroquad::prelude::*;
use std::cell::Cell;

pub const TITLE: &str = "SnakeD - Neon Edition (Rust)";
pub const SCR_WIDTH: f32 = 640.0;
pub const SCR_HEIGHT: f32 = 760.0;
pub const GRID_SIZE: f32 = 20.0;

pub const WORLD_PAD_TOP: f32 = 96.0;
pub const WORLD_PAD_RIGHT: f32 = 20.0;
pub const WORLD_PAD_BOTTOM: f32 = 64.0;
pub const WORLD_PAD_LEFT: f32 = 20.0;

pub const NUM_GRID_H: i32 =
    ((SCR_WIDTH - WORLD_PAD_LEFT - WORLD_PAD_RIGHT) / GRID_SIZE) as i32;
pub const NUM_GRID_V: i32 =
    ((SCR_HEIGHT - WORLD_PAD_TOP - WORLD_PAD_BOTTOM) / GRID_SIZE) as i32;

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

pub const SNAKE_COLOR: Color = rgba(25, 174, 255, 255);
// Legacy palette (kept for compatibility with the original code).
#[allow(dead_code)]
pub const FRUIT_COLOR: Color = rgba(255, 65, 65, 255);

/// Central place for the "neon" look of the game.
pub mod theme {
    use super::rgba;
    use macroquad::prelude::Color;

    pub const BG_TOP: Color = rgba(8, 12, 24, 255);
    pub const BG_BOTTOM: Color = rgba(16, 22, 42, 255);

    pub const FIELD: Color = rgba(10, 15, 30, 255);
    pub const PANEL: Color = rgba(17, 24, 44, 255);
    pub const PANEL_ALT: Color = rgba(26, 35, 60, 255);
    pub const BORDER: Color = rgba(38, 51, 82, 255);
    pub const BORDER_BRIGHT: Color = rgba(70, 96, 145, 255);

    pub const TEXT: Color = rgba(232, 240, 252, 255);
    pub const TEXT_MUTED: Color = rgba(122, 140, 175, 255);

    pub const ACCENT: Color = rgba(52, 226, 255, 255);
    #[allow(dead_code)]
    pub const ACCENT_SOFT: Color = rgba(24, 132, 173, 255);
    pub const GOLD: Color = rgba(255, 205, 92, 255);
    pub const DANGER: Color = rgba(255, 84, 112, 255);
    #[allow(dead_code)]
    pub const SUCCESS: Color = rgba(94, 230, 144, 255);

    pub const SNAKE_HEAD: Color = rgba(70, 238, 255, 255);
    pub const SNAKE_TAIL: Color = rgba(27, 83, 150, 255);
    pub const FRUIT: Color = rgba(255, 78, 104, 255);
    pub const LEAF: Color = rgba(96, 224, 138, 255);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Idle,
    Up,
    Right,
    Down,
    Left,
}

pub fn opposite(d: Direction) -> Direction {
    match d {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
        _ => Direction::Idle,
    }
}

pub fn clampf(value: f32, lo: f32, hi: f32) -> f32 {
    if value < lo {
        lo
    } else if value > hi {
        hi
    } else {
        value
    }
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = clampf(t, 0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

/// Sama seperti `sf::Color(r, g, b, a)` dengan alpha dalam skala 0..255.
#[allow(dead_code)]
pub fn with_alpha(c: Color, alpha: u8) -> Color {
    Color::new(c.r, c.g, c.b, alpha as f32 / 255.0)
}

pub fn faded(c: Color, alpha: f32) -> Color {
    Color::new(c.r, c.g, c.b, alpha)
}

// RNG kecil (xorshift64) supaya tidak bergantung pada API rand macroquad.
thread_local! {
    static RNG_STATE: Cell<u64> = Cell::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
            ^ 0x9E37_79B9_7F4A_7C15,
    );
}

fn next_u64() -> u64 {
    RNG_STATE.with(|state| {
        let mut x = state.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        state.set(x);
        x
    })
}

pub fn rand_unit() -> f32 {
    (next_u64() >> 11) as f32 * (1.0 / ((1u64 << 53) as f32))
}

pub fn rand_int(n: i32) -> i32 {
    if n <= 0 {
        0
    } else {
        (rand_unit() * n as f32) as i32
    }
}
