use amethyst::ecs::prelude::{Component, DenseVecStorage, World};

use ron::de::from_reader;
use serde::Deserialize;
use std::{collections::HashMap, fs::File};

use crate::resources::{GameModes};
use crate::components::{WeaponNames, Hitbox, HitboxShape};


#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum RaceCheckpointType {
    NotCheckpoint,
    Checkpoint,
    Lap,
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

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct WeaponSpawnBox {
    pub x: f32,
    pub y: f32,
    pub weapon_name: Option<WeaponNames>,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct EnemySpawnPoint {
    pub x: f32,
    pub y: f32,
    pub enemy_name: Option<EnemyNames>,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct ArenaCircle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct ArenaRectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32, //degrees
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



#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct ArenaElement {
    pub is_wall: bool,
    pub is_hill: bool,
    pub checkpoint: RaceCheckpointType,
    pub checkpoint_id: i32,
    pub is_weapon_box: bool,
    pub is_spawn_point: bool,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub is_sprite: bool,
    pub sprite: usize,
    pub sprite_scale: f32,
    pub hitbox: Hitbox,
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
    pub arena_rectangles: Vec<ArenaRectangle>, //not implemented yet
    pub weapon_spawn_boxes: Vec<WeaponSpawnBox>,
    pub king_hills: Vec<ArenaKingHill>,
    pub race_checkpoints: Vec<ArenaRaceCheckpoint>,
    pub player_spawn_points: Vec<PlayerSpawnPoint>,
    pub enemy_spawn_points: Vec<EnemySpawnPoint>, //not implemented yet
    pub custom_elements: Vec<ArenaElement>, //not implemented yet
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ArenaNames {
    OpenEmptyMap,
    StandardCombat,
    StandardKingOfTheHill,
    StandardRace,
}



#[derive(Clone)]
pub struct ArenaStoreResource {
    pub properties: HashMap<ArenaNames, ArenaProperties>,
    pub game_modes: HashMap<GameModes, Vec<ArenaNames>>,
}


/* Release rally.exe (crashes):
"\\\\?\\C:\\Users\\Mike\\rust\\amethyst\\rally_game\\target\\release\\assets/game/vehicles.ron"

cargo run
"C:\\Users\\Mike\\rust\\amethyst\\rally_game\\assets/game/vehicles.ron"
*/

pub fn build_arena_store(world: &mut World) {
    // let app_root = current_dir();
    // let input_path = app_root.unwrap().join("assets/game/vehicles.ron");

    let input_path_properties = format!("{}/assets/game/arena_properties.ron", env!("CARGO_MANIFEST_DIR"));
    let input_path_modes = format!("{}/assets/game/arena_game_modes.ron", env!("CARGO_MANIFEST_DIR"));
    
    let f_properties = File::open(&input_path_properties).expect("Failed opening file");
    let f_modes = File::open(&input_path_modes).expect("Failed opening file");

    let arena_properties_map: HashMap<ArenaNames, ArenaProperties> =
        from_reader(f_properties).expect("Failed to load config");
    let arena_game_modes_map: HashMap<GameModes, Vec<ArenaNames>> =
        from_reader(f_modes).expect("Failed to load config");

    let arena_store = ArenaStoreResource {
        properties: arena_properties_map,
        game_modes: arena_game_modes_map,
    };
    world.insert(arena_store.clone());
}



pub fn reform_weapon_spawn_box(spawn_box: WeaponSpawnBox) -> ArenaElement {
    ArenaElement {
        is_wall: false,
        is_hill: false,
        checkpoint: RaceCheckpointType::NotCheckpoint,
        checkpoint_id: 0,
        is_weapon_box: true,
        is_spawn_point: false,
        x: spawn_box.x,
        y: spawn_box.y,
        z: 0.0,
        is_sprite: false,
        sprite: 0,
        sprite_scale: 0.0,
        hitbox: Hitbox {
            width: 11.0,
            height: 11.0,
            angle: 0.0,
            shape: HitboxShape::Rectangle,
        },
    }
}