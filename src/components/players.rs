use amethyst::ecs::prelude::{Component, DenseVecStorage};

pub struct Player {
    pub id: usize,
    pub kills: i32,
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

impl Player {
    pub fn new(id: usize) -> Player {
        Player {
            id,
            kills: 0,
        }
    }
}