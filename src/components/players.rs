use amethyst::ecs::prelude::{Component, DenseVecStorage};
use crate::components::{WeaponTypes};

pub struct Player {
    pub id: usize,
    pub kills: i32,
    pub is_bot: bool,
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

impl Player {
    pub fn new(id: usize, is_bot: bool) -> Player {
        Player {
            id,
            kills: 0,
            is_bot,
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