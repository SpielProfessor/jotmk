use macroquad::prelude::*;
#[derive(Clone, Copy, PartialEq)]
pub enum Item {
    Speed,
    Quickshoot,
}

struct Items {
    name: String,
    texture: Texture2D,
}