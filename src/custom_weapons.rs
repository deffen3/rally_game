use amethyst::{
    assets::Loader,
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{Anchor, TtfFormat, UiCreator, UiEvent, UiEventType, UiFinder, UiText, UiTransform},
    utils::removal::{exec_removal, Removal},
    winit::VirtualKeyCode,
};

use crate::menu::MainMenu;

use crate::components::{WeaponNames, WeaponStoreResource};

const BUTTON_BACK_TO_MENU: &str = "back_to_menu";

#[derive(Default, Debug)]
pub struct CustomWeaponsMenu {
    ui_root: Option<Entity>,
    button_back_to_menu: Option<Entity>,

    weapon_names_list_display: Vec<(WeaponNames, String)>,
    text_weapon_names: Vec<Entity>,
}

impl SimpleState for CustomWeaponsMenu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        world.register::<Removal<u32>>();
        world.register::<UiText>();

        self.ui_root = Some(
            world.exec(|mut creator: UiCreator<'_>| creator.create("ui/custom_weapons.ron", ())),
        );

        let font = world.read_resource::<Loader>().load(
            "font/square.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        );

        {
            //Build list of weapon display names
            let fetched_weapon_store = world.fetch::<WeaponStoreResource>();

            for weapon_name in fetched_weapon_store.selection_order.iter() {
                let weapon_properties = fetched_weapon_store.properties.get(&weapon_name);

                if let Some(weapon_properties) = weapon_properties {
                    let weapon_display_name = weapon_properties.display_name.clone();

                    self.weapon_names_list_display
                        .push((*weapon_name, weapon_display_name));
                }
            }
        }

        for (idx, weapon_displays) in self.weapon_names_list_display.iter().enumerate() {
            let (_weapon_name, weapon_display_name) = weapon_displays;

            let icon_transform = UiTransform::new(
                weapon_display_name.to_string(),
                Anchor::TopLeft,
                Anchor::TopLeft,
                50.0,
                -140.0 - ((idx as f32) * 25.0),
                0.3,
                350.0,
                18.0,
            );

            let mut ui_text = UiText::new(
                font.clone(),
                weapon_display_name.to_string(),
                [1., 1., 1., 1.],
                18.,
            );

            ui_text.align = Anchor::MiddleLeft;

            let weapon_display_entity = world
                .create_entity()
                .with(icon_transform)
                .with(ui_text)
                .with(Removal::new(0 as u32))
                .build();

            self.text_weapon_names.push(weapon_display_entity);
        }
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = state_data;

        if self.button_back_to_menu.is_none() {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.button_back_to_menu = ui_finder.find(BUTTON_BACK_TO_MENU);
            });
        }

        Trans::None
    }

    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
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
                .expect("Failed to remove CustomWeaponsMenu");
        }

        exec_removal(&data.world.entities(), &data.world.read_storage(), 0 as u32);

        self.ui_root = None;

        self.button_back_to_menu = None;

        self.text_weapon_names = Vec::new();
    }
}
