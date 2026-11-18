//! Satu ruas tubuh ular. Padanan dari `segment.h` / `segment.cpp`.

use macroquad::prelude::*;

use crate::entity::Entity;
use crate::global::{GRID_SIZE, SNAKE_COLOR, WORLD_PAD_LEFT, WORLD_PAD_TOP};
use crate::ui;

#[derive(Clone, Copy)]
pub struct Segment {
    pub entity: Entity,
    px: i32,
    py: i32,
}

impl Segment {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            entity: Entity::new(x, y),
            px: x,
            py: y,
        }
    }

    pub fn x(&self) -> i32 {
        self.entity.x
    }

    pub fn y(&self) -> i32 {
        self.entity.y
    }

    #[allow(dead_code)]
    pub fn prev_x(&self) -> i32 {
        self.px
    }

    #[allow(dead_code)]
    pub fn prev_y(&self) -> i32 {
        self.py
    }

    pub fn set_x(&mut self, x: i32) {
        self.entity.set_x(x);
    }

    pub fn set_y(&mut self, y: i32) {
        self.entity.set_y(y);
    }

    pub fn save_prev_xy(&mut self) {
        self.px = self.entity.x;
        self.py = self.entity.y;
    }

    pub fn follow(&mut self, leading: &Segment) {
        self.save_prev_xy();
        self.entity.x = leading.px;
        self.entity.y = leading.py;
    }

    pub fn pixel_pos(&self, alpha: f32) -> Vec2 {
        let current_x = self.entity.x as f32 * GRID_SIZE + WORLD_PAD_LEFT + GRID_SIZE * 0.5;
        let current_y = self.entity.y as f32 * GRID_SIZE + WORLD_PAD_TOP + GRID_SIZE * 0.5;
        let previous_x = self.px as f32 * GRID_SIZE + WORLD_PAD_LEFT + GRID_SIZE * 0.5;
        let previous_y = self.py as f32 * GRID_SIZE + WORLD_PAD_TOP + GRID_SIZE * 0.5;

        vec2(
            previous_x + (current_x - previous_x) * alpha,
            previous_y + (current_y - previous_y) * alpha,
        )
    }

    pub fn show(&self, color: Color, alpha: f32, offset: Vec2) {
        let position = self.pixel_pos(alpha) + offset;
        let size = GRID_SIZE - 3.0;
        ui::fill_rounded_rect_centered(
            position.x,
            position.y,
            size,
            size,
            size * 0.36,
            if color.a > 0.0 { color } else { SNAKE_COLOR },
        );
    }
}
