use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder, UiText},
    winit::VirtualKeyCode,
};

use std::collections::HashMap;

use crate::rally::GameplayState;
use crate::welcome::WelcomeScreen;

use crate::components::WeaponNames;
use crate::resources::{GameModeSetup, GameModes};


pub const MAX_PLAYER_COUNT: usize = 4;
pub const MIN_PLAYER_COUNT: usize = 1;
pub const MIN_BOT_COUNT: usize = 0;

pub const INIT_PLAYER_COUNT: usize = 4;
pub const INIT_BOT_COUNT: usize = INIT_PLAYER_COUNT-1;


const BUTTON_CLASSIC_GUN_GAME: &str = "classic_gun_game";
const BUTTON_DEATHMATCH_KILLS: &str = "deathmatch_kills";
const BUTTON_DEATHMATCH_STOCK: &str = "deathmatch_stock";
const BUTTON_DEATHMATCH_TIME: &str = "deathmatch_time";
const BUTTON_KING_OF_THE_HILL: &str = "king_of_the_hill";
const BUTTON_COMBAT_RACE: &str = "combat_race";
const BUTTON_START_GAME: &str = "start_game";
const EDIT_TEXT_PLAYER_COUNT: &str = "player_count_field";
const EDIT_TEXT_BOT_COUNT: &str = "bot_count_field";


#[derive(Default, Debug)]
pub struct MainMenu {
    ui_root: Option<Entity>,
    button_classic_gun_game: Option<Entity>,
    button_deathmatch_kills: Option<Entity>,
    button_deathmatch_stock: Option<Entity>,
    button_deathmatch_time: Option<Entity>,
    button_king_of_the_hill: Option<Entity>,
    button_combat_race: Option<Entity>,
    button_start_game: Option<Entity>,
    edit_text_player_count: Option<Entity>,
    edit_text_bot_count: Option<Entity>,
}

