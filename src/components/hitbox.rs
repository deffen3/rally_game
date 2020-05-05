use amethyst::ecs::prelude::{Component, DenseVecStorage};

#[derive(Clone, Debug, PartialEq)]
pub enum HitboxShape {
    Rectangle,
    Circle,
}

#[derive(Clone)]
pub struct Hitbox {
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    pub shape: HitboxShape,
    pub is_wall: bool,
    pub is_hill: bool,
}

impl Component for Hitbox {
    type Storage = DenseVecStorage<Self>;
}

impl Hitbox {
    pub fn new(width: f32, height: f32, angle: f32, shape: HitboxShape, is_wall: bool, is_hill: bool) -> Hitbox {
        Hitbox {
            width,
            height,
            angle,
            shape,
            is_wall,
            is_hill,
        }
    }
}
