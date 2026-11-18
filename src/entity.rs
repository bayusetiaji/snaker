//! Basis entity: posisi grid. Padanan dari `entity.h` / `entity.cpp`.

#[derive(Clone, Copy, Debug, Default)]
pub struct Entity {
    pub x: i32,
    pub y: i32,
}

#[allow(dead_code)]
impl Entity {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn set_xy(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }
}