impl SimpleState for MainMenu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        let mut weapon_spawn_relative_chance_map = HashMap::new();
        weapon_spawn_relative_chance_map.insert(WeaponNames::LaserDoubleGimballed, 0);
        weapon_spawn_relative_chance_map.insert(WeaponNames::LaserDoubleBurstSide, 4);
        weapon_spawn_relative_chance_map.insert(WeaponNames::LaserPulseGimballed, 8);
        weapon_spawn_relative_chance_map.insert(WeaponNames::LaserBeam, 8);
        weapon_spawn_relative_chance_map.insert(WeaponNames::Shotgun, 8);
        weapon_spawn_relative_chance_map.insert(WeaponNames::ProjectileCannonFire, 8);
        weapon_spawn_relative_chance_map.insert(WeaponNames::ProjectileRapidFireTurret, 4);
        weapon_spawn_relative_chance_map.insert(WeaponNames::ProjectileBurstFire, 5);
        weapon_spawn_relative_chance_map.insert(WeaponNames::Flamethrower, 4);
        weapon_spawn_relative_chance_map.insert(WeaponNames::Missile, 3);
        weapon_spawn_relative_chance_map.insert(WeaponNames::Rockets, 3);
        weapon_spawn_relative_chance_map.insert(WeaponNames::SuperRocketGrenades, 2);
        weapon_spawn_relative_chance_map.insert(WeaponNames::Mine, 2);
        weapon_spawn_relative_chance_map.insert(WeaponNames::Trap, 2);
        weapon_spawn_relative_chance_map.insert(WeaponNames::LaserSword, 3);
        weapon_spawn_relative_chance_map.insert(WeaponNames::BackwardsLaserSword, 1);

        let mut chance_total: u32 = 0;

        for (_key, value) in weapon_spawn_relative_chance_map.iter() {
            chance_total += value;
        }

        let mut chance_aggregate: f32 = 0.0;
        let mut weapon_spawn_chances: Vec<(WeaponNames, f32)> = Vec::new();

        for (key, value) in weapon_spawn_relative_chance_map.iter() {
            if *value > 0 {
                weapon_spawn_chances.push((key.clone(), chance_aggregate));

                chance_aggregate += (*value as f32) / (chance_total as f32);
            }
        }

        log::info!("{:?}", weapon_spawn_chances);


        let game_mode_needs_init: bool;
        {
            let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

            if let Some(_game_mode_setup) = fetched_game_mode_setup {
                game_mode_needs_init = false;
            }
            else {
                game_mode_needs_init = true;
            }
        }

        if game_mode_needs_init {
            //Start off with default classic gun game mode
            world.insert(GameModeSetup {
                game_mode: GameModes::ClassicGunGame,
                match_time_limit: -1.0,
                points_to_win: 14,
                stock_lives: -1,
                checkpoint_count: 0,
                starter_weapon: WeaponNames::LaserDoubleGimballed,
                random_weapon_spawns: false,
                keep_picked_up_weapons: false,
                weapon_spawn_count: 2,
                weapon_spawn_timer: 20.0,
                weapon_spawn_chances: weapon_spawn_chances,
                max_players: INIT_PLAYER_COUNT,
                bot_players: INIT_BOT_COUNT,
                last_hit_threshold: 5.0,
            });
        }

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/menu.ron", ())));
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = state_data;

        //Initialize buttons
        if self.button_classic_gun_game.is_none()
            || self.button_deathmatch_kills.is_none()
            || self.button_deathmatch_stock.is_none()
            || self.button_deathmatch_time.is_none()
            || self.button_king_of_the_hill.is_none()
            || self.button_combat_race.is_none()
            || self.button_start_game.is_none()
        {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.button_classic_gun_game = ui_finder.find(BUTTON_CLASSIC_GUN_GAME);
                self.button_deathmatch_kills = ui_finder.find(BUTTON_DEATHMATCH_KILLS);
                self.button_deathmatch_stock = ui_finder.find(BUTTON_DEATHMATCH_STOCK);
                self.button_deathmatch_time = ui_finder.find(BUTTON_DEATHMATCH_TIME);
                self.button_king_of_the_hill = ui_finder.find(BUTTON_KING_OF_THE_HILL);
                self.button_combat_race = ui_finder.find(BUTTON_COMBAT_RACE);
                self.button_start_game = ui_finder.find(BUTTON_START_GAME);
            });
        }

        
        let mut player_count_init: bool = false;
        let mut bot_count_init: bool = false;

        //Find and Initialize input text value to match game mode
        if self.edit_text_player_count.is_none() {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.edit_text_player_count = ui_finder.find(EDIT_TEXT_PLAYER_COUNT);
                player_count_init = true;
            });
        }

        if self.edit_text_bot_count.is_none() {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.edit_text_bot_count = ui_finder.find(EDIT_TEXT_BOT_COUNT);
                bot_count_init = true;
            });
        }

        let mut ui_text = world.write_storage::<UiText>();
        let fetched_game_mode_setup = world.try_fetch_mut::<GameModeSetup>();

        if let Some(mut game_mode_setup) = fetched_game_mode_setup {
            //Set game mode to match user input after intialization has been completed
            if let Some(player_count) = self.edit_text_player_count.and_then(|entity| ui_text.get_mut(entity)) {
                if player_count_init {
                    player_count.text = game_mode_setup.max_players.to_string();
                }
                else if let Ok(value) = player_count.text.parse::<usize>() {
                    if value > MAX_PLAYER_COUNT {
                        game_mode_setup.max_players = MAX_PLAYER_COUNT;
                        player_count.text = game_mode_setup.max_players.to_string();
                    }
                    else if value < MIN_PLAYER_COUNT {
                        game_mode_setup.max_players = MIN_PLAYER_COUNT;
                        player_count.text = game_mode_setup.max_players.to_string();
                    }
                    else {
                        game_mode_setup.max_players = value;
                    }
                }
                else {
                    game_mode_setup.max_players = MIN_PLAYER_COUNT;
                }
            }

            if let Some(bot_count) = self.edit_text_bot_count.and_then(|entity| ui_text.get_mut(entity)) {
                if bot_count_init {
                    bot_count.text = game_mode_setup.bot_players.to_string();
                }
                else if let Ok(value) = bot_count.text.parse::<usize>() {
                    if value > MAX_PLAYER_COUNT {
                        game_mode_setup.bot_players = MAX_PLAYER_COUNT;
                        bot_count.text = game_mode_setup.bot_players.to_string();
                    }
                    else if value < MIN_BOT_COUNT {
                        game_mode_setup.bot_players = MIN_BOT_COUNT;
                        bot_count.text = game_mode_setup.bot_players.to_string();
                    }
                    else {
                        game_mode_setup.bot_players = value;
                    }
                }
                else {
                    game_mode_setup.bot_players = MIN_BOT_COUNT;
                }
            }
        }

        Trans::None
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let world = data.world;

        match event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Switch] Switching back to WelcomeScreen!");
                    Trans::Switch(Box::new(WelcomeScreen::default()))
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(UiEvent {
                event_type: UiEventType::Click,
                target,
            }) => {
                let fetched_game_mode_setup = world.try_fetch_mut::<GameModeSetup>();

                if let Some(mut game_mode_setup) = fetched_game_mode_setup {
                    if Some(target) == self.button_classic_gun_game {
                        game_mode_setup.game_mode = GameModes::ClassicGunGame;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 14;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = false;
                        game_mode_setup.keep_picked_up_weapons = false;
                    } else if Some(target) == self.button_deathmatch_kills {
                        game_mode_setup.game_mode = GameModes::DeathmatchKills;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 10;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = false;
                    } else if Some(target) == self.button_deathmatch_stock {
                        game_mode_setup.game_mode = GameModes::DeathmatchStock;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = -1;
                        game_mode_setup.stock_lives = 5;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = false;
                    } else if Some(target) == self.button_deathmatch_time {
                        game_mode_setup.game_mode = GameModes::DeathmatchTimedKD;
                        game_mode_setup.match_time_limit = 5.0 * 60.0; //in seconds, 5mins
                        game_mode_setup.points_to_win = -1;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = false;
                    } else if Some(target) == self.button_king_of_the_hill {
                        game_mode_setup.game_mode = GameModes::KingOfTheHill;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 100;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = false;
                    } else if Some(target) == self.button_combat_race {
                        game_mode_setup.game_mode = GameModes::Race;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 10;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 2;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = true;
                    } else if Some(target) == self.button_start_game {
                        log::info!("[Trans::Switch] Switching to GameplayState!");

                        return Trans::Switch(Box::new(GameplayState::default()));
                    }
                }

                Trans::None
            }
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        // after destroying the current UI, invalidate references as well (makes things cleaner)
        if let Some(root_entity) = self.ui_root {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove MainMenu");
        }

        self.ui_root = None;

        self.button_classic_gun_game = None;
        self.button_deathmatch_kills = None;
        self.button_deathmatch_stock = None;
        self.button_deathmatch_time = None;
        self.button_king_of_the_hill = None;
        self.button_combat_race = None;
        self.button_start_game = None;
        self.edit_text_player_count = None;
        self.edit_text_bot_count = None;
    }
}
