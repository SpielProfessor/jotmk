use macroquad::prelude::*;
use crate::bullet::{Bullet, Direction};
use crate::{GameState, GAME_SCREEN_MAIN, SHOOT_COOLDOWN_MAX, SPEED, TILE_SIZE};
use crate::collision::CollisionType;
use crate::items::Item;

#[derive(Clone)]
pub struct Player {
    pub coords: Vec2,
    pub flipped: bool,
    pub wh: Vec2,
    pub effect: Option<Item>,
    pub held_effect: Option<Item>,
    pub quickshoot: i32,
    pub effect_duration: i32,
    pub strength: i32,
    pub health: i32,
}
impl Player {
    pub fn reset_coords(&mut self) {
        self.coords = vec2(GAME_SCREEN_MAIN.w / 2. - 8., GAME_SCREEN_MAIN.h / 2. - 8.);
    }
}
impl Default for Player {
    fn default() -> Self {
        Self {
            coords: vec2(GAME_SCREEN_MAIN.w / 2. - 8., GAME_SCREEN_MAIN.h / 2. - 8.),
            flipped: false,
            wh: vec2(TILE_SIZE, TILE_SIZE),
            effect: None,
            held_effect: None,
            quickshoot: 0,
            effect_duration: 0,
            strength: 1,
            health: 3,
        }
    }
}


pub fn update_fixed(gs: &mut GameState) {
    // effect runtime
    if gs.player.effect_duration > 0 {
        gs.player.effect_duration -= 1;
    }
    if gs.player.effect_duration == 1 {
        gs.player.effect = None;
    }
    // effects
    if let Some(effect) = &gs.player.effect {
        if *effect == Item::Quickshoot && gs.player.quickshoot == 0 {
            gs.player.quickshoot = 12;
        }
    } else {
        if gs.player.quickshoot != 0 {
            gs.player.quickshoot = 0;
        }
    }
}
pub fn key_inputs(gs: &mut GameState, shoot_cooldown: &mut i32, collision_map: &Vec<CollisionType>) {
    let old_coords = gs.player.coords;
    // movement
    if is_key_down(KeyCode::W) && is_key_down(KeyCode::A) {
        gs.player.coords.y -= SPEED.sqrt();
        gs.player.coords.x -= SPEED.sqrt();
        gs.player.flipped = true;
    } else if is_key_down(KeyCode::S) && is_key_down(KeyCode::A) {
        gs.player.coords.y += SPEED.sqrt();
        gs.player.coords.x -= SPEED.sqrt();
        gs.player.flipped = true;
    } else if is_key_down(KeyCode::W) && is_key_down(KeyCode::D) {
        gs.player.coords.y -= SPEED.sqrt();
        gs.player.coords.x += SPEED.sqrt();
        gs.player.flipped = false;
    } else if is_key_down(KeyCode::S) && is_key_down(KeyCode::D) {
        gs.player.coords.y += SPEED.sqrt();
        gs.player.coords.x += SPEED.sqrt();
        gs.player.flipped = false;
    } else if is_key_down(KeyCode::W) {
        gs.player.coords.y -= SPEED
    } else if is_key_down(KeyCode::A) {
        gs.player.coords.x -= SPEED;
        gs.player.flipped = true;
    } else if is_key_down(KeyCode::D) {
        gs.player.coords.x += SPEED;
        gs.player.flipped = false;
    } else if is_key_down(KeyCode::S) {
        gs.player.coords.y += SPEED;
    }

    let pc = gs.player.coords;
    let ps = gs.player.wh;
    for tile in collision_map {
        if let CollisionType::Solid(x, y) = tile {
            // go in a bit collision-wise to make collisions feel better
            if Rect::new(gs.player.coords.x + 2., gs.player.coords.y + 2., gs.player.wh.x - 2., gs.player.wh.y - 2.).overlaps(&Rect::new(*x as f32 * TILE_SIZE, *y as f32 * 16., 16., 16.)) {
                gs.player.coords = old_coords;
            }
        }
    }

    // collision with world's outer borders (x<0, y<0, y>YMAX, x>XMAX)
    if gs.player.coords.x < 0. || gs.player.coords.y < 0. || (gs.player.coords.x + gs.player.wh.x) > GAME_SCREEN_MAIN.w || (gs.player.coords.y + gs.player.wh.y) > GAME_SCREEN_MAIN.h {
        gs.player.coords = old_coords;
    }


    // shooting
    if *shoot_cooldown == 0 {
        if is_key_down(KeyCode::Up) && is_key_down(KeyCode::Left) {
            gs.bullets.push(Bullet::new(gs.player.coords, Direction::Leftup));
            *shoot_cooldown = SHOOT_COOLDOWN_MAX - gs.player.quickshoot;
        } else if is_key_down(KeyCode::Up) && is_key_down(KeyCode::Right) {
            gs.bullets.push(Bullet::new(gs.player.coords, Direction::Rightup));
            *shoot_cooldown = SHOOT_COOLDOWN_MAX - gs.player.quickshoot;
        } else if is_key_down(KeyCode::Down) && is_key_down(KeyCode::Left) {
            gs.bullets.push(Bullet::new(gs.player.coords, Direction::Leftdown));
            *shoot_cooldown = SHOOT_COOLDOWN_MAX - gs.player.quickshoot;
        } else if is_key_down(KeyCode::Down) && is_key_down(KeyCode::Right) {
            gs.bullets.push(Bullet::new(gs.player.coords, Direction::Rightdown));
            *shoot_cooldown = SHOOT_COOLDOWN_MAX - gs.player.quickshoot;
        } else if is_key_down(KeyCode::Up) {
            gs.bullets.push(Bullet::new(gs.player.coords, Direction::Up));
            *shoot_cooldown = SHOOT_COOLDOWN_MAX - gs.player.quickshoot;
        } else if is_key_down(KeyCode::Down) {
            gs.bullets.push(Bullet::new(gs.player.coords, Direction::Down));
            *shoot_cooldown = SHOOT_COOLDOWN_MAX - gs.player.quickshoot;
        } else if is_key_down(KeyCode::Left) {
            gs.bullets.push(Bullet::new(gs.player.coords, Direction::Left));
            *shoot_cooldown = SHOOT_COOLDOWN_MAX - gs.player.quickshoot;
        } else if is_key_down(KeyCode::Right) {
            gs.bullets.push(Bullet::new(gs.player.coords, Direction::Right));
            *shoot_cooldown = SHOOT_COOLDOWN_MAX - gs.player.quickshoot;
        }
    }
}