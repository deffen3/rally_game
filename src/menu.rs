use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder, UiText},
    winit::VirtualKeyCode,
};

use std::collections::HashMap;

use crate::custom_arena::CustomArenaMenu;
use crate::custom_vehicles::CustomVehiclesMenu;
use crate::custom_weapons::CustomWeaponsMenu;
use crate::rally::GameplayState;
use crate::welcome::WelcomeScreen;

use crate::components::{
    build_arena_store, build_vehicle_store, build_weapon_store, get_none_vehicle, ArenaNames,
    ArenaStoreResource, VehicleNames, VehicleStats, WeaponNames, WeaponStoreResource,
};

use crate::resources::{
    GameEndCondition, GameModeSetup, GameModes, GameScore, GameTeamSetup, GameVehicleSetup,
    GameWeaponSelectionMode, GameWeaponSetup, TeamSetupTypes,
};

pub const MAX_PLAYER_COUNT: usize = 4;
pub const MIN_PLAYER_COUNT: usize = 1;
pub const MIN_BOT_COUNT: usize = 0;

pub const INIT_PLAYER_COUNT: usize = 4;
pub const INIT_BOT_COUNT: usize = INIT_PLAYER_COUNT - 1;

const BUTTON_CLASSIC_GUN_GAME: &str = "classic_gun_game";
const BUTTON_DEATHMATCH_KILLS: &str = "deathmatch_kills";
const BUTTON_DEATHMATCH_STOCK: &str = "deathmatch_stock";
const BUTTON_DEATHMATCH_TIME: &str = "deathmatch_time";
const BUTTON_KING_OF_THE_HILL: &str = "king_of_the_hill";
const BUTTON_COMBAT_RACE: &str = "combat_race";
const BUTTON_CAPTURE_THE_FLAG: &str = "capture_the_flag";
const BUTTON_SURVIVAL_WAVES: &str = "survival_waves";
const BUTTON_START_GAME: &str = "start_game";

const EDIT_TEXT_PLAYER_COUNT: &str = "player_count_field";
const EDIT_TEXT_BOT_COUNT: &str = "bot_count_field";

const TEXT_RULES: &str = "rules_text";

const EDIT_TEXT_POINTS_TO_WIN: &str = "points_to_win_field";
const TEXT_POINTS_TO_WIN_LABEL: &str = "points_to_win_text";
const EDIT_TEXT_STOCK_LIVES: &str = "stock_lives_field";
const EDIT_TEXT_TIME_LIMIT: &str = "time_limit_field";

const TEXT_WEAPON_SELECT_MODE: &str = "weapon_select_mode";
const BUTTON_NEXT_WEAPON_SELECT_MODE: &str = "next_weapon_select_mode";
const BUTTON_PREV_WEAPON_SELECT_MODE: &str = "prev_weapon_select_mode";

const BUTTON_CUSTOM_VEHICLES: &str = "customize_vehicles";
const BUTTON_CUSTOM_WEAPONS: &str = "customize_weapons";
const BUTTON_CUSTOM_ARENA: &str = "customize_arena";

const TEXT_PLAYER_TEAMS: &str = "player_teams_result_text";
const BUTTON_FFA: &str = "FFA_button";
const BUTTON_2V2: &str = "2v2_button";
const BUTTON_1V3: &str = "1v3_button";

const BUTTON_SET_CONTROLS_KEYBOARD: &str = "controls_keyboard";
const TEXT_CONTROLS_KEYBOARD: &str = "controls_keyboard_result";

