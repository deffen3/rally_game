use amethyst::ecs::prelude::{Component, DenseVecStorage};

pub struct Particles {
    pub dx: f32,
    pub dy: f32,
    pub life_timer: f32,
}

impl Component for Particles {
    type Storage = DenseVecStorage<Self>;
}