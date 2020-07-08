use amethyst::ecs::prelude::{Component, DenseVecStorage};

pub struct Particles {
    pub dx: f32,
    pub dy: f32,
    pub life_timer: f32,
}

impl Component for Particles {
    type Storage = DenseVecStorage<Self>;
}

pub struct Shockwave {
    pub radius: f32,
    pub time: f32,
}

impl Component for Shockwave {
    type Storage = DenseVecStorage<Self>;
}