#[derive(Default, Debug)]
pub struct MainMenu {
    ui_root: Option<Entity>,
    button_classic_gun_game: Option<Entity>,
    button_deathmatch_kills: Option<Entity>,
    button_deathmatch_stock: Option<Entity>,
    button_deathmatch_time: Option<Entity>,
    button_king_of_the_hill: Option<Entity>,
    button_capture_the_flag: Option<Entity>,
    button_combat_race: Option<Entity>,
    button_survival_waves: Option<Entity>,
    button_start_game: Option<Entity>,
    edit_text_player_count: Option<Entity>,
    edit_text_bot_count: Option<Entity>,
    text_rules: Option<Entity>,
    edit_text_points_to_win: Option<Entity>,
    text_points_to_win_label: Option<Entity>,
    edit_text_stock_lives: Option<Entity>,
    edit_text_time_limit: Option<Entity>,
    init_base_rules: bool,
    text_weapon_select_mode: Option<Entity>,
    button_next_weapon_select_mode: Option<Entity>,
    button_prev_weapon_select_mode: Option<Entity>,
    button_custom_vehicles: Option<Entity>,
    button_custom_weapons: Option<Entity>,
    button_custom_arena: Option<Entity>,
    text_player_teams: Option<Entity>,
    button_ffa: Option<Entity>,
    button_2v2: Option<Entity>,
    button_1v3: Option<Entity>,
    button_set_controls_keyboard: Option<Entity>,
    controls_keyboard_result: Option<Entity>,
}

