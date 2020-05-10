use crate::components::{
    WeaponNames,
};


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameModes {
    ClassicGunGame, //First to get a kill with each weapon. Weapons are hot-swapped after kills.
    DeathmatchKills, //First to a certain number of kills. New weapons can be picked up from arena.
    DeathmatchStock, //If you run out of lives you are out. Last player alive wins. New weapons can be picked up from arena.
    DeathmatchTimedKD, //Match ends after set time. Kills-Deaths is winner. Self-destructs are minus 2 deaths. New weapons can be picked up from arena.
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


pub const MAX_PLAYERS: usize = 4;
pub const BOT_PLAYERS: usize = MAX_PLAYERS - 1;
*/


#[derive(Clone)]
pub struct GameModeSetup {
    pub game_mode: GameModes,
    pub match_time_limit: f32,
    pub points_to_win: i32,
    pub stock_lives: i32,
    pub checkpoint_count: i32,
    pub starter_weapon: WeaponNames,
    pub random_weapon_spawns: bool,
    pub weapon_spawn_count: u32,
    pub weapon_spawn_timer: f32,
    pub max_players: usize,
    pub bot_players: usize,
}