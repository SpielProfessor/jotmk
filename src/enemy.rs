use macroquad::prelude::*;
use macroquad::rand::gen_range;
use macroUtils::include_texture;
use crate::{GameState, GAME_SCREEN_MAIN, TILE_SIZE};
use crate::bullet::Direction;
use crate::collision::CollisionType;
use crate::player::Player;

#[derive(Copy, Clone, PartialEq)]
enum HorizontalVertical {
    Horizontal,
    Vertical,
}

static mut TEXTURES: Vec<Texture2D> = vec![];
pub async fn initialize_enemy_textures() {
    unsafe {
        // ghost
        TEXTURES.push(include_texture!("../assets/enemy.png"));
    }
}

pub fn get_texture<'a>(id: usize) -> &'a Texture2D {
    unsafe {
        TEXTURES.get(id).unwrap()
    }
}

#[derive(Clone)]
pub struct Enemy {
    pub coords: Vec2,
    velocity: Vec2,
    speed: f32,
    pub wh: Vec2,
    preferred_direction: HorizontalVertical,
    hp: i32,
    pub last_blocked: bool,
    ldir: Direction,
}

impl Enemy {
    pub fn new_random(spawnpoints: &Vec<Vec2>) -> Self {
        let index = gen_range(0, spawnpoints.len());
        let preferred_direction = {
            let vertical = gen_range(0, 2);
            if vertical == 1 {
                HorizontalVertical::Vertical
            } else {
                HorizontalVertical::Horizontal
            }
        };
        Self {
            coords: spawnpoints.get(index).unwrap().clone(),
            velocity: vec2(0., 0.),
            speed: 0.6,
            wh: vec2(15., 15.),
            preferred_direction,
            hp: 2,
            last_blocked: false,
            ldir: Direction::Left,
        }
    }

    /// take damage. If the enemy dies, returns true, else it returns false
    pub fn damage(&mut self, damage: i32) -> bool {
        self.hp -= damage;
        if self.hp <= 0 {
            true
        } else {
            false
        }
    }

    /// update (fixed) for enemies
    /// # TODO: fix enemies studder at corners
    pub fn update(&mut self, player: &Player, collision_map: &Vec<CollisionType>) {

        // movement, "AI"
        // direction to go. Every enemy can prefer either horizontal or vertical movement
        let mut godir: Direction = Direction::Down;
        if self.preferred_direction == HorizontalVertical::Horizontal {
            if player.coords.x.round() > self.coords.x.round() {
                godir = Direction::Right;
            } else if player.coords.x.round() < self.coords.x.round() {
                godir = Direction::Left;
            } else if player.coords.y.round() > self.coords.y.round() {
                godir = Direction::Down;
            } else if player.coords.y.round() < self.coords.y.round() {
                godir = Direction::Up;
            } else {
                godir = Direction::Down;
            }
        } else {
            if player.coords.y.round() > self.coords.y.round() {
                godir = Direction::Down;
            } else if player.coords.y.round() < self.coords.y.round() {
                godir = Direction::Up;
            } else if player.coords.x.round() > self.coords.x.round() {
                godir = Direction::Right;
            } else if player.coords.x.round() < self.coords.x.round() {
                godir = Direction::Left;
            } else {
                godir = Direction::Down;
            }
        }
        // check if way is blocked. If it is, go the other direction.
        if godir == Direction::Left || godir == Direction::Right {
            if get_tile_collisionmap(&self.coords, &godir, collision_map, self.speed) || (self.last_blocked && self.ldir == godir) {
                if player.coords.y.round() >= self.coords.y.round() {
                    godir = Direction::Down;
                } else {
                    godir = Direction::Up;
                }
            }
        } else if godir == Direction::Up || godir == Direction::Down {
            if get_tile_collisionmap(&self.coords, &godir, collision_map, self.speed) || (self.last_blocked && self.ldir == godir) {
                if player.coords.x.round() >= self.coords.x.round() {
                    godir = Direction::Right;
                } else {
                    godir = Direction::Left;
                }
            }
        }
        // perform velocity changes
        if godir == Direction::Left {
            self.velocity = vec2(-self.speed, 0.);
        } else if godir == Direction::Right {
            self.velocity = vec2(self.speed, 0.);
        } else if godir == Direction::Down {
            self.velocity = vec2(0., self.speed);
        } else if godir == Direction::Up {
            self.velocity = vec2(0., -self.speed);
        }
        self.ldir = godir;

        // collision detection, final movement
        self.last_blocked = false;        // if needed, this gets updated later
        let old_coords = self.coords;
        self.coords += self.velocity;
        for tile in collision_map {
            if let CollisionType::Solid(x, y) = tile {
                if Rect::new(self.coords.x + 1., self.coords.y + 1., self.wh.x - 3., self.wh.y - 3.).overlaps(&Rect::new(*x as f32 * TILE_SIZE, *y as f32 * TILE_SIZE, TILE_SIZE, TILE_SIZE)) {
                    self.coords = old_coords;
                    self.last_blocked = true;
                }
            }
        }
    }
    /// draw the enemy. If `takes_damage` is true, show damage animation
    pub fn draw(&self, takes_damage: bool) {
        draw_texture(get_texture(0), self.coords.x + GAME_SCREEN_MAIN.x, self.coords.y + GAME_SCREEN_MAIN.y, if takes_damage { RED } else { WHITE });
    }
}

/// get tile at the coords `coords` and move to `direction` if it isn't diagonal. If the tile collides, return `true`, else `false`
fn get_tile_collisionmap(coords: &Vec2, direction_from_coords: &Direction, tilemap: &Vec<CollisionType>, speed: f32) -> bool {
    for tile in tilemap {
        if let CollisionType::Solid(x, y) = tile {
            let offset_coords = *coords + {
                // offset
                if *direction_from_coords == Direction::Up {
                    vec2(0., -speed)
                } else if *direction_from_coords == Direction::Right {
                    vec2(speed, 0.)
                } else if *direction_from_coords == Direction::Left {
                    vec2(-speed, 0.)
                } else if *direction_from_coords == Direction::Down {
                    vec2(0., speed)
                } else {
                    vec2(0., 0.)
                }
            };
            if (offset_coords.x / TILE_SIZE) as u32 == *x && (offset_coords.y / TILE_SIZE) as u32 == *y {
                return true;
            }
        }
    }
    false
}