impl SimpleState for MainMenu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        build_arena_store(world);

        build_weapon_store(world);

        let mut weapon_spawn_chances: Vec<(WeaponNames, f32)> = Vec::new();
        {
            let weapon_store_resource = world.fetch::<WeaponStoreResource>();
            let weapon_spawn_relative_chance_map = &weapon_store_resource.spawn_chance;

            let mut chance_total: u32 = 0;

            for (_key, value) in weapon_spawn_relative_chance_map.iter() {
                chance_total += value;
            }

            let mut chance_aggregate: f32 = 0.0;

            for (key, value) in weapon_spawn_relative_chance_map.iter() {
                if *value > 0 {
                    weapon_spawn_chances.push((key.clone(), chance_aggregate));

                    chance_aggregate += (*value as f32) / (chance_total as f32);
                }
            }

            log::debug!("{:?}", weapon_spawn_chances);
        }

        let game_mode_needs_init: bool;
        {
            let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();

            if let Some(_game_mode_setup) = fetched_game_mode_setup {
                game_mode_needs_init = false;
            } else {
                game_mode_needs_init = true;
            }
        }

        if game_mode_needs_init {
            //Re-architect this, as this is duplicated code

            //Start off with default classic gun game mode
            world.insert(GameModeSetup {
                game_mode: GameModes::ClassicGunGame,
                match_time_limit: -1.0,
                points_to_win: 14,
                stock_lives: -1,
                checkpoint_count: 0,
                game_end_condition: GameEndCondition::First,
                max_players: INIT_PLAYER_COUNT,
                bot_players: INIT_BOT_COUNT,
                last_hit_threshold: 5.0,
                arena_name: ArenaNames::StandardCombat,
                p1_keyboard: true,
            });

            //these are only defaults if a game-mode is not selected
            //  and classic-gun-game mode is launched straight from the start game button
            world.insert(GameWeaponSetup {
                mode: GameWeaponSelectionMode::GunGameForward,
                starter_weapon: WeaponNames::LaserDoubleGimballed,
                allowable_starter_weapons: vec![WeaponNames::LaserDoubleGimballed],
                random_weapon_spawns: false,
                random_weapon_spawn_count: 2,
                random_weapon_spawn_timer: 20.0,
                random_weapon_spawn_first_timer: 20.0,
                random_weapon_spawn_chances: weapon_spawn_chances,
                allow_map_specific_spawn_weapons: true,
                keep_picked_up_weapons: false,
                new_ammo_on_respawn: true,
            });

            world.insert(GameTeamSetup {
                mode: TeamSetupTypes::FreeForAll,
                teams: [0, 1, 2, 3],
            });

            world.insert(GameScore {
                game_ended: false,
                placements: Vec::new(),
            });

            let vehicle_store = build_vehicle_store(world);

            let vehicle_configs_map: &HashMap<VehicleNames, VehicleStats> =
                &vehicle_store.properties;

            let standard_vehicle_stats = match vehicle_configs_map.get(&VehicleNames::MediumCombat)
            {
                Some(vehicle_config) => vehicle_config.clone(),
                _ => get_none_vehicle(),
            };

            world.insert(GameVehicleSetup {
                names: [
                    VehicleNames::MediumCombat,
                    VehicleNames::MediumCombat,
                    VehicleNames::MediumCombat,
                    VehicleNames::MediumCombat,
                ],
                stats: [
                    standard_vehicle_stats.clone(),
                    standard_vehicle_stats.clone(),
                    standard_vehicle_stats.clone(),
                    standard_vehicle_stats.clone(),
                ],
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
            || self.button_capture_the_flag.is_none()
            || self.button_combat_race.is_none()
            || self.button_survival_waves.is_none()
            || self.button_start_game.is_none()
            || self.text_weapon_select_mode.is_none()
            || self.button_next_weapon_select_mode.is_none()
            || self.button_prev_weapon_select_mode.is_none()
            || self.button_custom_vehicles.is_none()
            || self.button_custom_weapons.is_none()
            || self.button_custom_arena.is_none()
            || self.button_ffa.is_none()
            || self.button_2v2.is_none()
            || self.button_1v3.is_none()
            || self.button_set_controls_keyboard.is_none()
            || self.controls_keyboard_result.is_none()
            || self.text_player_teams.is_none()
        {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.button_classic_gun_game = ui_finder.find(BUTTON_CLASSIC_GUN_GAME);
                self.button_deathmatch_kills = ui_finder.find(BUTTON_DEATHMATCH_KILLS);
                self.button_deathmatch_stock = ui_finder.find(BUTTON_DEATHMATCH_STOCK);
                self.button_deathmatch_time = ui_finder.find(BUTTON_DEATHMATCH_TIME);
                self.button_king_of_the_hill = ui_finder.find(BUTTON_KING_OF_THE_HILL);
                self.button_capture_the_flag = ui_finder.find(BUTTON_CAPTURE_THE_FLAG);
                self.button_combat_race = ui_finder.find(BUTTON_COMBAT_RACE);
                self.button_survival_waves = ui_finder.find(BUTTON_SURVIVAL_WAVES);
                self.button_start_game = ui_finder.find(BUTTON_START_GAME);
                self.text_weapon_select_mode = ui_finder.find(TEXT_WEAPON_SELECT_MODE);
                self.button_next_weapon_select_mode =
                    ui_finder.find(BUTTON_NEXT_WEAPON_SELECT_MODE);
                self.button_prev_weapon_select_mode =
                    ui_finder.find(BUTTON_PREV_WEAPON_SELECT_MODE);
                self.button_custom_vehicles = ui_finder.find(BUTTON_CUSTOM_VEHICLES);
                self.button_custom_weapons = ui_finder.find(BUTTON_CUSTOM_WEAPONS);
                self.button_custom_arena = ui_finder.find(BUTTON_CUSTOM_ARENA);
                self.button_ffa = ui_finder.find(BUTTON_FFA);
                self.button_2v2 = ui_finder.find(BUTTON_2V2);
                self.button_1v3 = ui_finder.find(BUTTON_1V3);
                self.button_set_controls_keyboard = ui_finder.find(BUTTON_SET_CONTROLS_KEYBOARD);
                self.controls_keyboard_result = ui_finder.find(TEXT_CONTROLS_KEYBOARD);
                self.text_player_teams = ui_finder.find(TEXT_PLAYER_TEAMS);
            });
        }

        if self.edit_text_points_to_win.is_none()
            || self.text_points_to_win_label.is_none()
            || self.edit_text_stock_lives.is_none()
            || self.edit_text_time_limit.is_none()
        {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.text_points_to_win_label = ui_finder.find(TEXT_POINTS_TO_WIN_LABEL);
                self.edit_text_points_to_win = ui_finder.find(EDIT_TEXT_POINTS_TO_WIN);
                self.edit_text_stock_lives = ui_finder.find(EDIT_TEXT_STOCK_LIVES);
                self.edit_text_time_limit = ui_finder.find(EDIT_TEXT_TIME_LIMIT);
            });

            self.init_base_rules = true;
        }

        let mut player_count_init: bool = false;
        let mut bot_count_init: bool = false;
        let mut game_mode_rules_init: bool = false;

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

        if self.text_rules.is_none() {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.text_rules = ui_finder.find(TEXT_RULES);
                game_mode_rules_init = true;
            });
        }

        let mut ui_text = world.write_storage::<UiText>();

        let fetched_game_weapon_setup = world.try_fetch_mut::<GameWeaponSetup>();

        if let Some(game_weapon_setup) = fetched_game_weapon_setup {
            if let Some(weapon_select_mode) = self
                .text_weapon_select_mode
                .and_then(|entity| ui_text.get_mut(entity))
            {
                weapon_select_mode.text =
                    get_weapon_select_mode_text(game_weapon_setup.mode.clone());
            }
        }

        let fetched_game_mode_setup = world.try_fetch_mut::<GameModeSetup>();

        if let Some(mut game_mode_setup) = fetched_game_mode_setup {
            if let Some(p1_keyboard_text) = self
                .controls_keyboard_result
                .and_then(|entity| ui_text.get_mut(entity))
            {
                if game_mode_setup.p1_keyboard {
                    p1_keyboard_text.text = "KEYBOARD".to_string();
                } else {
                    p1_keyboard_text.text = "CONTROLLER".to_string();
                }
            }

            //Set game mode to match user input after intialization has been completed
            if let Some(player_count) = self
                .edit_text_player_count
                .and_then(|entity| ui_text.get_mut(entity))
            {
                if player_count_init {
                    player_count.text = game_mode_setup.max_players.to_string();
                } else if let Ok(value) = player_count.text.parse::<usize>() {
                    if value > MAX_PLAYER_COUNT {
                        game_mode_setup.max_players = MAX_PLAYER_COUNT;
                        player_count.text = game_mode_setup.max_players.to_string();
                    } else if value < MIN_PLAYER_COUNT {
                        game_mode_setup.max_players = MIN_PLAYER_COUNT;
                        player_count.text = game_mode_setup.max_players.to_string();
                    } else {
                        game_mode_setup.max_players = value;
                    }
                } else {
                    game_mode_setup.max_players = MIN_PLAYER_COUNT;
                }
            }

            if let Some(bot_count) = self
                .edit_text_bot_count
                .and_then(|entity| ui_text.get_mut(entity))
            {
                if bot_count_init {
                    bot_count.text = game_mode_setup.bot_players.to_string();
                } else if let Ok(value) = bot_count.text.parse::<usize>() {
                    if value > game_mode_setup.max_players {
                        game_mode_setup.bot_players = game_mode_setup.max_players;
                        bot_count.text = game_mode_setup.bot_players.to_string();
                    } else if value < MIN_BOT_COUNT {
                        game_mode_setup.bot_players = MIN_BOT_COUNT;
                        bot_count.text = game_mode_setup.bot_players.to_string();
                    } else {
                        game_mode_setup.bot_players = value;
                    }
                } else {
                    game_mode_setup.bot_players = MIN_BOT_COUNT;
                }
            }

            if let Some(game_rules) = self.text_rules.and_then(|entity| ui_text.get_mut(entity)) {
                game_rules.text = get_game_rules_text(game_mode_setup.game_mode.clone());
            }

            if let Some(points_to_win_label) = self
                .text_points_to_win_label
                .and_then(|entity| ui_text.get_mut(entity))
            {
                points_to_win_label.text = get_points_label_text(game_mode_setup.game_mode.clone());
            }

            if let Some(points_to_win) = self
                .edit_text_points_to_win
                .and_then(|entity| ui_text.get_mut(entity))
            {
                if self.init_base_rules {
                    //Initialization of base rules
                    let setup_points_to_win = game_mode_setup.points_to_win.clone();
                    if setup_points_to_win <= 0 {
                        points_to_win.text = "".to_string();
                    } else {
                        points_to_win.text = setup_points_to_win.to_string();
                    }
                } else {
                    //Accepting User Input to modify base rules
                    if let Ok(value) = points_to_win.text.parse::<i32>() {
                        if value < 1 {
                            points_to_win.text = "".to_string();
                            game_mode_setup.points_to_win = -1;
                        } else {
                            game_mode_setup.points_to_win = value;
                        }
                    }
                }
            }

            if let Some(stock_lives) = self
                .edit_text_stock_lives
                .and_then(|entity| ui_text.get_mut(entity))
            {
                if self.init_base_rules {
                    //Initialization of base rules
                    let setup_stock_lives = game_mode_setup.stock_lives.clone();
                    if setup_stock_lives <= 0 {
                        stock_lives.text = "".to_string();
                    } else {
                        stock_lives.text = setup_stock_lives.to_string();
                    }
                } else {
                    //Accepting User Input to modify base rules
                    if let Ok(value) = stock_lives.text.parse::<i32>() {
                        if value < 1 {
                            stock_lives.text = "".to_string();
                            game_mode_setup.stock_lives = -1;
                        } else {
                            game_mode_setup.stock_lives = value;
                        }
                    }
                }
            }

            if let Some(time_limit) = self
                .edit_text_time_limit
                .and_then(|entity| ui_text.get_mut(entity))
            {
                if self.init_base_rules {
                    //Initialization of base rules
                    let setup_match_time_limit = game_mode_setup.match_time_limit.clone();
                    if setup_match_time_limit <= 0.0 {
                        time_limit.text = "".to_string();
                    } else {
                        time_limit.text = (setup_match_time_limit / 60.).floor().to_string();
                    }
                } else {
                    //Accepting User Input to modify base rules
                    if let Ok(value) = time_limit.text.parse::<f32>() {
                        if value <= 0.0 {
                            time_limit.text = "".to_string();
                            game_mode_setup.match_time_limit = -1.0;
                        } else {
                            game_mode_setup.match_time_limit = value * 60.;
                        }
                    }
                }
            }
        }

        let fetched_game_team_setup = world.try_fetch::<GameTeamSetup>();

        if let Some(game_team_setup) = fetched_game_team_setup {
            if let Some(team_setup) = self
                .text_player_teams
                .and_then(|entity| ui_text.get_mut(entity))
            {
                let team_setup_text = match game_team_setup.teams {
                    [0, 1, 2, 3] => "FFA".to_string(),
                    [0, 0, 1, 1] => "P1 P2 v P3 P4".to_string(),
                    [0, 1, 0, 1] => "P1 P3 v P2 P4".to_string(),
                    [0, 1, 1, 0] => "P1 P4 v P2 P3".to_string(),
                    [0, 1, 1, 1] => "P1 v P2 P3 P4".to_string(),
                    [1, 0, 1, 1] => "P2 v P1 P3 P4".to_string(),
                    [1, 1, 0, 1] => "P3 v P1 P2 P4".to_string(),
                    [1, 1, 1, 0] => "P4 v P1 P2 P3".to_string(),
                    _ => "???".to_string(),
                };

                team_setup.text = team_setup_text;
            }
        }

        if self.init_base_rules {
            self.init_base_rules = false;
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
                let fetched_game_weapon_setup = world.try_fetch_mut::<GameWeaponSetup>();
                let fetched_game_team_setup = world.try_fetch_mut::<GameTeamSetup>();

                let fetched_arena_store = world.try_fetch::<ArenaStoreResource>();

                if let Some(mut game_mode_setup) = fetched_game_mode_setup {
                    if Some(target) == self.button_set_controls_keyboard {
                        game_mode_setup.p1_keyboard = !game_mode_setup.p1_keyboard;
                    } else if Some(target) == self.button_classic_gun_game {
                        game_mode_setup.game_mode = GameModes::ClassicGunGame;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 14;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.game_end_condition = GameEndCondition::First;
                        self.init_base_rules = true;
                    } else if Some(target) == self.button_deathmatch_kills {
                        game_mode_setup.game_mode = GameModes::DeathmatchKills;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 10;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.game_end_condition = GameEndCondition::First;
                        self.init_base_rules = true;
                    } else if Some(target) == self.button_deathmatch_stock {
                        game_mode_setup.game_mode = GameModes::DeathmatchStock;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = -1;
                        game_mode_setup.stock_lives = 5;
                        game_mode_setup.game_end_condition = GameEndCondition::AllButOne;
                        self.init_base_rules = true;
                    } else if Some(target) == self.button_deathmatch_time {
                        game_mode_setup.game_mode = GameModes::DeathmatchTimedKD;
                        game_mode_setup.match_time_limit = 5.0 * 60.0; //in seconds, 5mins
                        game_mode_setup.points_to_win = -1;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.game_end_condition = GameEndCondition::AllButOne; //but usually just ends by time
                        self.init_base_rules = true;
                    } else if Some(target) == self.button_king_of_the_hill {
                        game_mode_setup.game_mode = GameModes::KingOfTheHill;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 50;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.game_end_condition = GameEndCondition::First;
                        self.init_base_rules = true;
                    } else if Some(target) == self.button_combat_race {
                        game_mode_setup.game_mode = GameModes::Race;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 10;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.game_end_condition = GameEndCondition::AllButOneExtended; //extended for a few seconds after
                        self.init_base_rules = true;
                    } else if Some(target) == self.button_capture_the_flag {
                        game_mode_setup.game_mode = GameModes::CaptureTheFlag;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 10;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.game_end_condition = GameEndCondition::First; //extended for a few seconds after
                        self.init_base_rules = true;
                    } else if Some(target) == self.button_survival_waves {
                        game_mode_setup.game_mode = GameModes::SurvivalWaves;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 10;
                        game_mode_setup.stock_lives = 3;
                        game_mode_setup.game_end_condition = GameEndCondition::AllButOneExtended; //extended for a few seconds after
                        self.init_base_rules = true;
                    }

                    //Select default arena map
                    if let Some(arena_store) = fetched_arena_store {
                        let game_mode_arena =
                            match arena_store.game_modes.get(&game_mode_setup.game_mode) {
                                Some(game_mode_arenas) => game_mode_arenas[0],
                                _ => ArenaNames::OpenEmptyMap,
                            };
                        game_mode_setup.arena_name = game_mode_arena;

                        let checkpoint_count = match arena_store.properties.get(&game_mode_arena) {
                            Some(arena_props) => arena_props.race_checkpoints.len(),
                            _ => 0,
                        };
                        game_mode_setup.checkpoint_count = checkpoint_count as i32;
                    } else {
                        game_mode_setup.arena_name = ArenaNames::OpenEmptyMap;
                        game_mode_setup.checkpoint_count = 0;
                    }

                    if let Some(mut game_weapon_setup) = fetched_game_weapon_setup {
                        if Some(target) == self.button_next_weapon_select_mode {
                            if game_mode_setup.game_mode == GameModes::ClassicGunGame {
                                game_weapon_setup.mode = match game_weapon_setup.mode {
                                    GameWeaponSelectionMode::GunGameForward => {
                                        GameWeaponSelectionMode::GunGameReverse
                                    }
                                    GameWeaponSelectionMode::GunGameReverse => {
                                        GameWeaponSelectionMode::GunGameRandom
                                    }
                                    GameWeaponSelectionMode::GunGameRandom => {
                                        GameWeaponSelectionMode::GunGameForward
                                    }
                                    _ => GameWeaponSelectionMode::GunGameForward,
                                }
                            } else {
                                game_weapon_setup.mode = match game_weapon_setup.mode {
                                    GameWeaponSelectionMode::StarterAndPickup => {
                                        GameWeaponSelectionMode::CustomStarterAndPickup
                                    }
                                    GameWeaponSelectionMode::CustomStarterAndPickup => {
                                        GameWeaponSelectionMode::FullCustom
                                    }
                                    GameWeaponSelectionMode::FullCustom => {
                                        GameWeaponSelectionMode::VehiclePreset
                                    }
                                    GameWeaponSelectionMode::VehiclePreset => {
                                        GameWeaponSelectionMode::StarterAndPickup
                                    }
                                    _ => GameWeaponSelectionMode::StarterAndPickup,
                                }
                            }
                        } else if Some(target) == self.button_prev_weapon_select_mode {
                            if game_mode_setup.game_mode == GameModes::ClassicGunGame {
                                game_weapon_setup.mode = match game_weapon_setup.mode {
                                    GameWeaponSelectionMode::GunGameForward => {
                                        GameWeaponSelectionMode::GunGameRandom
                                    }
                                    GameWeaponSelectionMode::GunGameReverse => {
                                        GameWeaponSelectionMode::GunGameForward
                                    }
                                    GameWeaponSelectionMode::GunGameRandom => {
                                        GameWeaponSelectionMode::GunGameReverse
                                    }
                                    _ => GameWeaponSelectionMode::GunGameForward,
                                }
                            } else {
                                game_weapon_setup.mode = match game_weapon_setup.mode {
                                    GameWeaponSelectionMode::StarterAndPickup => {
                                        GameWeaponSelectionMode::VehiclePreset
                                    }
                                    GameWeaponSelectionMode::CustomStarterAndPickup => {
                                        GameWeaponSelectionMode::StarterAndPickup
                                    }
                                    GameWeaponSelectionMode::FullCustom => {
                                        GameWeaponSelectionMode::CustomStarterAndPickup
                                    }
                                    GameWeaponSelectionMode::VehiclePreset => {
                                        GameWeaponSelectionMode::FullCustom
                                    }
                                    _ => GameWeaponSelectionMode::StarterAndPickup,
                                }
                            }
                        } else if Some(target) == self.button_classic_gun_game {
                            game_weapon_setup.mode = GameWeaponSelectionMode::GunGameForward;
                            game_weapon_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                            game_weapon_setup.random_weapon_spawns = false;
                            game_weapon_setup.keep_picked_up_weapons = false;
                        } else if Some(target) == self.button_deathmatch_kills {
                            game_weapon_setup.mode = GameWeaponSelectionMode::StarterAndPickup;
                            game_weapon_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                            game_weapon_setup.random_weapon_spawns = true;
                            game_weapon_setup.keep_picked_up_weapons = false;
                        } else if Some(target) == self.button_deathmatch_stock {
                            game_weapon_setup.mode = GameWeaponSelectionMode::StarterAndPickup;
                            game_weapon_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                            game_weapon_setup.random_weapon_spawns = true;
                            game_weapon_setup.keep_picked_up_weapons = false;
                        } else if Some(target) == self.button_deathmatch_time {
                            game_weapon_setup.mode = GameWeaponSelectionMode::StarterAndPickup;
                            game_weapon_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                            game_weapon_setup.random_weapon_spawns = true;
                            game_weapon_setup.keep_picked_up_weapons = false;
                        } else if Some(target) == self.button_king_of_the_hill {
                            game_weapon_setup.mode = GameWeaponSelectionMode::StarterAndPickup;
                            game_weapon_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                            game_weapon_setup.random_weapon_spawns = true;
                            game_weapon_setup.keep_picked_up_weapons = false;
                        } else if Some(target) == self.button_combat_race {
                            game_weapon_setup.mode = GameWeaponSelectionMode::StarterAndPickup;
                            game_weapon_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                            game_weapon_setup.random_weapon_spawns = true;
                            game_weapon_setup.keep_picked_up_weapons = true;
                        } else if Some(target) == self.button_capture_the_flag {
                            game_weapon_setup.mode = GameWeaponSelectionMode::StarterAndPickup;
                            game_weapon_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                            game_weapon_setup.random_weapon_spawns = true;
                            game_weapon_setup.keep_picked_up_weapons = false;
                        } else if Some(target) == self.button_survival_waves {
                            game_weapon_setup.mode = GameWeaponSelectionMode::StarterAndPickup;
                            game_weapon_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                            game_weapon_setup.random_weapon_spawns = true;
                            game_weapon_setup.keep_picked_up_weapons = false;
                        }
                    }
                }

                if let Some(mut game_team_setup) = fetched_game_team_setup {
                    if Some(target) == self.button_ffa {
                        game_team_setup.mode = TeamSetupTypes::FreeForAll;
                        game_team_setup.teams = [0, 1, 2, 3];
                    }
                    if Some(target) == self.button_2v2 {
                        game_team_setup.mode = TeamSetupTypes::TwoVsTwo;
                        game_team_setup.teams = match game_team_setup.teams {
                            [0, 0, 1, 1] => [0, 1, 0, 1],
                            [0, 1, 0, 1] => [0, 1, 1, 0],
                            _ => [0, 0, 1, 1],
                        };
                    }
                    if Some(target) == self.button_1v3 {
                        game_team_setup.mode = TeamSetupTypes::OneVsThree;
                        game_team_setup.teams = match game_team_setup.teams {
                            [0, 1, 1, 1] => [1, 0, 1, 1],
                            [1, 0, 1, 1] => [1, 1, 0, 1],
                            [1, 1, 0, 1] => [1, 1, 1, 0],
                            _ => [0, 1, 1, 1],
                        };
                    }
                }

                if Some(target) == self.button_custom_vehicles {
                    return Trans::Switch(Box::new(CustomVehiclesMenu::default()));
                } else if Some(target) == self.button_custom_weapons {
                    return Trans::Switch(Box::new(CustomWeaponsMenu::default()));
                } else if Some(target) == self.button_custom_arena {
                    return Trans::Switch(Box::new(CustomArenaMenu::default()));
                } else if Some(target) == self.button_start_game {
                    log::info!("[Trans::Switch] Switching to GameplayState!");
                    return Trans::Switch(Box::new(GameplayState::default()));
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
        self.button_capture_the_flag = None;
        self.button_combat_race = None;
        self.button_survival_waves = None;
        self.button_start_game = None;
        self.edit_text_player_count = None;
        self.edit_text_bot_count = None;
        self.edit_text_points_to_win = None;
        self.edit_text_stock_lives = None;
        self.edit_text_time_limit = None;
        self.text_weapon_select_mode = None;
        self.button_next_weapon_select_mode = None;
        self.button_prev_weapon_select_mode = None;
        self.button_custom_vehicles = None;
        self.button_custom_weapons = None;
        self.button_custom_arena = None;
        self.button_ffa = None;
        self.button_2v2 = None;
        self.button_1v3 = None;
        self.button_set_controls_keyboard = None;
        self.text_player_teams = None;
    }
}

