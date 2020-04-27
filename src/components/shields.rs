use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};

#[derive(Clone)]
pub struct Shield {
    pub value: f32,
    pub max: f32,
    pub recharge_rate: f32,
    pub cooldown_timer: f32,
    pub cooldown_reset: f32,
    pub radius: f32,
    pub entity: Entity,
}

impl Component for Shield {
    type Storage = DenseVecStorage<Self>;
}

impl Shield {
    pub fn new(value: f32, max:f32, recharge_rate: f32, 
        cooldown_timer:f32, cooldown_reset:f32, 
        radius: f32,
        entity: Entity,
    ) -> Shield {
        Shield {
            value,
            max,
            recharge_rate,
            cooldown_timer,
            cooldown_reset,
            radius,
            entity,
        }
    }
}