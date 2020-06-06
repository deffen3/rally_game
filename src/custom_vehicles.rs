use amethyst::{
    assets::{Handle},
    core::transform::Transform,
    core::math::Vector3,
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder, UiText},
    renderer::{
        SpriteSheet, SpriteRender,
    },
    winit::VirtualKeyCode,
    utils::{
        removal::{exec_removal, Removal},
    },
};

use std::collections::HashMap;
use std::f32::consts::PI;

use crate::menu::MainMenu;
use crate::rally::load_sprite_sheet;

use crate::resources::{GameVehicleSetup};

use crate::entities::initialize_camera;

use crate::components::{determine_vehicle_weight_stats, get_vehicle_name_string, 
    VehicleMovementType, VehicleStoreResource, get_next_vehicle_name, get_prev_vehicle_name,
    VehicleNames, VehicleStats, VehicleTypes,
};


const BUTTON_BACK_TO_MENU: &str = "back_to_menu";

const BUTTON_PREV_VEHICLE: &str = "prev_vehicle";
const BUTTON_NEXT_VEHICLE: &str = "next_vehicle";

const TEXT_VEHICLE_NAME: &str = "veh_text";
const TEXT_SHIELDS: &str = "shields";
const TEXT_ARMOR: &str = "armor";
const TEXT_HEALTH: &str = "health";
const TEXT_WEIGHT: &str = "weight";



#[derive(Default, Debug)]
pub struct CustomVehiclesMenu {
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,

    ui_root: Option<Entity>,

    button_back_to_menu: Option<Entity>,

    button_p1_prev_vehicle: [Option<Entity>; 4],
    button_p1_next_vehicle: [Option<Entity>; 4],

    text_p1_vehicle_name: [Option<Entity>; 4],
    text_p1_shields: [Option<Entity>; 4],
    text_p1_armor: [Option<Entity>; 4],
    text_p1_health: [Option<Entity>; 4],
    text_p1_weight: [Option<Entity>; 4],

    p1_cur_vehicle_name: [Option<VehicleNames>; 4],
    p1_vehicle_sprite: [Option<Entity>; 4],

    camera: Option<Entity>,
}

