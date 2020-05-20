use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
    winit::VirtualKeyCode,
};

use crate::menu::MainMenu;

use crate::resources::{GameModeSetup};



const BUTTON_BACK_TO_MENU: &str = "back_to_menu";


#[derive(Default, Debug)]
pub struct CustomArenaMenu {
    ui_root: Option<Entity>,
    button_back_to_menu: Option<Entity>,
}

impl SimpleState for CustomArenaMenu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/custom_arena.ron", ())));
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

        self.ui_root = None;

        self.button_back_to_menu = None;
    }
}