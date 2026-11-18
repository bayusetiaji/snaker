//! State machine, main loop helpers, render, audio, partikel, dan persistensi.
//! Padanan dari `game.cpp` / `game.h`.

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::input::KeyCode;
use macroquad::prelude::*;
use std::fs;

use crate::fruit::Fruit;
use crate::global::{
    clampf, faded, mix, rand_unit,
    theme::{self, ACCENT, DANGER, GOLD},
    Direction, NUM_GRID_H, NUM_GRID_V, SCR_HEIGHT, SCR_WIDTH, WORLD_PAD_LEFT, WORLD_PAD_TOP,
    GRID_SIZE,
};
use crate::snake::Snake;
use crate::ui::{self, Align};

const K_FIELD_LEFT: f32 = WORLD_PAD_LEFT;
const K_FIELD_TOP: f32 = WORLD_PAD_TOP;
const K_FIELD_WIDTH: f32 = NUM_GRID_H as f32 * GRID_SIZE;
const K_FIELD_HEIGHT: f32 = NUM_GRID_V as f32 * GRID_SIZE;
const K_HUD_HEIGHT: f32 = WORLD_PAD_TOP - 8.0;
const BEST_FILE: &str = "hiscore.dat";
const SOUND_VOLUME: f32 = 0.62;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
    life: f32,
    max_life: f32,
    size: f32,
    color: Color,
}

pub struct Game {
    state: State,
    score: i32,
    hiscore: i32,
    record_at_start: i32,
    new_record: bool,

    elapsed: f32,
    state_time: f32,

    shake_time: f32,
    shake_duration: f32,
    shake_magnitude: f32,

    snake: Snake,
    fruit: Fruit,
    particles: Vec<Particle>,

    font: Font,
    sound_pick: Sound,
    sound_hit: Sound,
}

impl Game {
    pub async fn new() -> Self {
        let font = load_ttf_font_from_bytes(include_bytes!("../assets/Exo-DemiBold.otf"))
            .expect("Gagal memuat font Exo-DemiBold.otf");
        let sound_pick = load_sound_from_bytes(include_bytes!("../assets/pick.wav"))
            .await
            .expect("Gagal memuat pick.wav");
        let sound_hit = load_sound_from_bytes(include_bytes!("../assets/hit.wav"))
            .await
            .expect("Gagal memuat hit.wav");

        let hiscore = Self::load_best();
        let snake = Snake::new();
        let fruit = Fruit::new(1, 1);

        let mut game = Self {
            state: State::Menu,
            score: 0,
            hiscore,
            record_at_start: 0,
            new_record: false,
            elapsed: 0.0,
            state_time: 0.0,
            shake_time: 0.0,
            shake_duration: 0.3,
            shake_magnitude: 6.0,
            snake,
            fruit,
            particles: Vec::new(),
            font,
            sound_pick,
            sound_hit,
        };
        game.spawn_fruit();
        game
    }

    fn load_best() -> i32 {
        match fs::read_to_string(BEST_FILE) {
            Ok(text) => text.trim().parse::<i32>().unwrap_or(0).max(0),
            Err(_) => 0,
        }
    }

    pub fn save_best(&self) {
        let _ = fs::write(BEST_FILE, self.hiscore.to_string());
    }

    fn speed_level(&self) -> i32 {
        1 + (self.score / 4).min(7)
    }

    fn tick_ms(&self) -> i32 {
        190 - (self.speed_level() - 1) * 13
    }

    fn play(&self, sound: &Sound, volume: f32) {
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }

    fn stop_sounds(&self) {
        stop_sound(&self.sound_pick);
        stop_sound(&self.sound_hit);
    }

    fn add_shake(&mut self, duration: f32, magnitude: f32) {
        self.shake_time = duration;
        self.shake_duration = duration;
        self.shake_magnitude = magnitude;
    }

    fn update_shake(&mut self, dt: f32) {
        self.shake_time = (self.shake_time - dt).max(0.0);
    }

    fn shake_offset(&self) -> Vec2 {
        if self.shake_time <= 0.0 {
            return vec2(0.0, 0.0);
        }
        let k = clampf(self.shake_time / self.shake_duration, 0.0, 1.0);
        vec2(
            (rand_unit() * 2.0 - 1.0) * self.shake_magnitude * k,
            (rand_unit() * 2.0 - 1.0) * self.shake_magnitude * k,
        )
    }

