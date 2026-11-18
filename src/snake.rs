//! Ular: pergerakan, interpolasi, tabrakan, dan rendering.
//! Padanan dari `snake.h` / `snake.cpp`.

use macroquad::prelude::*;

use crate::fruit::Fruit;
use crate::global::{
    clampf, mix,
    theme::{self, ACCENT, SNAKE_HEAD},
    Direction, GRID_SIZE, NUM_GRID_H, NUM_GRID_V,
};
use crate::segment::Segment;
use crate::ui;

pub struct Snake {
    segments: Vec<Segment>,
    t_elapsed: f32,
    dir: Direction,
    next: Direction,
    queued: Direction,
    last_dir: Direction,
    tick_ms: i32,
}

impl Snake {
    pub fn new() -> Self {
        let mut snake = Self {
            segments: Vec::new(),
            t_elapsed: 0.0,
            dir: Direction::Idle,
            next: Direction::Idle,
            queued: Direction::Idle,
            last_dir: Direction::Down,
            tick_ms: 190,
        };
        snake.reset(15, 15);
        snake
    }

    pub fn reset(&mut self, x: i32, y: i32) {
        self.segments.clear();
        self.dir = Direction::Idle;
        self.next = Direction::Idle;
        self.queued = Direction::Idle;
        self.last_dir = Direction::Down;
        self.t_elapsed = 0.0;

        for i in 0..5 {
            self.segments.push(Segment::new(x, y + i));
        }
    }

    pub fn turn(&mut self, direction: Direction) {
        if direction == Direction::Idle {
            return;
        }

        // Input pertama: orientasi ulang tubuh agar arah awal apa pun aman.
        if self.dir == Direction::Idle
            && self.next == Direction::Idle
            && self.queued == Direction::Idle
        {
            self.reorient(direction);
            self.next = direction;
            self.last_dir = direction;
            return;
        }

        let reference = if self.queued != Direction::Idle {
            self.queued
        } else if self.next != Direction::Idle {
            self.next
        } else {
            self.dir
        };

        if direction == reference || direction == crate::global::opposite(reference) {
            return;
        }

        if self.next == Direction::Idle {
            self.next = direction;
        } else if self.queued == Direction::Idle {
            self.queued = direction;
        }
    }

    pub fn update(&mut self, frame_ms: f32, tick_ms: i32) {
        self.tick_ms = tick_ms;
        let dt = frame_ms.min(tick_ms as f32);

        if self.dir == Direction::Idle {
            if self.next == Direction::Idle {
                self.t_elapsed = 0.0;
                return;
            }
            self.dir = self.next;
            self.next = self.queued;
            self.queued = Direction::Idle;
            self.t_elapsed = 0.0;
        }

        self.t_elapsed += dt;

        if self.t_elapsed >= tick_ms as f32 {
            if self.next != Direction::Idle {
                self.dir = self.next;
                self.next = self.queued;
                self.queued = Direction::Idle;
            }
            self.step();
            self.t_elapsed -= tick_ms as f32;
            if self.t_elapsed < 0.0 {
                self.t_elapsed = 0.0;
            }
        }
    }

    fn reorient(&mut self, facing: Direction) {
        if self.segments.is_empty() {
            return;
        }
        let (dx, dy) = match facing {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Right => (-1, 0),
            Direction::Left => (1, 0),
            _ => return,
        };

        let hx = self.segments[0].x();
        let hy = self.segments[0].y();
        for (i, segment) in self.segments.iter_mut().enumerate() {
            let offset = i as i32;
            segment.set_x(hx + dx * offset);
            segment.set_y(hy + dy * offset);
            segment.save_prev_xy();
        }
    }

    fn step(&mut self) {
        if self.dir == Direction::Idle || self.segments.is_empty() {
            return;
        }
        self.last_dir = self.dir;

        let head = &mut self.segments[0];
        head.save_prev_xy();
        match self.dir {
            Direction::Up => head.set_y(head.y() - 1),
            Direction::Right => head.set_x(head.x() + 1),
            Direction::Down => head.set_y(head.y() + 1),
            Direction::Left => head.set_x(head.x() - 1),
            Direction::Idle => {}
        }

        for i in 1..self.segments.len() {
            let leading = self.segments[i - 1];
            self.segments[i].follow(&leading);
        }
    }

