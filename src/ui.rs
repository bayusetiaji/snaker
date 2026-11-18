//! Primitif menggambar: rounded rect, panel, glow, background, teks, keycap.
//! Padanan dari `ui.h` / `ui.cpp`.

use macroquad::prelude::*;

use crate::global::{
    clampf, faded, mix,
    theme::{self, BG_BOTTOM, BG_TOP},
    SCR_HEIGHT, SCR_WIDTH,
};

const PI: f32 = std::f32::consts::PI;

fn corner_center(corner: i32, width: f32, height: f32, radius: f32) -> Vec2 {
    match corner {
        0 => vec2(radius, radius),
        1 => vec2(width - radius, radius),
        2 => vec2(width - radius, height - radius),
        _ => vec2(radius, height - radius),
    }
}

/// Builds the boundary points of a shape that looks like a rounded rectangle.
pub fn rounded_rect_points(
    width: f32,
    height: f32,
    radius: f32,
    mut corner_segments: i32,
) -> Vec<Vec2> {
    if corner_segments < 1 {
        corner_segments = 1;
    }
    let radius = clampf(radius, 0.0, width.min(height) / 2.0);
    let total = corner_segments * 4;
    let mut points = Vec::with_capacity(total as usize);
    let step = 90.0 / corner_segments as f32;

    for corner in 0..4 {
        let center = corner_center(corner, width, height, radius);
        let base = 180.0 + corner as f32 * 90.0;
        for i in 0..corner_segments {
            let angle = (base + i as f32 * step) * PI / 180.0;
            points.push(vec2(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }
    }
    points
}

/// Isi rounded rectangle memakai triangle fan (tanpa overlap antar segitiga).
pub fn fill_rounded_rect(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    fill: Color,
) {
    let points = rounded_rect_points(width, height, radius, 5);
    let center = vec2(x + width / 2.0, y + height / 2.0);
    let origin = vec2(x, y);
    let n = points.len();
    for i in 0..n {
        draw_triangle(center, points[i] + origin, points[(i + 1) % n] + origin, fill);
    }
}

pub fn fill_rounded_rect_centered(
    cx: f32,
    cy: f32,
    width: f32,
    height: f32,
    radius: f32,
    fill: Color,
) {
    fill_rounded_rect(
        cx - width / 2.0,
        cy - height / 2.0,
        width,
        height,
        radius,
        fill,
    );
}

pub fn draw_panel(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    fill: Color,
    outline: Color,
    outline_thickness: f32,
) {
    fill_rounded_rect(x, y, width, height, radius, fill);
    if outline_thickness <= 0.0 {
        return;
    }
    let points = rounded_rect_points(width, height, radius, 5);
    let origin = vec2(x, y);
    for i in 0..points.len() {
        draw_line(
            points[i].x + origin.x,
            points[i].y + origin.y,
            points[(i + 1) % points.len()].x + origin.x,
            points[(i + 1) % points.len()].y + origin.y,
            outline_thickness,
            outline,
        );
    }
}

pub fn draw_glow(center: Vec2, radius: f32, color: Color, intensity: f32) {
    const LAYERS: i32 = 5;
    for i in 0..LAYERS {
        let scale = 1.0 - i as f32 * 0.16;
        let mut layer = color;
        layer.a = clampf(
            color.a * (0.10 + 0.11 * i as f32) * intensity,
            0.0,
            1.0,
        );
        draw_circle(center.x, center.y, radius * scale, layer);
    }
}

pub fn draw_background(time: f32) {
    const BANDS: i32 = 72;
    for i in 0..BANDS {
        let t0 = i as f32 / BANDS as f32;
        let t1 = (i + 1) as f32 / BANDS as f32;
        let color = mix(BG_TOP, BG_BOTTOM, (t0 + t1) * 0.5);
        draw_rectangle(
            0.0,
            t0 * SCR_HEIGHT,
            SCR_WIDTH,
            SCR_HEIGHT / BANDS as f32 + 1.0,
            color,
        );
    }

    let sx = SCR_WIDTH;
    let sy = SCR_HEIGHT;
    draw_glow(
        vec2(
            sx * 0.10 + (time * 0.23).sin() * 26.0,
            sy * 0.16 + (time * 0.19).cos() * 20.0,
        ),
        230.0,
        Color::new(35.0 / 255.0, 200.0 / 255.0, 1.0, 26.0 / 255.0),
        0.9,
    );
    draw_glow(
        vec2(
            sx * 0.90 + (time * 0.17).cos() * 24.0,
            sy * 0.78 + (time * 0.21).sin() * 26.0,
        ),
        260.0,
        Color::new(66.0 / 255.0, 100.0 / 255.0, 1.0, 24.0 / 255.0),
        0.9,
    );
    draw_glow(
        vec2(sx * 0.82, sy * 0.16 + (time * 0.27).sin() * 14.0),
        150.0,
        Color::new(150.0 / 255.0, 80.0 / 255.0, 1.0, 18.0 / 255.0),
        0.8,
    );
}

#[derive(Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
}

fn glyph_width(font: &Font, ch: char, size: u16) -> f32 {
    measure_text(&ch.to_string(), Some(font), size, 1.0).width
}

pub fn text_width(font: &Font, string: &str, size: u16, letter_spacing: f32) -> f32 {
    let mut width = 0.0;
    let mut first = true;
    for ch in string.chars() {
        if !first {
            width += letter_spacing;
        }
        width += glyph_width(font, ch, size);
        first = false;
    }
    width
}

/// Menggambar teks dengan posisi `x, y` sebagai pojok kiri-atas (seperti sf::Text).
pub fn draw_text(
    font: &Font,
    string: &str,
    size: u16,
    color: Color,
    x: f32,
    y: f32,
    align: Align,
    letter_spacing: f32,
) {
    let total = text_width(font, string, size, letter_spacing);
    let start_x = match align {
        Align::Left => x,
        Align::Center => x - total * 0.5,
        Align::Right => x - total,
    };

    let dims = measure_text(string, Some(font), size, 1.0);
    let baseline = y + dims.offset_y;
    let params = TextParams {
        font: Some(font),
        font_size: size,
        font_scale: 1.0,
        font_scale_aspect: 1.0,
        rotation: 0.0,
        color,
    };

    let mut cursor = start_x;
    for ch in string.chars() {
        let text = ch.to_string();
        draw_text_ex(&text, cursor, baseline, params.clone());
        cursor += glyph_width(font, ch, size) + letter_spacing;
    }
}

pub fn draw_keycap(
    font: &Font,
    label: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    highlighted: bool,
) {
    draw_panel(
        x,
        y,
        width,
        height,
        7.0,
        if highlighted {
            Color::new(31.0 / 255.0, 55.0 / 255.0, 78.0 / 255.0, 1.0)
        } else {
            theme::PANEL_ALT
        },
        if highlighted {
            theme::ACCENT
        } else {
            theme::BORDER
        },
        1.0,
    );

    draw_text(
        font,
        label,
        12,
        if highlighted {
            theme::ACCENT
        } else {
            theme::TEXT
        },
        x + width / 2.0,
        y + height / 2.0 - 7.0,
        Align::Center,
        1.05,
    );
}

pub fn pulse_color(color: Color, time: f32, low: f32, high: f32) -> Color {
    let wave = 0.5 + 0.5 * (time * 4.0).sin();
    faded(color, low + (high - low) * wave)
}
