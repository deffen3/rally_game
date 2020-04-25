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
}

impl Component for Hitbox {
    type Storage = DenseVecStorage<Self>;
}

impl Hitbox {
    pub fn new(
        width: f32,
        height: f32,
        angle: f32,
        shape: HitboxShape,
    ) -> Hitbox {
        Hitbox {
            width,
            height,
            angle,
            shape,
        }
    }
}