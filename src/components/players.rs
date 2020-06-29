use amethyst::ecs::prelude::{Component, DenseVecStorage};

use crate::components::WeaponFireTypes;

#[derive(Clone, Debug, PartialEq)]
pub enum BotMode {
    Sleep,
    RunTo,
    RunRandom,
    RunBlind,
    FindEnemy,
    TakeTheHill,
    PickUpWeaponBox,
    Racing,
    StopAim,
    StrafeAim,
    CollisionTurn,
    CollisionMove,
    Mining,
    Chasing,
    Swording,
    Repairing,
}

pub struct Player {
    pub id: usize,
    pub team: i32,
    pub kills: i32,
    pub gun_game_kills: i32,
    pub deaths: i32,
    pub earned_collision_kills: u32,
    pub objective_points: f32,
    pub checkpoint_completed: i32,
    pub laps_completed: i32,
    pub on_hill: bool,
    pub is_bot: bool,
    pub bot_mode: BotMode,
    pub bot_move_cooldown: f32,
    pub bot_move_cooldown_reset: f32,
    pub path_target: Option<(f32, f32, f32)>,
    pub path_plan: Option<Vec<(f32, f32, f32)>>,
    pub last_accel_input: Option<f32>,
    pub last_turn_input: Option<f32>,
    pub last_hit_by_id: Option<usize>,
    pub last_hit_timer: f32,
    pub last_made_hit_timer: f32,
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

impl Player {
    pub fn new(id: usize, team: i32, is_bot: bool) -> Player {
        Player {
            id,
            team,
            kills: 0,
            gun_game_kills: 0,
            deaths: 0,
            earned_collision_kills: 0,
            objective_points: 0.0,
            checkpoint_completed: 0,
            laps_completed: 0,
            on_hill: false,
            is_bot,
            bot_mode: BotMode::Sleep,
            bot_move_cooldown: -1.0,
            bot_move_cooldown_reset: 1.0,
            path_target: None,
            path_plan: None,
            last_accel_input: Some(0.0),
            last_turn_input: Some(0.0),
            last_hit_by_id: None,
            last_hit_timer: 0.0,
            last_made_hit_timer: 0.0,
        }
    }
}

pub struct PlayerWeaponIcon {
    pub id: usize,
    pub weapon_fire_type: WeaponFireTypes,
}

impl Component for PlayerWeaponIcon {
    type Storage = DenseVecStorage<Self>;
}

impl PlayerWeaponIcon {
    pub fn new(id: usize, weapon_fire_type: WeaponFireTypes) -> PlayerWeaponIcon {
        PlayerWeaponIcon { id, weapon_fire_type }
    }
}
