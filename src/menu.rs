use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
    winit::VirtualKeyCode,
};

use crate::rally::GameplayState;
use crate::welcome::WelcomeScreen;

use crate::components::WeaponNames;
use crate::resources::{GameModeSetup, GameModes};

const BUTTON_CLASSIC_GUN_GAME: &str = "classic_gun_game";
const BUTTON_DEATHMATCH_KILLS: &str = "deathmatch_kills";
const BUTTON_DEATHMATCH_STOCK: &str = "deathmatch_stock";
const BUTTON_DEATHMATCH_TIME: &str = "deathmatch_time";
const BUTTON_KING_OF_THE_HILL: &str = "king_of_the_hill";
const BUTTON_COMBAT_RACE: &str = "combat_race";

#[derive(Default, Debug)]
pub struct MainMenu {
    ui_root: Option<Entity>,
    button_classic_gun_game: Option<Entity>,
    button_deathmatch_kills: Option<Entity>,
    button_deathmatch_stock: Option<Entity>,
    button_deathmatch_time: Option<Entity>,
    button_king_of_the_hill: Option<Entity>,
    button_combat_race: Option<Entity>,
}

impl SimpleState for MainMenu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        //Start off with default classic gun game mode
        world.insert(GameModeSetup {
            game_mode: GameModes::ClassicGunGame,
            match_time_limit: -1.0,
            points_to_win: 15,
            stock_lives: -1,
            checkpoint_count: 0,
            starter_weapon: WeaponNames::LaserDoubleGimballed,
            random_weapon_spawns: false,
            keep_picked_up_weapons: false,
            weapon_spawn_count: 2,
            weapon_spawn_timer: 20.0,
            max_players: 4,
            bot_players: 3,
        });

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/menu.ron", ())));
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = state_data;

        if self.button_classic_gun_game.is_none()
            || self.button_deathmatch_kills.is_none()
            || self.button_deathmatch_stock.is_none()
            || self.button_deathmatch_time.is_none()
            || self.button_king_of_the_hill.is_none()
            || self.button_combat_race.is_none()
        {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.button_classic_gun_game = ui_finder.find(BUTTON_CLASSIC_GUN_GAME);
                self.button_deathmatch_kills = ui_finder.find(BUTTON_DEATHMATCH_KILLS);
                self.button_deathmatch_stock = ui_finder.find(BUTTON_DEATHMATCH_STOCK);
                self.button_deathmatch_time = ui_finder.find(BUTTON_DEATHMATCH_TIME);
                self.button_king_of_the_hill = ui_finder.find(BUTTON_KING_OF_THE_HILL);
                self.button_combat_race = ui_finder.find(BUTTON_COMBAT_RACE);
            });
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
                /*
                pub struct GameModeSetup {
                    pub game_mode: GameModes,
                    pub match_time_limit: f32,
                    pub points_to_win: i32,
                    pub stock_lives: i32,
                    pub checkpoint_count: i32,
                    pub starter_weapon: WeaponNames,
                    pub random_weapon_spawns: bool,
                    pub max_players: usize,
                    pub bot_players: usize,
                }
                */

                let fetched_game_mode_setup = world.try_fetch_mut::<GameModeSetup>();

                if let Some(mut game_mode_setup) = fetched_game_mode_setup {
                    game_mode_setup.max_players = 4;
                    game_mode_setup.bot_players = 3;

                    if Some(target) == self.button_classic_gun_game {
                        log::info!("[Trans::Switch] Switching to GameplayState!");

                        game_mode_setup.game_mode = GameModes::ClassicGunGame;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 15;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = false;
                        game_mode_setup.keep_picked_up_weapons = false;

                        return Trans::Switch(Box::new(GameplayState::default()));
                    } else if Some(target) == self.button_deathmatch_kills {
                        log::info!("[Trans::Switch] Switching to GameplayState!");

                        game_mode_setup.game_mode = GameModes::DeathmatchKills;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 5;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = false;

                        return Trans::Switch(Box::new(GameplayState::default()));
                    } else if Some(target) == self.button_deathmatch_stock {
                        log::info!("[Trans::Switch] Switching to GameplayState!");

                        game_mode_setup.game_mode = GameModes::DeathmatchStock;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = -1;
                        game_mode_setup.stock_lives = 5;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = false;

                        return Trans::Switch(Box::new(GameplayState::default()));
                    } else if Some(target) == self.button_deathmatch_time {
                        log::info!("[Trans::Switch] Switching to GameplayState!");

                        game_mode_setup.game_mode = GameModes::DeathmatchTimedKD;
                        game_mode_setup.match_time_limit = 5.0 * 60.0; //in seconds, 5mins
                        game_mode_setup.points_to_win = -1;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = false;

                        return Trans::Switch(Box::new(GameplayState::default()));
                    } else if Some(target) == self.button_king_of_the_hill {
                        log::info!("[Trans::Switch] Switching to GameplayState!");

                        game_mode_setup.game_mode = GameModes::KingOfTheHill;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 100;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = false;

                        return Trans::Switch(Box::new(GameplayState::default()));
                    } else if Some(target) == self.button_combat_race {
                        log::info!("[Trans::Switch] Switching to GameplayState!");

                        game_mode_setup.game_mode = GameModes::Race;
                        game_mode_setup.match_time_limit = -1.0;
                        game_mode_setup.points_to_win = 10;
                        game_mode_setup.stock_lives = -1;
                        game_mode_setup.checkpoint_count = 0;
                        game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                        game_mode_setup.random_weapon_spawns = true;
                        game_mode_setup.keep_picked_up_weapons = true;

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
    }
}
