use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    winit::{MouseButton, VirtualKeyCode},
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    assets::{AssetStorage, Handle, Loader},
    ui::{UiText, UiTransform, UiCreator},
};


use crate::audio::initialize_audio;

use crate::components::{
    Armor, Health, Hitbox, Player, Repair, Shield, Vehicle, 
    Weapon, WeaponFire, WeaponNames,
    PlayerWeaponIcon,
};


use crate::entities::{initialize_arena_walls, initialize_camera, initialize_ui, intialize_player};

use crate::resources::{initialize_weapon_fire_resource, WeaponFireResource, GameModeSetup, GameModes};



#[derive(Default, Debug)]
pub struct WelcomeScreen {
    ui_handle: Option<Entity>,
}

impl SimpleState for WelcomeScreen {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        self.ui_handle =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/welcome.ron", ())));
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_mouse_button_down(&event, MouseButton::Left) {
                    log::info!("[Trans::Switch] Switching to MainMenu!");
                    Trans::Switch(Box::new(crate::menu::MainMenu::default()))
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(root_entity) = self.ui_handle {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove WelcomeScreen");
        }

        self.ui_handle = None;
    }
}