use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};

#[derive(Clone)]
pub struct Repair {
    pub activated: bool,
    pub cooldown_timer: f32,
    pub cooldown_threshold: f32,
    pub entity: Entity,
}

impl Component for Repair {
    type Storage = DenseVecStorage<Self>;
}