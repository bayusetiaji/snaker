//! Apel: animasi idle, masa hidup, dan rendering. Padanan dari `fruit.cpp`.

use macroquad::prelude::*;

use crate::entity::Entity;
use crate::global::{
    clampf,
    theme::{self, FRUIT, LEAF},
    GRID_SIZE, NUM_GRID_H, NUM_GRID_V, WORLD_PAD_LEFT, WORLD_PAD_TOP,
};
use crate::ui;

const LIFETIME: f32 = 9.0;

pub struct Fruit {
    pub entity: Entity,
    life: f32,
    anim: f32,
}

impl Fruit {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            entity: Entity::new(x, y),
            life: LIFETIME,
            anim: 0.0,
        }
    }

    pub fn x(&self) -> i32 {
        self.entity.x
    }

    pub fn y(&self) -> i32 {
        self.entity.y
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.entity.x = crate::global::rand_int(NUM_GRID_H - 2) + 1;
        self.entity.y = crate::global::rand_int(NUM_GRID_V - 2) + 1;
        self.life = LIFETIME;
    }

    pub fn locate(&mut self, x: i32, y: i32) {
        self.entity.set_xy(x, y);
        self.life = LIFETIME;
    }

    /// Advances the idle animation only.
    pub fn animate(&mut self, dt: f32) {
        self.anim += dt;
    }

    /// Advances the animation and the remaining lifetime.
    pub fn update(&mut self, dt: f32) {
        self.anim += dt;
        self.life -= dt;
    }

    pub fn expired(&self) -> bool {
        self.life <= 0.0
    }

    pub fn life_ratio(&self) -> f32 {
        if LIFETIME <= 0.0 {
            0.0
        } else {
            clampf(self.life / LIFETIME, 0.0, 1.0)
        }
    }

    pub fn pixel_center(&self) -> Vec2 {
        vec2(
            self.entity.x as f32 * GRID_SIZE + WORLD_PAD_LEFT + GRID_SIZE * 0.5,
            self.entity.y as f32 * GRID_SIZE + WORLD_PAD_TOP + GRID_SIZE * 0.5,
        )
    }

    pub fn show(&self, offset: Vec2) {
        let center = self.pixel_center() + offset;
        let pulse = 1.0 + 0.075 * (self.anim * 5.2).sin();
        let bob = (self.anim * 3.1).sin() * 1.1;
        let position = vec2(center.x, center.y + bob);

        let urgency = self.life_ratio();
        let glow = if urgency < 0.25 {
            0.75 + 0.25 * (self.anim * 15.0).sin()
        } else {
            1.0
        };

        ui::draw_glow(
            position,
            21.0 * pulse,
            Color::new(FRUIT.r, FRUIT.g, FRUIT.b, 110.0 / 255.0),
            0.8 * glow,
        );

        let mut body_color = FRUIT;
        if urgency < 0.25 {
            body_color.a =
                (150.0 + 105.0 * (0.5 + 0.5 * (self.anim * 15.0).sin())) / 255.0;
        }
        draw_circle(
            position.x,
            position.y,
            GRID_SIZE * 0.34 * pulse,
            body_color,
        );

        draw_circle(
            position.x - 2.4,
            position.y - 2.8,
            2.0,
            Color::new(1.0, 1.0, 1.0, 175.0 / 255.0),
        );

        // Daun: elips kecil dimiringkan (disimulasikan dengan ellipsoid).
        let leaf_pos = position + vec2(3.0, -6.0);
        draw_ellipse(
            leaf_pos.x,
            leaf_pos.y,
            2.9 * 2.0 * 1.6,
            2.9 * 2.0 * 0.72,
            -28.0,
            LEAF,
        );

        let _ = theme::FRUIT;
    }
}
