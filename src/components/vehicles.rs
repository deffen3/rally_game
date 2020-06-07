use amethyst::{
    core::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entity, World},
};

use rand::Rng;
use std::f32::consts::PI;
use ron::de::from_reader;
use serde::Deserialize;
use std::{collections::HashMap, fs::File};

use crate::components::{Armor, Health, Player, Repair, Shield};
use crate::rally::{ARENA_HEIGHT, ARENA_WIDTH, UI_HEIGHT};
use crate::resources::GameModes;
use crate::entities::ui::PlayerStatusText;


#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum VehicleMovementType {
    Hover, //hover craft can turn to spin in place, and have the same friction regardless of velocity/vehicle angles
    Car, //cars can only turn if moving, and have high friction when velocity angle differs greatly from vehicle angle
    Tank, //tanks can turn to spin in place, and have high friction when velocity angle differs greatly from vehicle angle
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VehicleState {
    Active,
    InActive,
    InRespawn,
}

pub struct Vehicle {
    pub movement_type: VehicleMovementType,
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub dr: f32,
    pub angle_to_closest_vehicle: Option<f32>,
    pub dist_to_closest_vehicle: Option<f32>,
    pub angle_to_closest_targetable_vehicle: Option<f32>,
    pub dist_to_closest_targetable_vehicle: Option<f32>,
    pub collision_cooldown_timer: f32,
    pub health: Health,
    pub armor: Armor,
    pub shield: Shield,
    pub repair: Repair,
    pub engine_weight: f32,
    pub weapon_weight: f32,
    pub engine_force: f32,
    pub max_velocity: f32,
    pub malfunction: f32,
    pub malfunction_cooldown_timer: f32,
    pub ion_malfunction_pct: f32,
    pub respawn_timer: f32,
    pub death_x: f32,
    pub death_y: f32,
    pub death_angle: f32,
    pub state: VehicleState,
    pub player_status_text: PlayerStatusText,
}

pub fn determine_vehicle_weight(vehicle: &Vehicle) -> f32 {
    //typical vehicle weight = 100 at S:100/A:100/H:100 with normal engine efficiency

    //health makes up the main hull of the vehicle, and contributes 20 base weight + 20 per 100 health
    //shields make up 15 weight
    //armor another 25 weight
    //engine another 20 weight

    //typical weapon weight adds about 10.0
    //  for a total of about 110.0

    
    //a lighter racing vehicle with s:25/A:0/H:100 would weigh:
    //  B:20 + H:20 + S:3.75 + E:20 + W:10 = 73.75,
    //  and therefore would have about 50% quicker acceleration
    //  but could only take about 42% typical damage before blowing up

    //a heavy-weight tank combat vehicle with s:200/A:200/H:150 would weigh:
    //  B:20 + H:30 + S:30 + A:50 + E:20 + W:10 = 160,
    //  and therefore would have about 45% slower acceleration
    //  but would take almost 550 damage, an 83% increase


    //NOTE: lost armor does not contribute to weight, only the current value of armor matters
    let vehicle_weight = (20.0 + vehicle.health.max * 20. / 100.)
        + (vehicle.shield.max * 15. / 100.)
        + (vehicle.armor.value * 25. / 100.)
        + vehicle.engine_weight
        + vehicle.weapon_weight;

    vehicle_weight
}

pub fn determine_vehicle_weight_stats(vehicle: VehicleStats) -> f32 {
    let vehicle_weight = (20.0 + vehicle.max_health * 20. / 100.)
        + (vehicle.max_shield * 15. / 100.)
        + (vehicle.max_armor * 25. / 100.)
        + vehicle.engine_weight;

    vehicle_weight
}


impl Component for Vehicle {
    type Storage = DenseVecStorage<Self>;
}

impl Vehicle {
    pub fn new(
        player_status_text: PlayerStatusText,
        health_entity: Entity,
        armor_entity: Entity,
        shield_entity: Entity,
        repair_entity: Entity,
        max_shield: f32,
        max_armor: f32,
        max_health: f32,
        engine_force: f32,
        engine_weight: f32,
        max_velocity: f32,
        weapon_weight: f32,
        movement_type: VehicleMovementType,
        width: f32,
        height: f32,
    ) -> Vehicle {
        Vehicle {
            movement_type,
            width,
            height,
            dx: 0.0,
            dy: 0.0,
            dr: 0.0,
            angle_to_closest_vehicle: None,
            dist_to_closest_vehicle: None,
            angle_to_closest_targetable_vehicle: None,
            dist_to_closest_targetable_vehicle: None,
            collision_cooldown_timer: -1.0,
            health: Health {
                value: max_health,
                max: max_health,
                repair_rate: 5.0,
                entity: health_entity,
            },
            armor: Armor {
                value: max_armor,
                max: max_armor,
                entity: armor_entity,
            },
            shield: Shield {
                value: max_shield,
                max: max_shield,
                recharge_rate: 5.0,
                cooldown_timer: -1.0,
                cooldown_reset: 3.5,
                repair_timer: 0.0,
                repair_threshold: 5.0,
                radius: 15.0,
                entity: shield_entity,
            },
            repair: Repair {
                activated: false,
                init_timer: 0.0,
                init_threshold: 1.5,
                entity: repair_entity,
            },
            engine_force,
            engine_weight,
            weapon_weight,
            max_velocity,
            malfunction: 0.0,
            malfunction_cooldown_timer: -1.0,
            ion_malfunction_pct: 0.0,
            respawn_timer: 5.0,
            death_x: 0.0,
            death_y: 0.0,
            death_angle: 0.0,
            state: VehicleState::Active,
            player_status_text,
        }
    }
}

