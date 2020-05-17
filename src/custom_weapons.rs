use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder, UiText},
    winit::VirtualKeyCode,
};

use std::collections::HashMap;

use crate::menu::MainMenu;

use crate::components::WeaponNames;
use crate::resources::{GameModeSetup, GameModes};



const BUTTON_BACK_TO_MENU: &str = "back_to_menu";


#[derive(Default, Debug)]
pub struct CustomWeaponsMenu {
    ui_root: Option<Entity>,
    button_back_to_menu: Option<Entity>,
}

impl SimpleState for CustomWeaponsMenu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/custom_weapons.ron", ())));
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = state_data;

        if self.button_back_to_menu.is_none()
        {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.button_back_to_menu = ui_finder.find(BUTTON_BACK_TO_MENU);
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
                    log::info!("[Trans::Switch] Switching back to MainMenu!");
                    Trans::Switch(Box::new(MainMenu::default()))
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
                    // if Some(target) == self.button {
                    //     game_mode_setup.game_mode = GameModes::Race;
                    //     game_mode_setup.match_time_limit = -1.0;
                    //     game_mode_setup.points_to_win = 10;
                    //     game_mode_setup.stock_lives = -1;
                    //     game_mode_setup.checkpoint_count = 2;
                    //     game_mode_setup.starter_weapon = WeaponNames::LaserDoubleGimballed;
                    //     game_mode_setup.random_weapon_spawns = true;
                    //     game_mode_setup.keep_picked_up_weapons = true;
                    // } 
                    if Some(target) == self.button_back_to_menu {
                        log::info!("[Trans::Switch] Switching back to MainMenu!");
                        return Trans::Switch(Box::new(MainMenu::default()));
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
                .expect("Failed to remove CustomWeaponsMenu");
        }

        self.ui_root = None;

        self.button_back_to_menu = None;
    }
}