fn get_game_rules_text(game_mode: GameModes) -> String {
    match game_mode {
        GameModes::ClassicGunGame => "Classic Gun Game:\nFirst to get a kill with each weapon wins. Weapons are hot-swapped after each kill.".to_string(),
        GameModes::DeathmatchKills => "Deathmatch - Kills:\nFirst to a certain number of kills wins. New weapons can be picked up from arena.".to_string(),
        GameModes::DeathmatchStock => "Deathmatch - Stock:\nIf you run out of lives you are out. Last player alive wins. New weapons can be picked up from arena.".to_string(),
        GameModes::DeathmatchTimedKD => "Deathmatch - Timed:\nMatch ends after set time. Highest score of Kills minus Deaths is the winner. Self-destructs are minus 2 deaths. New weapons can be picked up from arena.".to_string(),
        GameModes::KingOfTheHill => "King of the Hill:\nPlayers gains points for being the only person in the special 'hill' zone. First player to a certain number of points wins. New weapons can be picked up from arena.".to_string(),
        GameModes::Race => "Combat Race:\nIt's a race with weapons active. First player to complete the required number of laps wins. New weapons can be picked up from the arena race track.".to_string(),
        GameModes::CaptureTheFlag => "Capture the Flag:\nPlayers gain a point every time they return the flag back to their zone. New weapons can be picked up from arena".to_string(),
        GameModes::SurvivalWaves => "Survival - Waves:\nSee how long you can stay alive (number of waves of enemies). Last player alive wins. New weapons can be picked up from arena".to_string(),
    }
}