pub fn kill_restart_vehicle(
    player: &Player,
    vehicle: &mut Vehicle,
    transform: &mut Transform,
    stock_lives: i32,
) {
    vehicle.death_x = transform.translation().x;
    vehicle.death_y = transform.translation().y;

    let (_, _, vehicle_angle) = transform.rotation().euler_angles();
    vehicle.death_angle = vehicle_angle;

    //transform.set_translation_x(10.0 * ARENA_WIDTH);
    //transform.set_translation_y(10.0 * ARENA_HEIGHT);

    if stock_lives > 0 && player.deaths >= stock_lives {
        vehicle.state = VehicleState::InActive;
    } else {
        vehicle.state = VehicleState::InRespawn;
    }
}

pub fn check_respawn_vehicle(
    vehicle: &mut Vehicle,
    transform: &mut Transform,
    dt: f32,
    game_mode: GameModes,
    last_spawn_index: u32,
) -> u32 {
    let mut rng = rand::thread_rng();

    let mut spawn_index = last_spawn_index;

    if vehicle.state == VehicleState::InRespawn {
        vehicle.respawn_timer -= dt;

        if vehicle.respawn_timer < 0.0 {
            vehicle.state = VehicleState::Active;

            vehicle.respawn_timer = 5.0;

            vehicle.dx = 0.0;
            vehicle.dy = 0.0;
            vehicle.dr = 0.0;

            vehicle.shield.value = vehicle.shield.max;
            vehicle.shield.cooldown_timer = -1.;

            vehicle.armor.value = vehicle.armor.max;
            vehicle.health.value = vehicle.health.max;

            if game_mode == GameModes::Race {
                transform.set_rotation_2d(vehicle.death_angle);
                transform.set_translation_xyz(vehicle.death_x, vehicle.death_y, 0.0);
            } else {
                //Ensure that the spawn_index != last_spawn_index
                spawn_index = rng.gen_range(0, 3) as u32;

                if spawn_index >= last_spawn_index {
                    spawn_index += 1;
                }

                let spacing_factor = 5.0;
                let height = ARENA_HEIGHT + UI_HEIGHT;

                let (starting_rotation, starting_x, starting_y) = match spawn_index {
                    0 => (
                        -PI / 4.0,
                        ARENA_WIDTH / spacing_factor,
                        height / spacing_factor,
                    ),
                    1 => (
                        PI + PI / 4.0,
                        ARENA_WIDTH / spacing_factor,
                        height - (height / spacing_factor),
                    ),
                    2 => (
                        PI / 2.0 - PI / 4.0,
                        ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                        height / spacing_factor,
                    ),
                    3 => (
                        PI / 2.0 + PI / 4.0,
                        ARENA_WIDTH - (ARENA_WIDTH / spacing_factor),
                        height - (height / spacing_factor),
                    ),
                    _ => (
                        -PI / 4.0,
                        ARENA_WIDTH / spacing_factor,
                        height / spacing_factor,
                    ),
                };

                transform.set_rotation_2d(starting_rotation as f32);
                transform.set_translation_xyz(starting_x as f32, starting_y as f32, 0.0);
            }
        }
    }

    spawn_index
}

pub fn vehicle_damage_model(
    vehicle: &mut Vehicle,
    mut damage: f32,
    piercing_damage_pct: f32,
    shield_damage_pct: f32,
    armor_damage_pct: f32,
    health_damage_pct: f32,
) -> bool {
    let mut piercing_damage: f32 = 0.0;

    if piercing_damage_pct > 0.0 {
        piercing_damage = damage * piercing_damage_pct / 100.0;
        damage -= piercing_damage;
    }

    //println!("H:{:>6.3} A:{:>6.3} S:{:>6.3} P:{:>6.3}, D:{:>6.3}",vehicle.health, vehicle.armor, vehicle.shield, piercing_damage, damage);

    if vehicle.shield.value > 0.0 {
        vehicle.shield.value -= damage * shield_damage_pct / 100.0;
        damage = 0.0;

        if vehicle.shield.value < 0.0 {
            damage -= vehicle.shield.value; //over damage on shields, needs taken from armor
            vehicle.shield.value = 0.0;
        } else {
            //take damage to shields, but shields are still alive, reset shield recharge cooldown
            vehicle.shield.cooldown_timer = vehicle.shield.cooldown_reset;
        }
    }

    if vehicle.armor.value > 0.0 {
        vehicle.armor.value -= damage * armor_damage_pct / 100.0;
        damage = 0.0;

        if vehicle.armor.value < 0.0 {
            damage -= vehicle.armor.value; //over damage on armor, needs taken from health
            vehicle.armor.value = 0.0;
        }
    }

    let health_damage: f32 = (damage + piercing_damage) * health_damage_pct / 100.0;

    let mut vehicle_destroyed = false;

    if vehicle.health.value > 0.0 {
        //only destroy once
        if vehicle.health.value <= health_damage {
            vehicle_destroyed = true;
            vehicle.health.value = 0.0;
        } else {
            vehicle.health.value -= health_damage;
        }
    }

    //println!("H:{:>6.3} A:{:>6.3} S:{:>6.3}",vehicle.health, vehicle.armor, vehicle.shield);

    vehicle_destroyed
}




