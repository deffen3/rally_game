use amethyst::ecs::prelude::{Component, DenseVecStorage, World};

use serde::Deserialize;
use std::collections::HashMap;

use crate::resources::{GameModes};
use crate::components::{WeaponNames, Hitbox, HitboxShape};
use crate::load_ron_asset;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ArenaNames {
    OpenEmptyMap,
    StandardCombat,
    StandardKingOfTheHill,
    StandardRace,
    ChaosCombat,
    LargeCombat,
}




#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum RaceCheckpointType {
    NotCheckpoint,
    Checkpoint,
    Lap,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum ObstacleType {
    Open,
    Wall,
    Zone,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct ZoneEffects {
    pub accel_rate: f32,
    pub damage_rate: f32,
}


#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum EnemyNames {
    AutoTurret,
}


#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct PlayerSpawnPoint {
    pub x: f32,
    pub y: f32,
    pub rotation: f32, //degrees
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct WeaponBoxSpawner {
    pub x: f32,
    pub y: f32,
    pub weapon_names: Option<Vec<(WeaponNames, u32)>>,
    pub first_spawn_time: Option<f32>,
    pub spawn_time: Option<f32>,
    pub ammo: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct WeaponBox {
    pub x: f32,
    pub y: f32,
    pub weapon_names: Option<Vec<(WeaponNames, u32)>>,
    pub ammo: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct EnemySpawnPoint {
    pub x: f32,
    pub y: f32,
    pub enemy_name: Option<EnemyNames>,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct ArenaCircle {
    pub obstacle_type: ObstacleType,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub effect: Option<ZoneEffects>,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct ArenaRectangle {
    pub obstacle_type: ObstacleType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32, //degrees
    pub effects: Option<ZoneEffects>,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct ArenaKingHill {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct ArenaRaceCheckpoint {
    pub x: f32,
    pub y: f32,
    pub length: f32,
    pub rotation: f32, //degrees
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct ArenaFloor {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}



#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ArenaElement {
    pub obstacle_type: ObstacleType,
    pub is_hill: bool,
    pub checkpoint: RaceCheckpointType,
    pub checkpoint_id: i32,
    pub is_weapon_box: bool,
    pub is_spawn_point: bool,
    pub is_weapon_spawn_point: bool,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub is_sprite: bool,
    pub sprite: usize,
    pub sprite_scale: f32,
    pub weapon_names: Option<Vec<(WeaponNames, u32)>>,
    pub first_spawn_time: Option<f32>,
    pub spawn_time: Option<f32>,
    pub spawn_timer: Option<f32>,
    pub ammo: Option<u32>,
    pub hitbox: Hitbox,
    pub effects: Option<ZoneEffects>,
}

impl Component for ArenaElement {
    type Storage = DenseVecStorage<Self>;
}



#[derive(Clone, Debug, PartialEq, Deserialize, Default)]
pub struct ArenaProperties {
    pub width: f32,
    pub height: f32,
    pub floor: Vec<ArenaFloor>,
    pub arena_circles: Vec<ArenaCircle>,
    pub arena_rectangles: Vec<ArenaRectangle>,
    pub weapon_spawners: Vec<WeaponBoxSpawner>,
    pub king_hills: Vec<ArenaKingHill>,
    pub race_checkpoints: Vec<ArenaRaceCheckpoint>,
    pub player_spawn_points: Vec<PlayerSpawnPoint>,
    pub enemy_spawn_points: Vec<EnemySpawnPoint>, //not implemented yet
    pub custom_elements: Vec<ArenaElement>, //not implemented yet
}


#[derive(Clone)]
pub struct ArenaStoreResource {
    pub properties: HashMap<ArenaNames, ArenaProperties>,
    pub game_modes: HashMap<GameModes, Vec<ArenaNames>>,
}


pub fn build_arena_store(world: &mut World) {
    world.insert(ArenaStoreResource {
        properties: load_ron_asset(&["game", "arena_properties.ron"]),
        game_modes: load_ron_asset(&["game", "arena_game_modes.ron"]),
    });
}



pub fn reform_weapon_spawner(spawner: WeaponBoxSpawner) -> ArenaElement {
    ArenaElement {
        obstacle_type: ObstacleType::Open,
        is_hill: false,
        checkpoint: RaceCheckpointType::NotCheckpoint,
        checkpoint_id: 0,
        is_weapon_box: false,
        is_spawn_point: false,
        is_weapon_spawn_point: true,
        x: spawner.x,
        y: spawner.y,
        z: 0.0,
        is_sprite: false,
        sprite: 0,
        sprite_scale: 0.0,
        weapon_names: None,
        first_spawn_time: spawner.first_spawn_time,
        spawn_time: spawner.spawn_time,
        spawn_timer: spawner.first_spawn_time,
        ammo: spawner.ammo,
        hitbox: Hitbox {
            width: 11.0,
            height: 11.0,
            angle: 0.0,
            shape: HitboxShape::Rectangle,
        },
        effects: None,
    }
}