    fn spawn_burst(&mut self, position: Vec2, color: Color, count: i32, speed: f32) {
        for _ in 0..count {
            let angle = rand_unit() * std::f32::consts::TAU;
            let magnitude = speed * (0.35 + rand_unit() * 0.9);
            let mut particle_color = color;
            if rand_unit() < 0.25 {
                particle_color = Color::new(1.0, 1.0, 1.0, 1.0);
            }
            let max_life = 0.4 + rand_unit() * 0.5;
            self.particles.push(Particle {
                position,
                velocity: vec2(angle.cos() * magnitude, angle.sin() * magnitude),
                life: max_life,
                max_life,
                size: 1.4 + rand_unit() * 2.6,
                color: particle_color,
            });
        }
    }

    fn update_particles(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.particles.len() {
            let p = &mut self.particles[i];
            p.life -= dt;
            if p.life <= 0.0 {
                self.particles.swap_remove(i);
                continue;
            }
            p.position += p.velocity * dt;
            p.velocity.y += 80.0 * dt;
            p.velocity *= (1.0 - dt * 1.8).max(0.0);
            i += 1;
        }
    }

    fn draw_particles(&self, offset: Vec2) {
        for p in &self.particles {
            let t = clampf(p.life / p.max_life, 0.0, 1.0);
            let radius = p.size * (0.35 + 0.65 * t);
            let position = p.position + offset;
            draw_circle(
                position.x,
                position.y,
                radius,
                Color::new(p.color.r, p.color.g, p.color.b, t),
            );
        }
    }

    fn spawn_fruit(&mut self) {
        for _ in 0..400 {
            let x = 1 + rand_unit_int(NUM_GRID_H - 2);
            let y = 1 + rand_unit_int(NUM_GRID_V - 2);
            if !self.snake.occupies(x, y) {
                self.fruit.locate(x, y);
                return;
            }
        }
        self.fruit.locate(1, 1);
    }

    fn start_game(&mut self) {
        self.score = 0;
        self.record_at_start = self.hiscore;
        self.new_record = false;
        self.state_time = 0.0;
        self.shake_time = 0.0;
        self.particles.clear();

        self.snake.reset(15, 15);
        self.spawn_fruit();

        self.stop_sounds();
        self.state = State::Playing;
    }

    fn go_menu(&mut self) {
        self.score = 0;
        self.state_time = 0.0;
        self.shake_time = 0.0;
        self.particles.clear();

        self.snake.reset(15, 15);
        self.spawn_fruit();
        self.fruit.animate(0.0);

        self.stop_sounds();
        self.state = State::Menu;
    }

    fn pause_game(&mut self) {
        if self.state != State::Playing {
            return;
        }
        self.state = State::Paused;
        self.state_time = 0.0;
    }

    fn resume_game(&mut self) {
        if self.state != State::Paused {
            return;
        }
        self.state = State::Playing;
        self.state_time = 0.0;
    }

    fn die(&mut self) {
        self.state = State::GameOver;
        self.state_time = 0.0;

        let head = self.snake.head_pixel(self.snake.alpha());
        self.spawn_burst(head, DANGER, 38, 210.0);
        self.spawn_burst(head, ACCENT, 20, 130.0);
        self.add_shake(0.38, 9.0);

        self.new_record = self.score > self.record_at_start && self.score > 0;
        if self.score > self.hiscore {
            self.hiscore = self.score;
        }
        self.save_best();

        self.stop_sounds();
        self.play(&self.sound_hit, SOUND_VOLUME);
    }

    fn eat_fruit(&mut self) {
        let position = self.fruit.pixel_center();

        self.snake.grow();
        self.spawn_fruit();

        self.score += 1;
        if self.score > self.hiscore {
            self.hiscore = self.score;
        }

        self.spawn_burst(position, theme::FRUIT, 18, 150.0);
        self.spawn_burst(position, GOLD, 5, 85.0);
        self.add_shake(0.08, 2.0);

        self.stop_sounds();
        self.play(&self.sound_pick, SOUND_VOLUME);
    }

