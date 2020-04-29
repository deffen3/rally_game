use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};

#[derive(Clone)]
pub struct Shield {
    pub value: f32,
    pub max: f32,
    pub recharge_rate: f32,
    pub cooldown_timer: f32,
    pub cooldown_reset: f32,
    pub repair_timer: f32,
    pub repair_threshold: f32,
    pub radius: f32,
    pub entity: Entity,
}

impl Component for Shield {
    type Storage = DenseVecStorage<Self>;
}