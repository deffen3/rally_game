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

#[derive(Clone)]
pub struct GameModeSetup {
    pub game_mode: GameModes,
    pub match_time_limit: f32, //-1.0 * 60.0; //In seconds. Applies to all games modes. Typically set negative(off) for non Timed matches.
    pub points_to_win: i32, //Applies to all games modes. Typically set negative(off) for Stock or Timed_KD.
    pub stock_lives: i32, //Applies to all games modes. Typically set negative(off) for non Stock battles.
    pub checkpoint_count: i32, //Applies only to Race mode. Must be set equal to the number of checkpoints on the racetrack.
    pub starter_weapon: WeaponNames,
    pub random_weapon_spawns: bool, //Applies to all game modes except GunGame
    pub keep_picked_up_weapons: bool,
    pub weapon_spawn_count: u32,
    pub weapon_spawn_timer: f32,
    pub max_players: usize,
    pub bot_players: usize,
}