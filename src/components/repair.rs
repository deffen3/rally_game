use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};

#[derive(Clone)]
pub struct Repair {
    pub activated: bool,
    pub init_timer: f32,
    pub init_threshold: f32,
    pub entity: Entity,
}

impl Component for Repair {
    type Storage = DenseVecStorage<Self>;
}