impl SimpleState for CustomVehiclesMenu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        world.register::<Removal<u32>>();

        self.camera = Some(initialize_camera(world));

        self.sprite_sheet_handle.replace(load_sprite_sheet(
            world,
            "texture/rally_spritesheet.png".to_string(),
            "texture/rally_spritesheet.ron".to_string(),
        ));

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/custom_vehicles.ron", ())));

        self.p1_cur_vehicle_name = [None; 4];
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

        for player_index in 0..4 {
            if self.button_p1_prev_vehicle[player_index].is_none()
                || self.button_p1_next_vehicle[player_index].is_none()
                || self.text_p1_vehicle_name[player_index].is_none()
                || self.text_p1_shields[player_index].is_none()
                || self.text_p1_armor[player_index].is_none()
                || self.text_p1_health[player_index].is_none()
                || self.text_p1_weight[player_index].is_none()
            {
                world.exec(|ui_finder: UiFinder<'_>| {
                    self.button_p1_prev_vehicle[player_index] = ui_finder.find(&format!("p{}_{}", player_index+1, BUTTON_PREV_VEHICLE));
                    self.button_p1_next_vehicle[player_index] = ui_finder.find(&format!("p{}_{}", player_index+1, BUTTON_NEXT_VEHICLE));

                    self.text_p1_vehicle_name[player_index] = ui_finder.find(&format!("p{}_{}", player_index+1, TEXT_VEHICLE_NAME));
                    self.text_p1_shields[player_index] = ui_finder.find(&format!("p{}_{}", player_index+1, TEXT_SHIELDS));
                    self.text_p1_armor[player_index] = ui_finder.find(&format!("p{}_{}", player_index+1, TEXT_ARMOR));
                    self.text_p1_health[player_index] = ui_finder.find(&format!("p{}_{}", player_index+1, TEXT_HEALTH));
                    self.text_p1_weight[player_index] = ui_finder.find(&format!("p{}_{}", player_index+1, TEXT_WEIGHT));
                });
            }
        }


        for player_index in 0..4 {
            let p1_vehicle_name: Option<VehicleNames>;
            let p1_vehicle_sprite_type: Option<VehicleTypes>;

            {
                let mut ui_text = world.write_storage::<UiText>();
                let fetched_game_vehicle_setup = world.try_fetch::<GameVehicleSetup>();

                if let Some(game_vehicle_setup) = fetched_game_vehicle_setup {

                    if let Some(veh_name) = self.text_p1_vehicle_name[player_index].and_then(|entity| ui_text.get_mut(entity)) {
                        veh_name.text = get_vehicle_name_string(game_vehicle_setup.names[player_index].clone());
                    }

                    let veh_stats = game_vehicle_setup.stats[player_index];            

                    if let Some(veh_shields) = self.text_p1_shields[player_index].and_then(|entity| ui_text.get_mut(entity)) {
                        veh_shields.text = veh_stats.max_shield.to_string();
                    }

                    if let Some(veh_armor) = self.text_p1_armor[player_index].and_then(|entity| ui_text.get_mut(entity)) {
                        veh_armor.text = veh_stats.max_armor.to_string();
                    }

                    if let Some(veh_health) = self.text_p1_health[player_index].and_then(|entity| ui_text.get_mut(entity)) {
                        veh_health.text = veh_stats.max_health.to_string();
                    }

                    if let Some(veh_weight) = self.text_p1_weight[player_index].and_then(|entity| ui_text.get_mut(entity)) {
                        veh_weight.text = determine_vehicle_weight_stats(veh_stats).to_string();
                    }

                    if let Some(veh_name) = self.text_p1_vehicle_name[player_index].and_then(|entity| ui_text.get_mut(entity)) {
                        veh_name.text = get_vehicle_name_string(game_vehicle_setup.names[player_index].clone());
                    }

                    p1_vehicle_name = Some(game_vehicle_setup.names[player_index].clone());
                    p1_vehicle_sprite_type = Some(game_vehicle_setup.stats[player_index].vehicle_type.clone());
                }
                else {
                    p1_vehicle_name = None;
                    p1_vehicle_sprite_type = None;
                }
            }

            let p1_change_icon;

            if let Some(p1_cur_vehicle_name) = &self.p1_cur_vehicle_name[player_index] {
                if let Some(vehicle_name) = p1_vehicle_name.clone() {
                    if *p1_cur_vehicle_name != vehicle_name {
                        p1_change_icon = true;
                    }
                    else {
                        p1_change_icon = false;
                    }
                }
                else {
                    p1_change_icon = false;
                }
            }
            else {
                p1_change_icon = true;
            }

            if p1_change_icon {
                {
                    exec_removal(&world.entities(), &world.read_storage(), player_index as u32);
                }
            }


            let (x, y) = match player_index {
                0 => (100.0, 300.0),
                1 => (300.0, 300.0),
                2 => (100.0, 160.0),
                3 => (300.0, 160.0),
                _ => (100.0, 300.0),
            };

            if p1_change_icon {
                //UI vehicle icon
                let vehicle_sprite_number = match p1_vehicle_sprite_type.unwrap() {
                    VehicleTypes::MediumCombat => 0,
                    VehicleTypes::LightRacer => 44,
                    VehicleTypes::HeavyTank => 48,
                    VehicleTypes::CivilianCruiser => 52,
                    VehicleTypes::Interceptor => 58,
                };
            
                let vehicle_sprite_render = SpriteRender {
                    sprite_sheet: self.sprite_sheet_handle.clone().unwrap(),
                    sprite_number: vehicle_sprite_number + player_index,
                };

                let mut icon_transform = Transform::default();

                icon_transform.set_rotation_2d(-PI / 2.0);
                icon_transform.set_translation_xyz(x, y, 0.3);
                icon_transform.set_scale(Vector3::new(7., 7., 0.0));

                let p1_vehicle_sprite;
                {
                    p1_vehicle_sprite = world
                        .create_entity()
                        .with(icon_transform)
                        .with(vehicle_sprite_render.clone())
                        .with(Removal::new(player_index as u32))
                        .build();
                }

                self.p1_vehicle_sprite[player_index] = Some(p1_vehicle_sprite);
            }
        }

        Trans::None
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
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


                let fetched_game_vehicle_setup = data.world.try_fetch_mut::<GameVehicleSetup>();

                if let Some(mut game_vehicle_setup) = fetched_game_vehicle_setup {
                    for player_index in 0..4 {

                        if Some(target) == self.button_p1_next_vehicle[player_index] {
                            game_vehicle_setup.names[player_index] = get_next_vehicle_name(game_vehicle_setup.names[player_index].clone());
                            
                            let fetched_game_vehicle_store = data.world.try_fetch::<VehicleStoreResource>();

                            if let Some(game_vehicle_store) = fetched_game_vehicle_store {
                                let vehicle_configs_map: &HashMap<VehicleNames, VehicleStats> = &game_vehicle_store.store;
                
                                let veh_stats = match vehicle_configs_map.get(&game_vehicle_setup.names[player_index]) {
                                    Some(vehicle_config) => *vehicle_config,
                                    _ => VehicleStats {
                                        vehicle_type: VehicleTypes::MediumCombat,
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

                                game_vehicle_setup.stats[player_index] = veh_stats;
                            }
                        }
                        if Some(target) == self.button_p1_prev_vehicle[player_index] {
                            game_vehicle_setup.names[player_index] = get_prev_vehicle_name(game_vehicle_setup.names[player_index].clone());
                            
                            
                            let fetched_game_vehicle_store = data.world.try_fetch::<VehicleStoreResource>();

                            if let Some(game_vehicle_store) = fetched_game_vehicle_store {
                                let vehicle_configs_map: &HashMap<VehicleNames, VehicleStats> = &game_vehicle_store.store;
                
                                let veh_stats = match vehicle_configs_map.get(&game_vehicle_setup.names[player_index]) {
                                    Some(vehicle_config) => *vehicle_config,
                                    _ => VehicleStats {
                                        vehicle_type: VehicleTypes::MediumCombat,
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

                                game_vehicle_setup.stats[player_index] = veh_stats;
                            }
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

        for player_index in 0..4 {
            exec_removal(&data.world.entities(), &data.world.read_storage(), player_index as u32);
        }

        self.ui_root = None;

        self.button_back_to_menu = None;

        self.camera = None;

        self.p1_vehicle_sprite = [None; 4];

        self.button_p1_prev_vehicle = [None; 4];
        self.button_p1_next_vehicle = [None; 4];
        self.text_p1_vehicle_name = [None; 4];
        self.text_p1_shields = [None; 4];
        self.text_p1_armor = [None; 4];
        self.text_p1_health = [None; 4];
        self.text_p1_weight = [None; 4];
    }
}