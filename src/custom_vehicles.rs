use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder, UiText},
    winit::VirtualKeyCode,
};

use std::collections::HashMap;

use crate::menu::MainMenu;

use crate::resources::{GameModeSetup, GameVehicleSetup,};

use crate::components::{determine_vehicle_weight_stats, get_vehicle_name_string, 
    VehicleMovementType, VehicleStoreResource, get_next_vehicle_name, get_prev_vehicle_name,
    VehicleNames, VehicleStats,
};


const BUTTON_BACK_TO_MENU: &str = "back_to_menu";
const BUTTON_P1_PREV_VEHICLE: &str = "p1_prev_vehicle";
const BUTTON_P1_NEXT_VEHICLE: &str = "p1_next_vehicle";

const TEXT_P1_VEHICLE_NAME: &str = "p1_veh_text";
const TEXT_P1_SHIELDS: &str = "p1_shields";
const TEXT_P1_ARMOR: &str = "p1_armor";
const TEXT_P1_HEALTH: &str = "p1_health";
const TEXT_P1_WEIGHT: &str = "p1_weight";



#[derive(Default, Debug)]
pub struct CustomVehiclesMenu {
    ui_root: Option<Entity>,

    button_back_to_menu: Option<Entity>,

    button_p1_prev_vehicle: Option<Entity>,
    button_p1_next_vehicle: Option<Entity>,

    text_p1_vehicle_name: Option<Entity>,
    text_p1_shields: Option<Entity>,
    text_p1_armor: Option<Entity>,
    text_p1_health: Option<Entity>,
    text_p1_weight: Option<Entity>,

}

impl SimpleState for CustomVehiclesMenu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/custom_vehicles.ron", ())));
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = state_data;

        if self.button_back_to_menu.is_none()
            || self.button_p1_prev_vehicle.is_none()
            || self.button_p1_next_vehicle.is_none()
            || self.text_p1_vehicle_name.is_none()
            || self.text_p1_shields.is_none()
            || self.text_p1_armor.is_none()
            || self.text_p1_health.is_none()
            || self.text_p1_weight.is_none()
        {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.button_back_to_menu = ui_finder.find(BUTTON_BACK_TO_MENU);

                self.button_p1_prev_vehicle = ui_finder.find(BUTTON_P1_PREV_VEHICLE);
                self.button_p1_next_vehicle = ui_finder.find(BUTTON_P1_NEXT_VEHICLE);

                self.text_p1_vehicle_name = ui_finder.find(TEXT_P1_VEHICLE_NAME);
                self.text_p1_shields = ui_finder.find(TEXT_P1_SHIELDS);
                self.text_p1_armor = ui_finder.find(TEXT_P1_ARMOR);
                self.text_p1_health = ui_finder.find(TEXT_P1_HEALTH);
                self.text_p1_weight = ui_finder.find(TEXT_P1_WEIGHT);
            });
        }


        let mut ui_text = world.write_storage::<UiText>();
        let fetched_game_vehicle_setup = world.try_fetch_mut::<GameVehicleSetup>();

        if let Some(mut game_vehicle_setup) = fetched_game_vehicle_setup {

            if let Some(veh_name) = self.text_p1_vehicle_name.and_then(|entity| ui_text.get_mut(entity)) {
                veh_name.text = get_vehicle_name_string(game_vehicle_setup.p1_name.clone());
            }

            let veh_stats = game_vehicle_setup.p1_stats;            

            if let Some(veh_shields) = self.text_p1_shields.and_then(|entity| ui_text.get_mut(entity)) {
                veh_shields.text = veh_stats.max_shield.to_string();
            }

            if let Some(veh_armor) = self.text_p1_armor.and_then(|entity| ui_text.get_mut(entity)) {
                veh_armor.text = veh_stats.max_armor.to_string();
            }

            if let Some(veh_health) = self.text_p1_health.and_then(|entity| ui_text.get_mut(entity)) {
                veh_health.text = veh_stats.max_health.to_string();
            }

            if let Some(veh_weight) = self.text_p1_weight.and_then(|entity| ui_text.get_mut(entity)) {
                veh_weight.text = determine_vehicle_weight_stats(veh_stats).to_string();
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

                let fetched_game_vehicle_setup = world.try_fetch_mut::<GameVehicleSetup>();

                if let Some(mut game_vehicle_setup) = fetched_game_vehicle_setup {
                    if Some(target) == self.button_p1_next_vehicle {
                        game_vehicle_setup.p1_name = get_next_vehicle_name(game_vehicle_setup.p1_name.clone());
                        
                        let fetched_game_vehicle_store = world.try_fetch::<VehicleStoreResource>();

                        if let Some(game_vehicle_store) = fetched_game_vehicle_store {
                            let vehicle_configs_map: &HashMap<VehicleNames, VehicleStats> = &game_vehicle_store.store;
            
                            let veh_stats = match vehicle_configs_map.get(&game_vehicle_setup.p1_name) {
                                Some(vehicle_config) => *vehicle_config,
                                _ => VehicleStats {
                                    max_shield: 0.0,
                                    max_armor: 0.0,
                                    max_health: 0.0,
                                    engine_force: 0.0,
                                    engine_weight: 0.0,
                                    width: 0.0,
                                    height: 0.0,
                                    max_velocity: 0.0,
                                    movement_type: VehicleMovementType::Hover,
                                    health_repair_rate: 0.0,
                                    health_repair_time: 0.0,
                                    shield_recharge_rate: 0.0,
                                    shield_cooldown: 0.0,
                                    shield_repair_time: 0.0,
                                    shield_radius: 0.0,
                                },
                            };

                            game_vehicle_setup.p1_stats = veh_stats;
                        }
                    }
                    if Some(target) == self.button_p1_prev_vehicle {
                        game_vehicle_setup.p1_name = get_prev_vehicle_name(game_vehicle_setup.p1_name.clone());
                        
                        let fetched_game_vehicle_store = world.try_fetch::<VehicleStoreResource>();

                        if let Some(game_vehicle_store) = fetched_game_vehicle_store {
                            let vehicle_configs_map: &HashMap<VehicleNames, VehicleStats> = &game_vehicle_store.store;
            
                            let veh_stats = match vehicle_configs_map.get(&game_vehicle_setup.p1_name) {
                                Some(vehicle_config) => *vehicle_config,
                                _ => VehicleStats {
                                    max_shield: 0.0,
                                    max_armor: 0.0,
                                    max_health: 0.0,
                                    engine_force: 0.0,
                                    engine_weight: 0.0,
                                    width: 0.0,
                                    height: 0.0,
                                    max_velocity: 0.0,
                                    movement_type: VehicleMovementType::Hover,
                                    health_repair_rate: 0.0,
                                    health_repair_time: 0.0,
                                    shield_recharge_rate: 0.0,
                                    shield_cooldown: 0.0,
                                    shield_repair_time: 0.0,
                                    shield_radius: 0.0,
                                },
                            };

                            game_vehicle_setup.p1_stats = veh_stats;
                        }
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
                .expect("Failed to remove CustomVehicleMenu");
        }

        self.ui_root = None;

        self.button_back_to_menu = None;

        self.button_p1_prev_vehicle = None;
        self.button_p1_next_vehicle = None;
        self.text_p1_vehicle_name = None;
        self.text_p1_shields = None;
        self.text_p1_armor = None;
        self.text_p1_health = None;
        self.text_p1_weight = None;
    }
}