use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};

#[derive(Clone)]
pub struct Armor {
    pub value: f32,
    pub max: f32,
    pub entity: Entity,
}

impl Component for Armor {
    type Storage = DenseVecStorage<Self>;
}