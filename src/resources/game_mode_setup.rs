use crate::components::{
    WeaponNames,
};


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameModes {
    ClassicGunGame, //First to get a kill with each weapon. Weapons are hot-swapped after kills.
    Deathmatch_Kills, //First to a certain number of kills. New weapons can be picked up from arena.
    Deathmatch_Stock, //If you run out of lives you are out. Last player alive wins. New weapons can be picked up from arena.
    Deathmatch_Timed_KD, //Match ends after set time. Kills-Deaths is winner. Self-destructs are minus 2 deaths. New weapons can be picked up from arena.
    Race,
    KingOfTheHill, //Player gains points for being the only person in the special "hill" zone. First player to a certain number of points wins. New weapons can be picked up from arena.
}


/*
pub const GAME_MODE: GameModes = GameModes::KingOfTheHill;


pub const MATCH_TIME_LIMIT: f32 = -1.0 * 60.0; //In seconds. Applies to all games modes. Typically set negative(off) for non Timed matches.

pub const POINTS_TO_WIN: i32 = 100; //Applies to all games modes. Typically set negative(off) for Stock or Timed_KD.

pub const STOCK_LIVES: i32 = -1; //Applies to all games modes. Typically set negative(off) for non Stock battles.

pub const CHECKPOINT_COUNT: i32 = 2; //Applies only to Race mode. Must be set equal to the number of checkpoints on the racetrack.

pub const STARTER_WEAPON: WeaponNames = WeaponNames::LaserDoubleGimballed;

pub const RANDOM_WEAPON_SPAWNS: bool = true; //Applies to all game modes except GunGame
*/

#[derive(Clone)]
pub struct GameModeSetup {
    game_mode: GameModes,
    match_time_limit: f32,
    points_to_win: i32,
    stock_lives: i32,
    checkpoint_count: i32,
    starter_weapon: WeaponNames,
    random_weapon_spawns: bool,
}