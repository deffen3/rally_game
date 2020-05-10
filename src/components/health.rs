use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};

#[derive(Clone)]
pub struct Health {
    pub value: f32,
    pub max: f32,
    pub repair_rate: f32,
    pub entity: Entity,
}

impl Component for Health {
    type Storage = DenseVecStorage<Self>;
}
