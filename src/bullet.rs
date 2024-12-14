use macroquad::prelude::*;
use crate::{GameState, GAME_SCREEN_MAIN};

#[derive(PartialEq, Clone)]
pub struct Bullet {
    pub coords: Vec2,
    pub velocity: Vec2,
}

const BULLET_SPEED: f32 = 2.0;
impl Bullet {
    pub fn new(coords: Vec2, dir: Direction) -> Self {
        let mut velocity = vec2(0., 0.);

        if dir == Direction::Up {
            velocity = vec2(0., -BULLET_SPEED)
        } else if dir == Direction::Down {
            velocity = vec2(0., BULLET_SPEED)
        } else if dir == Direction::Left {
            velocity = vec2(-BULLET_SPEED, 0.)
        } else if dir == Direction::Right {
            velocity = vec2(BULLET_SPEED, 0.)
        } else if dir == Direction::Leftup {
            velocity = vec2(-BULLET_SPEED.sqrt(), -BULLET_SPEED.sqrt());
        } else if dir == Direction::Leftdown {
            velocity = vec2(-BULLET_SPEED.sqrt(), BULLET_SPEED.sqrt());
        } else if dir == Direction::Rightup {
            velocity = vec2(BULLET_SPEED.sqrt(), -BULLET_SPEED.sqrt());
        } else if dir == Direction::Rightdown {
            velocity = vec2(BULLET_SPEED.sqrt(), BULLET_SPEED.sqrt());
        }

        Bullet {
            velocity,
            coords: coords + vec2(8., 8.),
        }
    }
    pub fn update(&mut self) {
        self.coords += self.velocity;
    }
    pub fn draw(&self) {
        draw_circle(self.coords.x + GAME_SCREEN_MAIN.x, self.coords.y + GAME_SCREEN_MAIN.y, 2.0, DARKGRAY);
    }
}

#[derive(PartialEq, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Leftup,
    Leftdown,
    Rightup,
    Rightdown,
}