fn get_points_label_text(game_mode: GameModes) -> String {
    match game_mode {
        GameModes::ClassicGunGame => "Points to Win:".to_string(),
        GameModes::DeathmatchKills => "Kills to Win:".to_string(),
        GameModes::DeathmatchStock => "Kills to Win:".to_string(), //Objective is typically based on lives, and Kills are typically ignored
        GameModes::DeathmatchTimedKD => "Points to Win:".to_string(),
        GameModes::KingOfTheHill => "Points to Win:".to_string(),
        GameModes::Race => "Laps to Win:".to_string(),
        GameModes::CaptureTheFlag => "Flags to Win:".to_string(),
        GameModes::SurvivalWaves => "Waves to Win:".to_string(),
    }
}

fn get_weapon_select_mode_text(weapon_select_mode: GameWeaponSelectionMode) -> String {
    match weapon_select_mode {
        GameWeaponSelectionMode::GunGameForward => "Gun-Game Forward".to_string(),
        GameWeaponSelectionMode::GunGameReverse => "Gun-Game Reverse".to_string(),
        GameWeaponSelectionMode::GunGameRandom => "Gun-Game Random".to_string(),
        GameWeaponSelectionMode::StarterAndPickup => "Starter + Pickup".to_string(),
        GameWeaponSelectionMode::CustomStarterAndPickup => "Custom Starter + Pickup".to_string(),
        GameWeaponSelectionMode::FullCustom => "Full Custom".to_string(),
        GameWeaponSelectionMode::VehiclePreset => "Vehicle Presets".to_string(),
    }
}