    pub fn grow(&mut self) {
        if let Some(tail) = self.segments.last() {
            let segment = Segment::new(tail.x(), tail.y());
            self.segments.push(segment);
        }
    }

    pub fn eat(&self, fruit: &Fruit) -> bool {
        self.segments
            .first()
            .map(|h| h.x() == fruit.x() && h.y() == fruit.y())
            .unwrap_or(false)
    }

    pub fn hit(&self) -> bool {
        let head = match self.segments.first() {
            Some(h) => h,
            None => return false,
        };

        if head.x() <= 0
            || head.x() >= NUM_GRID_H
            || head.y() <= 0
            || head.y() >= NUM_GRID_V
        {
            return true;
        }

        self.segments
            .iter()
            .skip(1)
            .any(|s| s.x() == head.x() && s.y() == head.y())
    }

    pub fn occupies(&self, x: i32, y: i32) -> bool {
        self.segments.iter().any(|s| s.x() == x && s.y() == y)
    }

    pub fn direction(&self) -> Direction {
        self.dir
    }

    pub fn length(&self) -> usize {
        self.segments.len()
    }

    pub fn alpha(&self) -> f32 {
        if self.tick_ms <= 0 || self.dir == Direction::Idle {
            return 0.0;
        }
        clampf(self.t_elapsed / self.tick_ms as f32, 0.0, 1.0)
    }

    pub fn head_pixel(&self, alpha: f32) -> Vec2 {
        self.segments
            .first()
            .map(|h| h.pixel_pos(alpha))
            .unwrap_or_default()
    }

    pub fn show(&self, alpha: f32, offset: Vec2) {
        if self.segments.is_empty() {
            return;
        }

        let count = self.segments.len();
        let head = self.segments[0].pixel_pos(alpha) + offset;

        // Soft neon halo around the head.
        ui::draw_glow(
            head,
            GRID_SIZE * 1.45,
            Color::new(ACCENT.r, ACCENT.g, ACCENT.b, 90.0 / 255.0),
            0.9,
        );

        // Body first (from tail to neck), then the head on top.
        for i in (1..count).rev() {
            let t = if count > 1 {
                i as f32 / (count - 1) as f32
            } else {
                0.0
            };
            let color = mix(SNAKE_HEAD, theme::SNAKE_TAIL, 0.22 + 0.78 * t);
            self.segments[i].show(color, alpha, offset);
        }

        let head_size = GRID_SIZE - 1.0;
        ui::fill_rounded_rect_centered(
            head.x,
            head.y,
            head_size,
            head_size,
            head_size * 0.38,
            SNAKE_HEAD,
        );

        draw_circle(
            head.x - 2.6,
            head.y - 3.0,
            3.4,
            Color::new(1.0, 1.0, 1.0, 52.0 / 255.0),
        );

        // Eyes look in the last movement direction.
        let look = if self.dir == Direction::Idle {
            self.last_dir
        } else {
            self.dir
        };
        let forward = match look {
            Direction::Up => vec2(0.0, -1.0),
            Direction::Right => vec2(1.0, 0.0),
            Direction::Left => vec2(-1.0, 0.0),
            _ => vec2(0.0, 1.0),
        };
        let side = vec2(-forward.y, forward.x);
        let eye_base = head + forward * 3.0;
        let pupil_offset = forward * 1.1;

        let eye_left = eye_base + side * 3.8;
        let eye_right = eye_base - side * 3.8;
        draw_circle(eye_left.x, eye_left.y, 2.7, Color::new(250.0/255.0, 254.0/255.0, 1.0, 1.0));
        draw_circle(eye_right.x, eye_right.y, 2.7, Color::new(250.0/255.0, 254.0/255.0, 1.0, 1.0));

        let pupil_left = eye_left + pupil_offset;
        let pupil_right = eye_right + pupil_offset;
        draw_circle(pupil_left.x, pupil_left.y, 1.25, Color::new(8.0/255.0, 18.0/255.0, 36.0/255.0, 1.0));
        draw_circle(pupil_right.x, pupil_right.y, 1.25, Color::new(8.0/255.0, 18.0/255.0, 36.0/255.0, 1.0));
    }
}