    /// Mengembalikan `true` jika pemain meminta keluar (Escape di menu).
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match self.state {
            State::Menu => {
                if key == KeyCode::Enter || key == KeyCode::Space {
                    self.start_game();
                } else if key == KeyCode::Escape {
                    return true;
                } else if let Some(direction) = direction_from_key(key) {
                    self.start_game();
                    self.snake.turn(direction);
                }
            }
            State::Playing => {
                if let Some(direction) = direction_from_key(key) {
                    self.snake.turn(direction);
                } else if key == KeyCode::P || key == KeyCode::Escape {
                    self.pause_game();
                } else if key == KeyCode::R {
                    self.start_game();
                }
            }
            State::Paused => {
                if key == KeyCode::P || key == KeyCode::Escape || key == KeyCode::Enter {
                    self.resume_game();
                } else if key == KeyCode::R {
                    self.start_game();
                } else if key == KeyCode::M {
                    self.go_menu();
                }
            }
            State::GameOver => {
                if key == KeyCode::Enter || key == KeyCode::Space || key == KeyCode::R {
                    self.start_game();
                } else if key == KeyCode::Escape || key == KeyCode::M {
                    self.go_menu();
                }
            }
        }
        false
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
        self.state_time += dt;

        if self.state == State::Menu {
            self.fruit.animate(dt);
            return;
        }
        if self.state == State::Paused {
            return;
        }

        self.update_particles(dt);
        self.update_shake(dt);

