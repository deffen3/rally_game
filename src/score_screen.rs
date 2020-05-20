use amethyst::{
    core::{Time},
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
    winit::VirtualKeyCode,
    utils::{
        removal::{exec_removal},
    },
};

use crate::menu::MainMenu;

use crate::resources::{GameModeSetup, GameScore};


const SCORE_SCREEN_TIMER_INIT: f32 = 2.0;

const BUTTON_BACK_TO_MENU: &str = "back_to_menu";


#[derive(Default, Debug)]
pub struct ScoreScreen {
    ui_root: Option<Entity>,
    button_back_to_menu: Option<Entity>,
    load_timer: f32,
    loaded: bool,
}

impl SimpleState for ScoreScreen {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        {
            let fetched_game_score = world.try_fetch_mut::<GameScore>();

            if let Some(mut game_score) = fetched_game_score {
                game_score.game_ended = false; //reset
            }
        }

        self.loaded = false;
        self.load_timer = SCORE_SCREEN_TIMER_INIT;
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = state_data;

        let dt;
        {
            let fetched_time = world.try_fetch::<Time>();

            if let Some(time) = fetched_time {
                dt = time.delta_seconds();
            }
            else {
                dt = 0.01;
            }
        }

        {
            if !self.loaded && self.load_timer <= 0.0 {
                self.loaded = true;
                self.ui_root =
                    Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/score_screen.ron", ())));
            }
            else {
                self.load_timer -= dt;
            }
        }

        {
            if self.button_back_to_menu.is_none()
            {
                world.exec(|ui_finder: UiFinder<'_>| {
                    self.button_back_to_menu = ui_finder.find(BUTTON_BACK_TO_MENU);
                });
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
                if Some(target) == self.button_back_to_menu {
                    log::info!("[Trans::Switch] Switching back to MainMenu!");
                    return Trans::Switch(Box::new(MainMenu::default()));
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
                .expect("Failed to remove CustomArenaMenu");
        }

        exec_removal(&data.world.entities(), &data.world.read_storage(), 0 as u32);

        self.ui_root = None;

        self.button_back_to_menu = None;
    }
}