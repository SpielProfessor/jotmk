#[derive(PartialEq, Clone, Copy)]
pub enum CollisionType {
    Solid(u32, u32),
    Empty(u32, u32),
}