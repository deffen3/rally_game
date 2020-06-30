use amethyst::ecs::prelude::{Component, DenseVecStorage};

use serde::Deserialize;





#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum HitboxShape {
    Rectangle,
    Circle,
    InnerQuarterCircle,
    OuterQuarterCircle,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
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