        if self.state == State::Playing {
            self.snake.update(dt * 1000.0, self.tick_ms());
            self.fruit.update(dt);

            if self.snake.hit() {
                self.die();
            } else if self.snake.eat(&self.fruit) {
                self.eat_fruit();
            } else if self.fruit.expired() {
                self.spawn_fruit();
            }
        }
    }

    fn draw_grid(&self, offset: Vec2) {
        let left = K_FIELD_LEFT + 10.0;
        let top = K_FIELD_TOP + 10.0;
        let right = K_FIELD_LEFT + K_FIELD_WIDTH - 10.0;
        let bottom = K_FIELD_TOP + K_FIELD_HEIGHT - 10.0;

        for i in 0..NUM_GRID_H {
            let x = left + i as f32 * GRID_SIZE;
            let mut color = theme::BORDER;
            color.a = if i % 5 == 0 { 28.0 / 255.0 } else { 12.0 / 255.0 };
            draw_line(
                x + offset.x,
                top + offset.y,
                x + offset.x,
                bottom + offset.y,
                1.0,
                color,
            );
        }
        for i in 0..NUM_GRID_V {
            let y = top + i as f32 * GRID_SIZE;
            let mut color = theme::BORDER;
            color.a = if i % 5 == 0 { 28.0 / 255.0 } else { 12.0 / 255.0 };
            draw_line(
                left + offset.x,
                y + offset.y,
                right + offset.x,
                y + offset.y,
                1.0,
                color,
            );
        }
    }

    fn draw_field(&self, offset: Vec2) {
        ui::draw_panel(
            K_FIELD_LEFT,
            K_FIELD_TOP,
            K_FIELD_WIDTH,
            K_FIELD_HEIGHT,
            16.0,
            theme::FIELD,
            theme::BORDER,
            1.0,
        );

        self.draw_grid(offset);
        self.fruit.show(offset);
        self.snake.show(self.snake.alpha(), offset);
        self.draw_particles(offset);

        ui::draw_panel(
            K_FIELD_LEFT,
            K_FIELD_TOP,
            K_FIELD_WIDTH,
            K_FIELD_HEIGHT,
            16.0,
            Color::new(0.0, 0.0, 0.0, 0.0),
            faded(theme::BORDER_BRIGHT, 130.0 / 255.0),
            2.0,
        );
    }

    fn draw_hud(&self) {
        draw_rectangle(0.0, 0.0, SCR_WIDTH, K_HUD_HEIGHT, theme::PANEL);
        draw_rectangle(0.0, K_HUD_HEIGHT - 1.0, SCR_WIDTH, 1.0, theme::BORDER);

        ui::draw_text(
            &self.font, "SCORE", 10, theme::TEXT_MUTED, 24.0, 15.0, Align::Left, 1.8,
        );
        ui::draw_text(
            &self.font,
            &self.score.to_string(),
            30,
            theme::TEXT,
            24.0,
            29.0,
            Align::Left,
            1.0,
        );

        ui::draw_text(
            &self.font, "BEST", 10, theme::TEXT_MUTED, 616.0, 15.0, Align::Right, 1.8,
        );
        ui::draw_text(
            &self.font,
            &self.hiscore.to_string(),
            30,
            GOLD,
            616.0,
            29.0,
            Align::Right,
            1.0,
        );

        ui::draw_text(
            &self.font, "SPEED", 10, theme::TEXT_MUTED, 320.0, 15.0, Align::Center, 1.8,
        );

        let level = self.speed_level();
        let seg_width = 13.0;
        let seg_gap = 4.0;
        let total = 8.0 * seg_width + 7.0 * seg_gap;
        let start_x = 320.0 - total * 0.5;
        for i in 0..8 {
            let active = i < level;
            ui::fill_rounded_rect(
                start_x + i as f32 * (seg_width + seg_gap),
                36.0,
                seg_width,
                5.0,
                2.5,
                if active {
                    ACCENT
                } else {
                    theme::PANEL_ALT
                },
            );
        }

        ui::draw_text(
            &self.font,
            &format!("LENGTH {}", self.snake.length()),
            9,
            faded(theme::TEXT_MUTED, 150.0 / 255.0),
            320.0,
            52.0,
            Align::Center,
            1.6,
        );
    }

    fn draw_footer(&self) {
        draw_rectangle(
            20.0,
            K_FIELD_TOP + K_FIELD_HEIGHT + 10.0,
            SCR_WIDTH - 40.0,
            1.0,
            faded(theme::BORDER, 90.0 / 255.0),
        );
        let y = K_FIELD_TOP + K_FIELD_HEIGHT + 26.0;
        ui::draw_text(
            &self.font,
            "ARROWS / WASD   MOVE",
            10,
            theme::TEXT_MUTED,
            24.0,
            y,
            Align::Left,
            1.4,
        );
        ui::draw_text(
            &self.font,
            "P / ESC   PAUSE",
            10,
            theme::TEXT_MUTED,
            320.0,
            y,
            Align::Center,
            1.4,
        );
        ui::draw_text(
            &self.font,
            "R   RESTART",
            10,
            theme::TEXT_MUTED,
            616.0,
            y,
            Align::Right,
            1.4,
        );
    }

    fn draw_idle_hint(&self) {
        if self.state != State::Playing || self.snake.direction() != Direction::Idle {
            return;
        }
        ui::draw_panel(
            170.0,
            126.0,
            300.0,
            48.0,
            12.0,
            Color::new(10.0 / 255.0, 15.0 / 255.0, 30.0 / 255.0, 238.0 / 255.0),
            Color::new(52.0 / 255.0, 226.0 / 255.0, 1.0, 90.0 / 255.0),
            1.0,
        );
        ui::draw_text(
            &self.font,
            "PRESS AN ARROW KEY TO START",
            12,
            ui::pulse_color(ACCENT, self.elapsed, 0.65, 1.0),
            320.0,
            143.0,
            Align::Center,
            1.7,
        );
    }

    fn draw_menu(&self) {
        draw_rectangle(
            0.0,
            0.0,
            SCR_WIDTH,
            SCR_HEIGHT,
            Color::new(4.0 / 255.0, 7.0 / 255.0, 15.0 / 255.0, 176.0 / 255.0),
        );
        ui::draw_glow(
            vec2(320.0, 132.0),
            190.0,
            Color::new(52.0 / 255.0, 226.0 / 255.0, 1.0, 44.0 / 255.0),
            1.0,
        );

        ui::draw_text(
            &self.font, "SNAKE", 76, Color::new(0.0, 0.0, 0.0, 120.0 / 255.0),
            321.0, 103.0, Align::Center, 1.65,
        );
        ui::draw_text(
            &self.font, "SNAKE", 76, theme::TEXT, 320.0, 100.0, Align::Center, 1.65,
        );
        ui::draw_text(
            &self.font, "NEON EDITION", 12, ACCENT, 320.0, 184.0, Align::Center, 3.0,
        );

        // Decorative snake made of glowing beads.
        const BEADS: i32 = 9;
        let start_x = 320.0 - (BEADS - 1) as f32 * 9.5;
        for i in 0..BEADS {
            let x = start_x + i as f32 * 19.0;
            let y = 244.0 + (self.elapsed * 2.2 - i as f32 * 0.5).sin() * 8.0;
            let t = i as f32 / (BEADS - 1) as f32;
            ui::fill_rounded_rect_centered(
                x,
                y,
                16.0,
                16.0,
                6.0,
                mix(theme::SNAKE_HEAD, theme::SNAKE_TAIL, 0.25 + t * 0.75),
            );
            if i == BEADS - 1 {
                ui::draw_glow(
                    vec2(x, y),
                    18.0,
                    Color::new(52.0 / 255.0, 226.0 / 255.0, 1.0, 70.0 / 255.0),
                    0.9,
                );
                draw_circle(
                    x + 2.0,
                    y - 2.0,
                    2.0,
                    Color::new(250.0 / 255.0, 254.0 / 255.0, 1.0, 1.0),
                );
            }
        }

        ui::draw_panel(
            110.0,
            300.0,
            420.0,
            250.0,
            18.0,
            Color::new(16.0 / 255.0, 23.0 / 255.0, 42.0 / 255.0, 242.0 / 255.0),
            theme::BORDER,
            1.0,
        );

        ui::draw_text(
            &self.font, "MOVE", 11, theme::TEXT_MUTED, 136.0, 330.0, Align::Left, 1.8,
        );
        let move_keys = ["W", "A", "S", "D"];
        for (i, key) in move_keys.iter().enumerate() {
            ui::draw_keycap(
                &self.font,
                key,
                380.0 + i as f32 * 31.0,
                322.0,
                28.0,
                28.0,
                false,
            );
        }

        ui::draw_text(
            &self.font, "PAUSE", 11, theme::TEXT_MUTED, 136.0, 378.0, Align::Left, 1.8,
        );
        ui::draw_keycap(&self.font, "P", 428.0, 370.0, 30.0, 28.0, false);
        ui::draw_keycap(&self.font, "ESC", 462.0, 370.0, 44.0, 28.0, false);

        ui::draw_text(
            &self.font, "RESTART", 11, theme::TEXT_MUTED, 136.0, 426.0, Align::Left, 1.8,
        );
        ui::draw_keycap(&self.font, "R", 478.0, 418.0, 28.0, 28.0, false);

        ui::draw_text(
            &self.font,
            "ARROW KEYS WORK TOO",
            9,
            faded(theme::TEXT_MUTED, 145.0 / 255.0),
            320.0,
            462.0,
            Align::Center,
            2.0,
        );
        draw_rectangle(134.0, 484.0, 372.0, 1.0, theme::BORDER);

        ui::draw_text(
            &self.font,
            "PRESS ENTER TO START",
            15,
            ui::pulse_color(ACCENT, self.elapsed, 0.6, 1.0),
            320.0,
            497.0,
            Align::Center,
            2.2,
        );
        ui::draw_text(
            &self.font, "BEST", 10, theme::TEXT_MUTED, 320.0, 580.0, Align::Center, 2.0,
        );
        ui::draw_text(
            &self.font,
            &self.hiscore.to_string(),
            22,
            GOLD,
            320.0,
            594.0,
            Align::Center,
            1.0,
        );
        ui::draw_text(
            &self.font,
            "EAT THE APPLES. AVOID THE WALLS AND YOUR OWN TAIL.",
            9,
            faded(theme::TEXT_MUTED, 130.0 / 255.0),
            320.0,
            648.0,
            Align::Center,
            1.25,
        );
        ui::draw_text(
            &self.font,
            "MACROQUAD  /  NEON EDITION",
            8,
            faded(theme::TEXT_MUTED, 80.0 / 255.0),
            320.0,
            716.0,
            Align::Center,
            1.6,
        );
    }

    fn draw_pause(&self) {
        draw_rectangle(
            0.0,
            0.0,
            SCR_WIDTH,
            SCR_HEIGHT,
            Color::new(4.0 / 255.0, 7.0 / 255.0, 15.0 / 255.0, 158.0 / 255.0),
        );
        ui::draw_glow(
            vec2(320.0, 330.0),
            120.0,
            Color::new(52.0 / 255.0, 226.0 / 255.0, 1.0, 32.0 / 255.0),
            1.0,
        );
        ui::draw_panel(
            170.0,
            260.0,
            300.0,
            230.0,
            18.0,
            Color::new(16.0 / 255.0, 23.0 / 255.0, 42.0 / 255.0, 245.0 / 255.0),
            theme::BORDER,
            1.0,
        );
        ui::draw_text(
            &self.font, "PAUSED", 36, theme::TEXT, 320.0, 290.0, Align::Center, 1.8,
        );
        ui::draw_text(
            &self.font, "TAKE A BREATH", 9, theme::TEXT_MUTED, 320.0, 340.0, Align::Center, 2.5,
        );
        draw_rectangle(202.0, 364.0, 236.0, 1.0, theme::BORDER);
        ui::draw_text(
            &self.font, "P / ESC    RESUME", 12, theme::TEXT, 320.0, 384.0, Align::Center, 1.5,
        );
        ui::draw_text(
            &self.font, "R    RESTART", 12, theme::TEXT_MUTED, 320.0, 414.0, Align::Center, 1.5,
        );
        ui::draw_text(
            &self.font, "M    MAIN MENU", 12, theme::TEXT_MUTED, 320.0, 444.0, Align::Center, 1.5,
        );
    }

    fn draw_game_over(&self) {
        draw_rectangle(
            0.0,
            0.0,
            SCR_WIDTH,
            SCR_HEIGHT,
            Color::new(4.0 / 255.0, 7.0 / 255.0, 15.0 / 255.0, 184.0 / 255.0),
        );
        ui::draw_glow(
            vec2(320.0, 300.0),
            150.0,
            Color::new(255.0 / 255.0, 84.0 / 255.0, 112.0 / 255.0, 40.0 / 255.0),
            1.0,
        );
        ui::draw_panel(
            110.0,
            200.0,
            420.0,
            360.0,
            18.0,
            Color::new(16.0 / 255.0, 23.0 / 255.0, 42.0 / 255.0, 247.0 / 255.0),
            Color::new(255.0 / 255.0, 84.0 / 255.0, 112.0 / 255.0, 120.0 / 255.0),
            1.0,
        );
        ui::draw_text(
            &self.font, "RUN ENDED", 9, theme::TEXT_MUTED, 320.0, 228.0, Align::Center, 2.5,
        );
        ui::draw_text(
            &self.font, "GAME OVER", 42, DANGER, 320.0, 246.0, Align::Center, 1.5,
        );

        if self.new_record {
            ui::draw_panel(
                252.0,
                307.0,
                136.0,
                24.0,
                12.0,
                Color::new(255.0 / 255.0, 205.0 / 255.0, 92.0 / 255.0, 28.0 / 255.0),
                Color::new(255.0 / 255.0, 205.0 / 255.0, 92.0 / 255.0, 130.0 / 255.0),
                1.0,
            );
            ui::draw_text(
                &self.font, "NEW RECORD", 9, GOLD, 320.0, 314.0, Align::Center, 2.4,
            );
        }

        ui::draw_text(
            &self.font, "FINAL SCORE", 9, theme::TEXT_MUTED, 320.0, 350.0, Align::Center, 2.4,
        );
        ui::draw_text(
            &self.font,
            &self.score.to_string(),
            54,
            theme::TEXT,
            320.0,
            364.0,
            Align::Center,
            1.0,
        );
        ui::draw_text(
            &self.font,
            &format!("BEST  {}", self.hiscore),
            12,
            GOLD,
            320.0,
            432.0,
            Align::Center,
            1.8,
        );
        draw_rectangle(170.0, 458.0, 300.0, 1.0, theme::BORDER);
        ui::draw_text(
            &self.font,
            "PRESS ENTER TO PLAY AGAIN",
            13,
            ui::pulse_color(theme::TEXT, self.elapsed, 0.62, 1.0),
            320.0,
            478.0,
            Align::Center,
            1.7,
        );
        ui::draw_text(
            &self.font, "ESC    MAIN MENU", 11, theme::TEXT_MUTED, 320.0, 512.0, Align::Center, 1.7,
        );
    }

    pub fn render(&self) {
        ui::draw_background(self.elapsed);
        let offset = self.shake_offset();

        if self.state == State::Menu {
            self.draw_field(offset);
            self.draw_menu();
            return;
        }

        self.draw_hud();
        self.draw_field(offset);
        self.draw_footer();
        self.draw_idle_hint();

        if self.state == State::Paused {
            self.draw_pause();
        } else if self.state == State::GameOver {
            self.draw_game_over();
        }
    }
}

fn direction_from_key(key: KeyCode) -> Option<Direction> {
    match key {
        KeyCode::Up | KeyCode::W => Some(Direction::Up),
        KeyCode::Right | KeyCode::D => Some(Direction::Right),
        KeyCode::Down | KeyCode::S => Some(Direction::Down),
        KeyCode::Left | KeyCode::A => Some(Direction::Left),
        _ => None,
    }
}

fn rand_unit_int(n: i32) -> i32 {
    crate::global::rand_int(n)
}