#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Hash, Eq)]
pub enum VehicleNames {
    MediumCombat,
    LightRacer,
    HeavyTank,
    CivilianCruiser,
    Interceptor,
    TSpeeder,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Hash, Eq)]
pub enum VehicleTypes {
    MediumCombat,
    LightRacer,
    HeavyTank,
    CivilianCruiser,
    Interceptor,
    TSpeeder,
}


pub fn get_next_vehicle_name(name: VehicleNames) -> VehicleNames {
    match name {
        VehicleNames::MediumCombat => VehicleNames::LightRacer,
        VehicleNames::LightRacer => VehicleNames::HeavyTank,
        VehicleNames::HeavyTank => VehicleNames::Interceptor,
        VehicleNames::Interceptor => VehicleNames::TSpeeder,
        VehicleNames::TSpeeder => VehicleNames::CivilianCruiser,
        VehicleNames::CivilianCruiser => VehicleNames::MediumCombat,
    }
}

pub fn get_prev_vehicle_name(name: VehicleNames) -> VehicleNames {
    match name {
        VehicleNames::MediumCombat => VehicleNames::CivilianCruiser,
        VehicleNames::LightRacer => VehicleNames::MediumCombat,
        VehicleNames::HeavyTank => VehicleNames::LightRacer,
        VehicleNames::Interceptor => VehicleNames::HeavyTank,
        VehicleNames::TSpeeder => VehicleNames::Interceptor,
        VehicleNames::CivilianCruiser => VehicleNames::TSpeeder,
    }
}



#[derive(Clone, Debug, Deserialize)]
pub struct VehicleStats {
    pub display_name: String,
    pub vehicle_type: VehicleTypes,
    pub max_shield: f32,
    pub max_armor: f32,
    pub max_health: f32,
    pub engine_force: f32,
    pub engine_weight: f32,
    pub width: f32,
    pub height: f32,
    pub sprite_scalar: f32,
    pub max_velocity: f32,
    pub movement_type: VehicleMovementType,
    pub health_repair_rate: f32,
    pub health_repair_time: f32,
    pub shield_recharge_rate: f32,
    pub shield_cooldown: f32,
    pub shield_repair_time: f32,
    pub shield_radius: f32,
}



#[derive(Clone)]
pub struct VehicleStoreResource {
    pub store: HashMap<VehicleNames, VehicleStats>,
}


pub fn build_vehicle_store(world: &mut World) -> VehicleStoreResource {
    let input_path = format!("{}/assets/game/vehicles.ron", env!("CARGO_MANIFEST_DIR"));
    let f = File::open(&input_path).expect("Failed opening file");

    let vehicle_configs_map: HashMap<VehicleNames, VehicleStats> =
        from_reader(f).expect("Failed to load config");

    let vehicle_store = VehicleStoreResource {
        store: vehicle_configs_map,
    };
    world.insert(vehicle_store.clone());

    vehicle_store
}

pub fn get_none_vehicle() -> VehicleStats {
    VehicleStats {
        display_name: "None".to_string(),
        vehicle_type: VehicleTypes::MediumCombat,
        max_shield: 0.0,
        max_armor: 0.0,
        max_health: 0.0,
        engine_force: 0.0,
        engine_weight: 0.0,
        width: 0.0,
        height: 0.0,
        sprite_scalar: 0.0,
        max_velocity: 0.0,
        movement_type: VehicleMovementType::Hover,
        health_repair_rate: 0.0,
        health_repair_time: 0.0,
        shield_recharge_rate: 0.0,
        shield_cooldown: 0.0,
        shield_repair_time: 0.0,
        shield_radius: 0.0,
    }
}


pub fn get_vehicle_sprites(vehicle_type: VehicleTypes) -> (usize, usize, usize) {
    let (vehicle_sprite_number, shield_sprite_number, armor_sprite_number) = match vehicle_type {
        VehicleTypes::MediumCombat => (0, 19, 20),
        VehicleTypes::LightRacer => (44, 19, 20),
        VehicleTypes::HeavyTank => (48, 57, 56),
        VehicleTypes::CivilianCruiser => (52, 19, 20),
        VehicleTypes::Interceptor => (58, 63, 62),
        VehicleTypes::TSpeeder => (64, 69, 68),
    };

    (vehicle_sprite_number, shield_sprite_number, armor_sprite_number)
}