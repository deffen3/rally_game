use amethyst::ecs::prelude::{Component, DenseVecStorage};
use crate::components::{WeaponTypes};

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



pub struct PlayerWeaponIcon {
    pub id: usize,
    pub weapon_type: WeaponTypes,
}

impl Component for PlayerWeaponIcon {
    type Storage = DenseVecStorage<Self>;
}

impl PlayerWeaponIcon {
    pub fn new(id: usize, weapon_type: WeaponTypes) -> PlayerWeaponIcon {
        PlayerWeaponIcon {
            id,
            weapon_type,
        }
    }
}