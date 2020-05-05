use crate::components::WeaponTypes;
use amethyst::ecs::prelude::{Component, DenseVecStorage};

#[derive(Clone, Debug, PartialEq)]
pub enum BotMode {
    Running,
    StopAim,
    CollisionTurn,
    CollisionMove,
    Mining,
    Chasing,
    Swording,
    TakeTheHill,
}

pub struct Player {
    pub id: usize,
    pub kills: i32,
    pub deaths: i32,
    pub earned_kill: bool,
    pub objective_points: f32,
    pub is_bot: bool,
    pub bot_mode: BotMode,
    pub bot_move_cooldown: f32,
    pub bot_move_cooldown_reset: f32,
    pub last_accel_input: Option<f32>,
    pub last_turn_input: Option<f32>,
    pub last_hit_by_id: Option<usize>,
    pub last_hit_timer: f32,
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

impl Player {
    pub fn new(id: usize, is_bot: bool) -> Player {
        Player {
            id,
            kills: 0,
            deaths: 0,
            earned_kill: false,
            objective_points: 0.0,
            is_bot,
            bot_mode: BotMode::StopAim,
            bot_move_cooldown: -1.0,
            bot_move_cooldown_reset: 1.0,
            last_accel_input: Some(0.0),
            last_turn_input: Some(0.0),
            last_hit_by_id: None,
            last_hit_timer: 0.0
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
        PlayerWeaponIcon { id, weapon_type }
    }
}
